use crate::types::Label;
use crate::vulkan::device::Device;
use crate::DeviceError;
use ash::vk;
use std::sync::Arc;
use typed_builder::TypedBuilder;

pub struct Framebuffer {
    raw: vk::Framebuffer,
    render_pass: vk::RenderPass,
    device: Arc<Device>,
    image_views: Vec<vk::ImageView>,
    render_area: math::Rect2D,
    layers: u32,
}

#[derive(Clone, Hash, TypedBuilder)]
pub struct FramebufferDescriptor<'a> {
    pub label: Label<'a>,
    pub image_views: &'a [vk::ImageView],
    pub render_area: math::Rect2D,
    pub render_pass: vk::RenderPass,
    pub layers: u32,
}

impl Framebuffer {
    pub fn raw(&self) -> vk::Framebuffer {
        self.raw
    }

    pub fn new(device: &Arc<Device>, desc: &FramebufferDescriptor) -> Result<Self, DeviceError> {
        let create_info = vk::FramebufferCreateInfo::builder()
            .render_pass(desc.render_pass)
            .attachments(&desc.image_views)
            .width(desc.render_area.width as u32)
            .height(desc.render_area.height as u32)
            .layers(desc.layers)
            .build();
        let raw = unsafe { device.raw().create_framebuffer(&create_info, None)? };
        if let Some(label) = desc.label {
            unsafe { device.set_object_name(vk::ObjectType::RENDER_PASS, raw, label) };
        }
        Ok(Self {
            raw,
            render_pass: desc.render_pass,
            device: device.clone(),
            image_views: desc.image_views.to_vec(),
            render_area: desc.render_area,
            layers: desc.layers,
        })
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            self.device.raw().destroy_framebuffer(self.raw, None);
        }
    }
}
