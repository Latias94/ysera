use std::rc::Rc;

use ash::vk;

use crate::vulkan::descriptor_set_layout::DescriptorSetLayout;
use crate::vulkan::device::Device;
use crate::DeviceError;

pub struct PipelineLayout {
    raw: vk::PipelineLayout,
    device: Rc<Device>,
}

impl PipelineLayout {
    pub fn raw(&self) -> vk::PipelineLayout {
        self.raw
    }

    pub fn new(device: &Rc<Device>, layouts: &[DescriptorSetLayout]) -> Result<Self, DeviceError> {
        let raw_layouts: Vec<vk::DescriptorSetLayout> = layouts.iter().map(|x| x.raw()).collect();
        let create_info = vk::PipelineLayoutCreateInfo::builder().set_layouts(&raw_layouts);

        let raw = device.create_pipeline_layout(&create_info)?;
        Ok(Self {
            raw,
            device: device.clone(),
        })
    }
}

impl Drop for PipelineLayout {
    fn drop(&mut self) {
        self.device.destroy_pipeline_layout(self.raw);
        log::debug!("Pipeline Layout destroyed.");
    }
}
