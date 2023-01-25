use crate::vulkan::buffer::{Buffer, StagingBufferDescriptor};
use crate::vulkan::command_buffer_allocator::CommandBufferAllocator;
use crate::vulkan::device::Device;
use crate::vulkan::image::{ColorImageDescriptor, Image};
use crate::vulkan::image_view::ImageView;
use crate::vulkan::sampler::Sampler;
use crate::DeviceError;
use ash::vk;
use gpu_allocator::vulkan::Allocator;
use image::io::Reader as ImageReader;
use image::EncodableLayout;
use parking_lot::Mutex;

use crate::vulkan::adapter::Adapter;
use crate::vulkan::instance::Instance;
use std::path::Path;
use std::rc::Rc;
use typed_builder::TypedBuilder;

#[derive(Clone, TypedBuilder)]
pub struct VulkanTextureDescriptor<'a> {
    pub adapter: &'a Adapter, // check mipmap format support
    pub instance: &'a Instance,
    pub device: &'a Rc<Device>,
    pub allocator: Rc<Mutex<Allocator>>,
    pub command_buffer_allocator: &'a CommandBufferAllocator,
    pub path: &'a Path,
    pub format: vk::Format,
    pub enable_mip_levels: bool,
}

pub struct VulkanTexture {
    image: Image,
    image_view: ImageView,
    sampler: Sampler,
}

impl VulkanTexture {
    pub fn width(&self) -> u32 {
        self.image.width()
    }

    pub fn height(&self) -> u32 {
        self.image.height()
    }

    pub fn raw_image_view(&self) -> vk::ImageView {
        self.image_view.raw()
    }

    pub fn raw_image(&self) -> vk::Image {
        self.image.raw()
    }

    pub fn raw_sampler(&self) -> vk::Sampler {
        self.sampler.raw()
    }

    pub fn image(&self) -> &Image {
        &self.image
    }

    pub fn image_view(&self) -> &ImageView {
        &self.image_view
    }

    pub fn sampler(&self) -> &Sampler {
        &self.sampler
    }

    pub fn new(desc: &VulkanTextureDescriptor) -> Result<VulkanTexture, DeviceError> {
        let display_path = desc.path.canonicalize().unwrap();

        let device = desc.device;
        let img = ImageReader::open(desc.path)
            .unwrap()
            .decode()
            .unwrap()
            .to_rgba8();
        let height = img.height();
        let width = img.width();
        let pixels = img.as_bytes();
        let mip_levels = if desc.enable_mip_levels {
            Image::max_mip_levels(width, height)
        } else {
            1
        };

        let staging_buffer_desc = StagingBufferDescriptor {
            label: Some("Vulkan Image Staging Buffer"),
            device,
            allocator: desc.allocator.clone(),
            elements: pixels,
            command_buffer_allocator: desc.command_buffer_allocator,
        };
        let staging_buffer = Buffer::new_staging_buffer(&staging_buffer_desc)?;

        let color_image_desc = ColorImageDescriptor {
            device,
            allocator: staging_buffer_desc.allocator.clone(),
            width,
            height,
            mip_levels,
            format: desc.format,
            samples: vk::SampleCountFlags::TYPE_1,
            extra_image_usage_flags: vk::ImageUsageFlags::TRANSFER_SRC, // cmd_blit_image
        };
        let mut image = Image::new_color_image(&color_image_desc)?;

        // TODO: 组合在一个命令缓冲区中并异步执行它们以获得更高的吞吐量
        image.transit_layout(
            desc.format,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            staging_buffer_desc.command_buffer_allocator,
            mip_levels,
        )?;

        image.copy_from(
            staging_buffer.raw(),
            width,
            height,
            staging_buffer_desc.command_buffer_allocator,
        )?;

        let image_view = ImageView::new_color_image_view(
            Some("VulkanTexture color image view"),
            device,
            image.raw(),
            image.format(),
            mip_levels,
        )?;

        let sampler = Sampler::new(device, mip_levels)?;

        log::debug!("VulkanTexture from '{}' created.", display_path.display());

        Self::generate_mipmaps(
            image.raw(),
            width,
            height,
            mip_levels,
            desc.command_buffer_allocator,
            desc.instance,
            desc.adapter,
            desc.format,
        )?;

        Ok(Self {
            image,
            image_view,
            sampler,
        })
    }

    fn generate_mipmaps(
        image: vk::Image,
        width: u32,
        height: u32,
        mip_levels: u32,
        command_buffer_allocator: &CommandBufferAllocator,
        instance: &Instance,
        adapter: &Adapter,
        format: vk::Format,
    ) -> Result<(), DeviceError> {
        let support_mip_levels = if mip_levels > 1 {
            unsafe {
                instance
                    .raw()
                    .get_physical_device_format_properties(adapter.raw(), format)
                    .optimal_tiling_features
                    .contains(vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR)
            }
        } else {
            true
        };
        if !support_mip_levels {
            // 在不支持的情况下，有两种选择。
            // 1. 可以实现一个函数，搜索常见的纹理图像格式，寻找支持 linear blitting 的格式
            // 2. 或者可以在软件中实现 mipmap 生成。然后，每个 mip 级别都可以以加载原始图像的相同方式加载到图像中。

            // 在运行时生成 mipmap 级别在实践中并不常见。通常它们是预先生成的，并与基本级别一起存储在纹理文件中，以提高加载速度。
            log::error!("Texture image format does not support linear blitting!");
        }

        command_buffer_allocator.create_single_use(|device, command_buffer| {
            let subresource = vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_array_layer(0)
                .layer_count(1)
                .level_count(1)
                .build();

            let mut barrier = vk::ImageMemoryBarrier::builder()
                .image(image)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .subresource_range(subresource)
                .build();

            let mut mip_width = width;
            let mut mip_height = height;

            for i in 1..mip_levels {
                barrier.subresource_range.base_mip_level = i - 1;
                barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
                barrier.new_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
                barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
                barrier.dst_access_mask = vk::AccessFlags::TRANSFER_READ;

                device.cmd_pipeline_barrier(
                    command_buffer.raw(),
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::DependencyFlags::empty(),
                    &[] as &[vk::MemoryBarrier],
                    &[] as &[vk::BufferMemoryBarrier],
                    &[barrier],
                );

                let src_subresource = vk::ImageSubresourceLayers::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .mip_level(i - 1)
                    .base_array_layer(0)
                    .layer_count(1)
                    .build();

                let dst_subresource = vk::ImageSubresourceLayers::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .mip_level(i)
                    .base_array_layer(0)
                    .layer_count(1)
                    .build();

                let blit = vk::ImageBlit::builder()
                    .src_offsets([
                        vk::Offset3D { x: 0, y: 0, z: 0 },
                        vk::Offset3D {
                            x: mip_width as i32,
                            y: mip_height as i32,
                            z: 1,
                        },
                    ])
                    .src_subresource(src_subresource)
                    .dst_offsets([
                        vk::Offset3D { x: 0, y: 0, z: 0 },
                        vk::Offset3D {
                            x: (if mip_width > 1 { mip_width / 2 } else { 1 }) as i32,
                            y: (if mip_height > 1 { mip_height / 2 } else { 1 }) as i32,
                            z: 1,
                        },
                    ])
                    .dst_subresource(dst_subresource)
                    .build();
                device.cmd_blit_image(
                    command_buffer.raw(),
                    image,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &[blit],
                    vk::Filter::LINEAR,
                );

                barrier.old_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
                barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
                barrier.src_access_mask = vk::AccessFlags::TRANSFER_READ;
                barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

                device.cmd_pipeline_barrier(
                    command_buffer.raw(),
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::FRAGMENT_SHADER,
                    vk::DependencyFlags::empty(),
                    &[] as &[vk::MemoryBarrier],
                    &[] as &[vk::BufferMemoryBarrier],
                    &[barrier],
                );

                if mip_width > 1 {
                    mip_width /= 2;
                }

                if mip_height > 1 {
                    mip_height /= 2;
                }
            }

            barrier.subresource_range.base_mip_level = mip_levels - 1;
            barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
            // 为了能够从着色器中的纹理图像开始采样
            barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
            barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

            device.cmd_pipeline_barrier(
                command_buffer.raw(),
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[] as &[vk::MemoryBarrier],
                &[] as &[vk::BufferMemoryBarrier],
                &[barrier],
            );
        })
    }
}
