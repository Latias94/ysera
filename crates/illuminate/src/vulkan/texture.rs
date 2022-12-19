use crate::vulkan::device::Device;
use crate::DeviceError;
use ash::vk;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, Allocator};
use gpu_allocator::MemoryLocation;
use parking_lot::Mutex;
use std::rc::Rc;

pub struct Texture {
    raw: vk::Image,
    device: Rc<Device>,
    allocator: Rc<Mutex<Allocator>>,
    allocation: Option<Allocation>,
    format: vk::Format,
    width: u32,
    height: u32,
}

pub struct TextureDescriptor<'a> {
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
    pub properties: vk::MemoryPropertyFlags,
    pub allocator: Rc<Mutex<Allocator>>,
}

impl Texture {
    pub fn raw(&self) -> vk::Image {
        self.raw
    }

    pub fn new(desc: TextureDescriptor) -> Result<Self, DeviceError> {
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
            .samples(desc.samples)
            .sharing_mode(desc.sharing_mode);
        let device = desc.device;
        let raw = device.create_texture(&create_info)?;

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
}

impl Drop for Texture {
    fn drop(&mut self) {
        let allocation = self.allocation.take();
        if let Some(allocation) = allocation {
            self.allocator.lock().free(allocation).unwrap();
        }
        self.device.destroy_texture(self.raw);
    }
}
