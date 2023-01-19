use alloc::rc::Rc;

use ash::vk;
use typed_builder::TypedBuilder;

use crate::vulkan::device::Device;
use crate::DeviceError;

#[derive(Clone, TypedBuilder)]
pub struct DescriptorSetLayoutCreateInfo<'a> {
    pub device: &'a Rc<Device>,
    pub descriptor_type: vk::DescriptorType,
    pub descriptor_count: u32,
    pub shader_stage_flags: vk::ShaderStageFlags,
}

pub struct DescriptorSetLayout {
    raw: vk::DescriptorSetLayout,
    device: Rc<Device>,
}

impl DescriptorSetLayout {
    pub fn raw(&self) -> vk::DescriptorSetLayout {
        self.raw
    }

    pub fn new(desc: DescriptorSetLayoutCreateInfo) -> Result<Self, DeviceError> {
        let device = desc.device;
        let ubo_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(desc.descriptor_type)
            .descriptor_count(desc.descriptor_count)
            .stage_flags(desc.shader_stage_flags)
            .build();
        let bindings = [ubo_binding];
        let create_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&bindings);
        let raw = device.create_descriptor_set_layout(&create_info)?;

        Ok(Self {
            raw,
            device: device.clone(),
        })
    }
}

impl Drop for DescriptorSetLayout {
    fn drop(&mut self) {
        self.device.destroy_descriptor_set_layout(self.raw);
    }
}
