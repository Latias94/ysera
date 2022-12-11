use ash::vk;

pub struct PipelineLayout {
    raw: vk::PipelineLayout,
}

impl PipelineLayout {
    pub fn raw(&self) -> vk::PipelineLayout {
        self.raw
    }
}
