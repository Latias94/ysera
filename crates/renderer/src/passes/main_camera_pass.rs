use rhi::RHI;

use crate::passes::{Descriptor, Framebuffer, RenderPass, RenderPassInitInfo, RenderPipelineBase};
use crate::RendererError;

pub struct MainCameraPass<R: RHI> {
    framebuffer: Framebuffer<R>,
    descriptor: Descriptor<R>,
    render_pipeline: Vec<RenderPipelineBase<R>>,
}

pub struct MainCameraPassInitInfo {
    enable_fxaa: bool,
}

impl RenderPassInitInfo for MainCameraPassInitInfo {}

impl<R: RHI> RenderPass for MainCameraPass<R> {
    type RenderPassInitInfo = MainCameraPassInitInfo;

    fn initialize(init_info: &Self::RenderPassInitInfo) -> Result<Self, RendererError> {
        todo!()
    }
}

impl<R: RHI> MainCameraPass<R> {
    fn setup_attachments() -> Result<(), RendererError> {
        todo!()
    }

    fn setup_pipelines() -> Result<RenderPipelineBase<R>, RendererError> {
        todo!()
    }
}
