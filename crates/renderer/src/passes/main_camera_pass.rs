use std::sync::Arc;

use rhi::types_v2::{
    RHIPipelineColorBlendStateCreateInfo, RHIPipelineDynamicStateCreateInfo,
    RHIPipelineInputAssemblyStateCreateInfo, RHIPipelineLayoutCreateInfo,
    RHIPipelineMultisampleStateCreateInfo, RHIPipelineRasterizationStateCreateFlags,
    RHIPipelineRasterizationStateCreateInfo, RHIPipelineVertexInputStateCreateInfo,
    RHIPipelineViewportStateCreateInfo,
};
use rhi::RHI;
use rhi_types::{
    RHIBlendFactor, RHIBlendOp, RHIColorComponentFlags, RHICullModeFlags, RHIDynamicState,
    RHIFrontFace, RHILogicOp, RHIPipelineColorBlendAttachmentState, RHIPolygonMode,
    RHIPrimitiveTopology, RHISampleCountFlagBits,
};

use crate::passes::{Descriptor, Framebuffer, RenderPass, RenderPassInitInfo, RenderPipelineBase};
use crate::shader::{Shader, ShaderDescriptor, ShaderUtil};
use crate::RendererError;

pub struct MainCameraPass<R: RHI> {
    // framebuffer: Framebuffer<R>,
    // descriptor: Descriptor<R>,
    // render_pipeline: Vec<RenderPipelineBase<R>>,
    // _marker: PhantomData<&'a ()>,
    rhi: R,
}

pub struct MainCameraPassInitInfo<R: RHI> {
    pub rhi: R,
}

impl<R: RHI> RenderPassInitInfo for MainCameraPassInitInfo<R> {}

impl<R: RHI> RenderPass for MainCameraPass<R> {
    type RenderPassInitInfo = MainCameraPassInitInfo<R>;

    fn initialize(init_info: Self::RenderPassInitInfo) -> Result<Self, RendererError> {
        // Self::setup_attachments(&init_info.rhi)?;
        let rhi = init_info.rhi;
        let pipeline = Self::setup_pipelines(&rhi)?;

        Ok(Self { rhi })
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
        let vertex_input_state = RHIPipelineVertexInputStateCreateInfo::builder().build();
        let input_assembly_state = RHIPipelineInputAssemblyStateCreateInfo::builder()
            .topology(RHIPrimitiveTopology::TRIANGLE_LIST)
            .build();
        let swapchain_desc = rhi.get_swapchain_info();

        let viewports = &[swapchain_desc.viewport];
        let scissors = &[swapchain_desc.scissor];
        let viewport_state = RHIPipelineViewportStateCreateInfo::builder()
            .viewports(viewports)
            .scissors(scissors)
            .build();
        let rasterization_state = RHIPipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(RHIPolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(RHICullModeFlags::BACK)
            .front_face(RHIFrontFace::CLOCKWISE)
            .depth_bias_enable(false)
            .build();
        let multisample_state = RHIPipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(RHISampleCountFlagBits::TYPE_1)
            .build();
        let attachment = RHIPipelineColorBlendAttachmentState::builder()
            .color_write_mask(RHIColorComponentFlags::RGBA)
            .blend_enable(false)
            .src_color_blend_factor(RHIBlendFactor::ONE) // Optional
            .dst_color_blend_factor(RHIBlendFactor::ZERO) // Optional
            .color_blend_op(RHIBlendOp::ADD) // Optional
            .src_alpha_blend_factor(RHIBlendFactor::ONE) // Optional
            .dst_alpha_blend_factor(RHIBlendFactor::ZERO) // Optional
            .alpha_blend_op(RHIBlendOp::ADD) // Optional
            .build();
        let attachments = &[attachment];
        let color_blend_state = RHIPipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(RHILogicOp::COPY)
            .attachments(attachments)
            .blend_constants([0.0, 0.0, 0.0, 0.0])
            .build();
        let dynamic_states = &[RHIDynamicState::VIEWPORT, RHIDynamicState::LINE_WIDTH];
        let dynamic_state = RHIPipelineDynamicStateCreateInfo::builder()
            .dynamic_states(dynamic_states)
            .build();
        let layout_info = RHIPipelineLayoutCreateInfo::builder().build();
        let pipeline_layout = unsafe { rhi.create_pipeline_layout(&layout_info)? };

        todo!()
    }
}
