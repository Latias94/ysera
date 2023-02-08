use std::sync::Arc;

use ash::vk;

use crate::vulkan::device::Device;
use crate::vulkan::shader::Shader;
use crate::DeviceError;

pub struct PipelineLayout {
    raw: vk::PipelineLayout,
    device: Arc<Device>,
}

impl PipelineLayout {
    pub fn raw(&self) -> vk::PipelineLayout {
        self.raw
    }

    pub unsafe fn new(
        device: &Arc<Device>,
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

        let raw = unsafe { device.raw().create_pipeline_layout(&create_info, None)? };
        Ok(Self {
            raw,
            device: device.clone(),
        })
    }
}

impl Drop for PipelineLayout {
    fn drop(&mut self) {
        unsafe {
            self.device.raw().destroy_pipeline_layout(self.raw, None);
        }
        log::debug!("Pipeline Layout destroyed.");
    }
}
