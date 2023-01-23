use alloc::rc::Rc;

use ash::vk;
use typed_builder::TypedBuilder;

use crate::vulkan::device::Device;
use crate::DeviceError;

#[derive(Clone, TypedBuilder)]
pub struct DescriptorPoolCreateInfo<'a> {
    pub ty: vk::DescriptorType,
    pub descriptor_count: u32,
    pub device: &'a Rc<Device>,
    pub max_sets: u32,
}

pub struct DescriptorPool {
    raw: vk::DescriptorPool,
    device: Rc<Device>,
}

impl DescriptorPool {
    pub fn raw(&self) -> vk::DescriptorPool {
        self.raw
    }

    pub fn new(desc: DescriptorPoolCreateInfo) -> Result<Self, DeviceError> {
        let device = desc.device;
        let ubo_size = vk::DescriptorPoolSize::builder()
            .ty(desc.ty)
            .descriptor_count(desc.descriptor_count)
            .build();
        let sampler_size = vk::DescriptorPoolSize::builder()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(desc.descriptor_count)
            .build();
        let pool_sizes = &[ubo_size, sampler_size];
        let info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(pool_sizes)
            .max_sets(desc.max_sets);
        let raw = device.create_descriptor_pool(&info)?;
        log::debug!("Descriptor Pool created.");
        Ok(Self {
            raw,
            device: device.clone(),
        })
    }
}

impl Drop for DescriptorPool {
    fn drop(&mut self) {
        self.device.destroy_descriptor_pool(self.raw);
        log::debug!("Descriptor Pool destroyed.");
    }
}
