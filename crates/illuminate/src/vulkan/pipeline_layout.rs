use std::rc::Rc;

use ash::vk;

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

    pub fn new(
        device: &Rc<Device>,
        layouts: &[vk::DescriptorSetLayout],
    ) -> Result<Self, DeviceError> {
        let vert_push_constant_range = vk::PushConstantRange::builder()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .offset(0)
            .size(64 /* 16 × 4 byte floats */)
            .build();
        let frag_push_constant_range = vk::PushConstantRange::builder()
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .offset(64)
            .size(4)
            .build();

        let push_constant_ranges = &[vert_push_constant_range, frag_push_constant_range];
        let create_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(layouts)
            .push_constant_ranges(push_constant_ranges);

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
