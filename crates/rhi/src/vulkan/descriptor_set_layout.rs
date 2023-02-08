use alloc::rc::Rc;

use ash::vk;
use typed_builder::TypedBuilder;

use crate::vulkan::device::Device;
use crate::DeviceError;

#[derive(Clone, TypedBuilder)]
pub struct DescriptorSetLayoutCreateInfo<'a> {
    pub device: &'a Rc<Device>,
    pub bindings: &'a [DescriptorSetLayoutBinding],
}

pub struct DescriptorSetLayoutBinding {
    pub binding: u32,
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

    pub unsafe fn new(desc: DescriptorSetLayoutCreateInfo) -> Result<Self, DeviceError> {
        let device = desc.device;

        let bindings = desc
            .bindings
            .iter()
            .map(|binding| {
                vk::DescriptorSetLayoutBinding::builder()
                    .binding(binding.binding)
                    .descriptor_type(binding.descriptor_type)
                    .descriptor_count(binding.descriptor_count)
                    .stage_flags(binding.shader_stage_flags)
                    .build()
            })
            .collect::<Vec<vk::DescriptorSetLayoutBinding>>();
        let create_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&bindings);
        let raw = unsafe {
            device
                .raw()
                .create_descriptor_set_layout(&create_info, None)?
        };
        log::debug!("Descriptor Set Layout created.");

        Ok(Self {
            raw,
            device: device.clone(),
        })
    }
}

impl Drop for DescriptorSetLayout {
    fn drop(&mut self) {
        unsafe {
            self.device
                .raw()
                .destroy_descriptor_set_layout(self.raw, None);
        }
        log::debug!("Descriptor Set Layout destroyed.");
    }
}
