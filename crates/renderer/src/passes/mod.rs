use rhi::RHI;

use crate::RendererError;

pub mod main_camera_pass;

pub trait RenderPass: Sized {
    type RenderPassInitInfo: RenderPassInitInfo;
    fn initialize(init_info: Self::RenderPassInitInfo) -> Result<Self, RendererError>;
}

pub trait RenderPassInitInfo: Sized {}

pub struct FramebufferAttachment<R: RHI> {
    image: R::Image,
    allocation: R::Allocation,
    image_view: R::ImageView,
    format: R::Format,
}

pub struct Framebuffer<R: RHI> {
    width: u32,
    height: u32,
    framebuffer: R::Framebuffer,
    render_pass: R::RenderPass,
    attachment: Vec<FramebufferAttachment<R>>,
}

pub struct RenderPipelineBase<R: RHI> {
    pipeline_layout: R::PipelineLayout,
    pipeline: R::Pipeline,
}

pub struct Descriptor<R: RHI> {
    descriptor_set_layout: R::DescriptorSetLayout,
    descriptor_set: R::DescriptorSet,
}

pub enum RenderPassAttachmentType {
    SwapchainImage,
    Depth,
}
