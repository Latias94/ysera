use crate::vulkan::command_buffer_allocator::CommandBufferAllocator;
use crate::vulkan::device::Device;
use crate::DeviceError;
use ash::vk;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, Allocator};
use gpu_allocator::MemoryLocation;
use parking_lot::Mutex;
use std::rc::Rc;

pub struct Image {
    raw: vk::Image,
    device: Rc<Device>,
    allocator: Rc<Mutex<Allocator>>,
    allocation: Option<Allocation>,
    format: vk::Format,
    width: u32,
    height: u32,
}

pub struct ImageDescriptor<'a> {
    pub device: &'a Rc<Device>,
    pub image_type: vk::ImageType,
    pub format: vk::Format,
    pub dimension: [u32; 2],
    pub mip_levels: u32,
    pub array_layers: u32,
    pub samples: vk::SampleCountFlags,
    pub tiling: vk::ImageTiling,
    pub usage: vk::ImageUsageFlags,
    pub sharing_mode: vk::SharingMode,
    pub allocator: Rc<Mutex<Allocator>>,
}

impl Image {
    pub fn raw(&self) -> vk::Image {
        self.raw
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn format(&self) -> vk::Format {
        self.format
    }

    pub fn new(desc: &ImageDescriptor) -> Result<Self, DeviceError> {
        let create_info = vk::ImageCreateInfo::builder()
            .image_type(desc.image_type)
            .extent(vk::Extent3D {
                width: desc.dimension[0],
                height: desc.dimension[1],
                depth: 1,
            })
            .mip_levels(desc.mip_levels)
            .array_layers(desc.array_layers)
            .format(desc.format)
            // vk::ImageTiling::LINEAR - Texels 是以行为主的顺序排列的，就像我们的像素阵列。平铺模式不能在以后的时间里改变。
            // 如果你想能够直接访问图像内存中的 texels，那么你必须使用 vk::ImageTiling::LINEAR。
            // vk::ImageTiling::OPTIMAL - Texels 是以执行已定义的顺序排列的，以达到最佳的访问效果。
            .tiling(desc.tiling)
            // vk::ImageLayout::UNDEFINED - 不能被 GPU 使用，而且第一次转换会丢弃纹理。
            // vk::ImageLayout::PREINITIALIZED - 不能被 GPU 使用，但第一次转换将保留 texels。
            // 很少有情况下需要在第一次转换时保留 texels。然而，有一个例子是，如果你想把一个图像和 vk::ImageTiling::LINEAR
            // 布局结合起来作为一个暂存图像(staging image)。在这种情况下，你想把文本数据上传到它那里，
            // 然后在不丢失数据的情况下把图像转换为传输源。然而，在我们的例子中，我们首先要把图像转换为传输目的地，
            // 然后从一个缓冲区对象中复制 texel 数据到它，所以我们不需要这个属性，可以安全地使用 vk::ImageLayout::UNDEFINED。
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(desc.usage)
            // 稀疏图像是只有某些区域实际上由内存支持的图像。例如，如果您对体素地形使用3D纹理，那么您可以使用它来避免分配内存来存储大量“空气”值。
            .samples(desc.samples)
            .sharing_mode(desc.sharing_mode);
        let device = desc.device;
        let raw = device.create_image(&create_info)?;

        // 为图像分配内存的方式与为缓冲区分配内存的方式完全相同。只不过这里使用 get_image_memory_requirements 而不是
        // get_buffer_memory_requirements，使用 bind_image_memory 而不是 bind_buffer_memory。
        let requirements = device.get_image_memory_requirements(raw);

        let allocator = desc.allocator.clone();
        let allocation = allocator
            .lock()
            .allocate(&AllocationCreateDesc {
                name: "Image",
                requirements,
                location: MemoryLocation::GpuOnly,
                linear: true,
            })
            .unwrap();

        unsafe {
            device
                .bind_image_memory(raw, allocation.memory(), allocation.offset())
                .unwrap()
        }

        Ok(Self {
            raw,
            device: desc.device.clone(),
            allocator,
            allocation: Some(allocation),
            format: desc.format,
            width: desc.dimension[0],
            height: desc.dimension[1],
        })
    }

    pub fn new_color_image(
        device: &Rc<Device>,
        allocator: Rc<Mutex<Allocator>>,
        width: u32,
        height: u32,
    ) -> Result<Self, DeviceError> {
        let image_desc = ImageDescriptor {
            device,
            image_type: vk::ImageType::TYPE_2D,
            format: vk::Format::R8G8B8A8_SRGB,
            dimension: [width, height],
            mip_levels: 1,
            array_layers: 1,
            samples: vk::SampleCountFlags::TYPE_1,
            tiling: vk::ImageTiling::OPTIMAL,
            usage: vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            allocator: allocator.clone(),
        };
        Self::new(&image_desc)
    }

    pub fn get_supported_format(
        instance: &ash::Instance,
        adapter: vk::PhysicalDevice,
        formats: &[vk::Format],
        tiling: vk::ImageTiling,
        features: vk::FormatFeatureFlags,
    ) -> Result<vk::Format, DeviceError> {
        formats
            .iter()
            .cloned()
            .find(|f| {
                let properties =
                    unsafe { instance.get_physical_device_format_properties(adapter, *f) };
                match tiling {
                    vk::ImageTiling::LINEAR => properties.linear_tiling_features.contains(features),
                    vk::ImageTiling::OPTIMAL => {
                        properties.optimal_tiling_features.contains(features)
                    }
                    _ => false,
                }
            })
            .ok_or(DeviceError::Other("Failed to find supported format!"))
    }

    pub fn get_depth_format(
        instance: &ash::Instance,
        adapter: vk::PhysicalDevice,
    ) -> Result<vk::Format, DeviceError> {
        let formats = &[
            vk::Format::D32_SFLOAT,
            vk::Format::D32_SFLOAT_S8_UINT,
            vk::Format::D24_UNORM_S8_UINT,
        ];

        Image::get_supported_format(
            instance,
            adapter,
            formats,
            vk::ImageTiling::OPTIMAL,
            vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        )
    }

    /// 屏障主要用于同步目的，因此必须指定哪些类型的涉及资源的操作必须发生在屏障之前，哪些涉及资源的操作必须等待屏障。
    pub fn transit_layout(
        &mut self,
        format: vk::Format,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
        command_buffer_allocator: &CommandBufferAllocator,
    ) -> Result<(), DeviceError> {
        command_buffer_allocator.create_single_use(|device, command_buffer| {
            let (src_access_mask, dst_access_mask, src_stage_mask, dst_stage_mask) =
                match (old_layout, new_layout) {
                    (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
                        vk::AccessFlags::empty(),
                        vk::AccessFlags::TRANSFER_WRITE,
                        vk::PipelineStageFlags::TOP_OF_PIPE,
                        vk::PipelineStageFlags::TRANSFER,
                    ),
                    (
                        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                    ) => (
                        vk::AccessFlags::TRANSFER_WRITE,
                        vk::AccessFlags::SHADER_READ,
                        vk::PipelineStageFlags::TRANSFER,
                        vk::PipelineStageFlags::FRAGMENT_SHADER,
                    ),
                    _ => panic!("Unsupported image layout transition!"),
                };

            let subresource = vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1)
                .build();
            let barrier = vk::ImageMemoryBarrier::builder()
                .old_layout(old_layout)
                .new_layout(new_layout)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(self.raw)
                .subresource_range(subresource)
                .src_access_mask(src_access_mask)
                .dst_access_mask(dst_access_mask)
                .build();
            // https://www.khronos.org/registry/vulkan/specs/1.0/html/vkspec.html#synchronization-access-types-supported
            device.cmd_pipeline_barrier(
                command_buffer.raw(),
                src_stage_mask,
                dst_stage_mask,
                vk::DependencyFlags::empty(),
                &[] as &[vk::MemoryBarrier],
                &[] as &[vk::BufferMemoryBarrier],
                &[barrier],
            );
        })?;

        Ok(())
    }

    pub fn copy_from(
        &mut self,
        buffer: vk::Buffer,
        width: u32,
        height: u32,
        command_buffer_allocator: &CommandBufferAllocator,
    ) -> Result<(), DeviceError> {
        command_buffer_allocator.create_single_use(|device, command_buffer| {
            let subresource = vk::ImageSubresourceLayers::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .mip_level(0)
                .base_array_layer(0)
                .layer_count(1)
                .build();

            let region = vk::BufferImageCopy::builder()
                .buffer_offset(0)
                .buffer_row_length(0)
                .buffer_image_height(0)
                .image_subresource(subresource)
                .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
                .image_extent(vk::Extent3D {
                    width,
                    height,
                    depth: 1,
                })
                .build();

            device.cmd_copy_buffer_to_image(
                command_buffer.raw(),
                buffer,
                self.raw,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[region],
            );
        })?;

        Ok(())
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        let allocation = self.allocation.take();
        if let Some(allocation) = allocation {
            self.allocator.lock().free(allocation).unwrap();
        }
        self.device.destroy_image(self.raw);
    }
}
