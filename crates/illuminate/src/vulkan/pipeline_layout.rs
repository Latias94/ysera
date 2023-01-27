use std::rc::Rc;

use ash::vk;

use crate::vulkan::device::Device;
use crate::vulkan::shader::Shader;
use crate::DeviceError;

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
        shaders: &[Shader],
        layouts: &[vk::DescriptorSetLayout],
    ) -> Result<Self, DeviceError> {
        let push_constant_ranges = shaders
            .iter()
            .map(|shader| shader.get_push_constant_range())
            .filter_map(|mut push_const| push_const.take())
            .collect::<Vec<_>>();

        let create_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(layouts)
            .push_constant_ranges(&push_constant_ranges);

        let raw = device.create_pipeline_layout(&create_info)?;
        log::debug!("Vulkan pipeline layout created.");
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
