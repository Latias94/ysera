use crate::RendererError;

pub mod main_camera_pass;

pub trait RenderPass: Sized {
    type RenderPassInitInfo: RenderPassInitInfo;
    fn initialize(init_info: &Self::RenderPassInitInfo) -> Result<Self, RendererError>;
}

pub trait RenderPassInitInfo: Sized {}
