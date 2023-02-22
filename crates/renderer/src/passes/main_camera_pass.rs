use crate::passes::{RenderPass, RenderPassInitInfo};
use crate::RendererError;

pub struct MainCameraPass {}

pub struct MainCameraPassInitInfo {
    enable_fxaa: bool,
}
impl RenderPassInitInfo for MainCameraPassInitInfo {}

impl RenderPass for MainCameraPass {
    type RenderPassInitInfo = MainCameraPassInitInfo;

    fn initialize(init_info: &Self::RenderPassInitInfo) -> Result<Self, RendererError> {
        todo!()
    }
}
