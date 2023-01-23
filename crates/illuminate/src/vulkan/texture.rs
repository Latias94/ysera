use crate::vulkan::buffer::{Buffer, StagingBufferDescriptor};
use crate::vulkan::command_buffer_allocator::CommandBufferAllocator;
use crate::vulkan::device::Device;
use crate::vulkan::image::Image;
use crate::vulkan::image_view::ImageView;
use crate::vulkan::sampler::Sampler;
use crate::DeviceError;
use ash::vk;
use gpu_allocator::vulkan::Allocator;
use image::io::Reader as ImageReader;
use image::EncodableLayout;
use parking_lot::Mutex;

use std::path::Path;
use std::rc::Rc;

pub struct VulkanTextureDescriptor<'a> {
    pub device: &'a Rc<Device>,
    pub allocator: Rc<Mutex<Allocator>>,
    pub command_buffer_allocator: &'a CommandBufferAllocator,
    pub path: &'a Path,
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
        let desc = StagingBufferDescriptor {
            label: Some("Vulkan Image Staging Buffer"),
            device,
            allocator: desc.allocator.clone(),
            elements: pixels,
            command_buffer_allocator: desc.command_buffer_allocator,
        };
        let staging_buffer = Buffer::new_staging_buffer(&desc)?;

        let mut image = Image::new_color_image(device, desc.allocator.clone(), width, height)?;

        // TODO: 组合在一个命令缓冲区中并异步执行它们以获得更高的吞吐量
        image.transit_layout(
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            desc.command_buffer_allocator,
        )?;

        image.copy_from(
            staging_buffer.raw(),
            width,
            height,
            desc.command_buffer_allocator,
        )?;

        // 为了能够从着色器中的纹理图像开始采样
        image.transit_layout(
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            desc.command_buffer_allocator,
        )?;

        let image_view = ImageView::new_color_image_view(
            Some("VulkanTexture color image view"),
            device,
            image.raw(),
            image.format(),
        )?;

        let sampler = Sampler::new(device)?;

        log::debug!("VulkanTexture from '{}' created.", display_path.display());

        Ok(Self {
            image,
            image_view,
            sampler,
        })
    }
}
