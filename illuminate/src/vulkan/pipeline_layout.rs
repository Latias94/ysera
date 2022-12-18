use crate::vulkan::device::Device;
use crate::DeviceError;
use ash::vk;
use std::rc::Rc;

pub struct PipelineLayout {
    raw: vk::PipelineLayout,
    device: Rc<Device>,
}

impl PipelineLayout {
    pub fn raw(&self) -> vk::PipelineLayout {
        self.raw
    }

    pub fn new(
        device: &Rc<Device>,
        layouts: &[vk::DescriptorSetLayout],
    ) -> Result<Self, DeviceError> {
        let create_info = vk::PipelineLayoutCreateInfo::builder().set_layouts(layouts);

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
