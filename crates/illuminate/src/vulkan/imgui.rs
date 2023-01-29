use alloc::rc::Rc;
use std::collections::HashSet;
use std::sync::Arc;

use ash::vk;
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};
use imgui::Context as ImguiContext;
use imgui::TextureId;
use imgui_rs_vulkan_renderer::{Options, Renderer};
use typed_builder::TypedBuilder;

use crate::vulkan::adapter::Adapter;
use crate::vulkan::descriptor_set_allocator::DescriptorSetAllocator;
use crate::vulkan::device::Device;
use crate::vulkan::instance::Instance;
use crate::vulkan::texture::VulkanTexture;
use crate::{DeviceError, MAX_FRAMES_IN_FLIGHT};

pub struct ImguiRenderer {
    _device: Rc<Device>,
    renderer: Renderer,
    texture_id_set: HashSet<TextureId>,
    descriptor_set_allocator: Rc<DescriptorSetAllocator>,
}

#[derive(TypedBuilder)]
pub struct ImguiRendererDescriptor<'a> {
    pub instance: Rc<Instance>,
    pub adapter: Rc<Adapter>,
    pub device: Rc<Device>,
    pub queue: vk::Queue,
    pub format: vk::Format,
    pub command_pool: vk::CommandPool,
    pub render_pass: vk::RenderPass,
    pub context: &'a mut ImguiContext,
    pub descriptor_set_allocator: Rc<DescriptorSetAllocator>,
}

impl ImguiRenderer {
    pub fn renderer_mut(&mut self) -> &mut Renderer {
        &mut self.renderer
    }

    pub fn new(desc: &mut ImguiRendererDescriptor) -> anyhow::Result<Self> {
        let device_properties = unsafe {
            desc.instance
                .raw()
                .get_physical_device_properties(desc.adapter.raw())
        };

        desc.context.fonts().tex_desired_width =
            device_properties.limits.max_image_dimension2_d as i32;

        let options = Some(Options {
            in_flight_frames: MAX_FRAMES_IN_FLIGHT,
            enable_depth_test: true,
            enable_depth_write: true,
        });

        let allocator = Allocator::new(&AllocatorCreateDesc {
            instance: desc.instance.raw().clone(),
            device: desc.device.raw().clone(),
            physical_device: desc.adapter.raw(),
            debug_settings: Default::default(),
            buffer_device_address: false,
        })?;

        let renderer = Renderer::with_gpu_allocator(
            Arc::new(std::sync::Mutex::new(allocator)),
            desc.device.raw().clone(),
            desc.queue,
            desc.command_pool,
            desc.render_pass,
            desc.context,
            options,
        )?;

        Ok(Self {
            renderer,
            descriptor_set_allocator: desc.descriptor_set_allocator.clone(),
            texture_id_set: HashSet::new(),
            _device: desc.device.clone(),
        })
    }

    /// Register a texture
    /// https://github.com/ocornut/imgui/pull/914
    pub fn add_texture(
        &mut self,
        texture: &VulkanTexture,
        image_layout: vk::ImageLayout,
    ) -> Result<TextureId, DeviceError> {
        let set = self
            .descriptor_set_allocator
            .allocate_texture_descriptor_set(texture, image_layout)?;
        let texture_id = self.renderer.textures().insert(set);
        Ok(texture_id)
    }
}
