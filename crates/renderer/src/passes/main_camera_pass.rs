use std::sync::Arc;

use rhi::RHI;

use crate::passes::{Descriptor, Framebuffer, RenderPass, RenderPassInitInfo, RenderPipelineBase};
use crate::shader::{Shader, ShaderDescriptor, ShaderUtil};
use crate::RendererError;

pub struct MainCameraPass<R: RHI> {
    framebuffer: Framebuffer<R>,
    descriptor: Descriptor<R>,
    render_pipeline: Vec<RenderPipelineBase<R>>,
    // _marker: PhantomData<&'a ()>,
}

pub struct MainCameraPassInitInfo<R: RHI> {
    // pub rhi: &'a R,
    pub rhi: Arc<R>,
}

impl<R: RHI> RenderPassInitInfo for MainCameraPassInitInfo<R> {}

impl<R: RHI> RenderPass for MainCameraPass<R> {
    type RenderPassInitInfo = MainCameraPassInitInfo<R>;

    fn initialize(init_info: &Self::RenderPassInitInfo) -> Result<Self, RendererError> {
        // Self::setup_attachments(&init_info.rhi)?;
        let pipeline = Self::setup_pipelines(&init_info.rhi)?;
        todo!()
    }
}

impl<R: RHI> MainCameraPass<R> {
    fn setup_attachments(rhi: &R) -> Result<(), RendererError> {
        todo!()
    }

    fn setup_pipelines(rhi: &R) -> Result<RenderPipelineBase<R>, RendererError> {
        let vert_bytes =
            unsafe { ShaderUtil::load_pre_compiled_spv_bytes_from_name("triangle.vert") };
        let frag_bytes =
            unsafe { ShaderUtil::load_pre_compiled_spv_bytes_from_name("triangle.frag") };
        let vert_desc = ShaderDescriptor {
            label: Some("Vertex Shader"),
            spv_bytes: &vert_bytes,
            entry_name: "main",
        };
        let vert_shader = Shader::<R>::new(rhi, &vert_desc)?;
        let frag_desc = ShaderDescriptor {
            label: Some("Fragment Shader"),
            spv_bytes: &frag_bytes,
            entry_name: "main",
        };
        let frag_shader = Shader::<R>::new(rhi, &frag_desc)?;

        todo!()
    }
}
