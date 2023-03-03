use std::collections::HashMap;

use rhi::types_v2::{
    RHIAttachmentDescription, RHIAttachmentReference, RHIFramebufferCreateInfo,
    RHIGraphicsPipelineCreateInfo, RHIPipelineBindPoint, RHIPipelineColorBlendStateCreateInfo,
    RHIPipelineDepthStencilStateCreateInfo, RHIPipelineDynamicStateCreateInfo,
    RHIPipelineInputAssemblyStateCreateInfo, RHIPipelineLayoutCreateInfo,
    RHIPipelineMultisampleStateCreateInfo, RHIPipelineRasterizationStateCreateInfo,
    RHIPipelineShaderStageCreateInfo, RHIPipelineTessellationStateCreateInfo,
    RHIPipelineVertexInputStateCreateInfo, RHIPipelineViewportStateCreateInfo,
    RHIRenderPassBeginInfo, RHIRenderPassCreateInfo, RHISubpassDependency, RHISubpassDescription,
    RHI_SUBPASS_EXTERNAL,
};
use rhi::RHI;
use rhi_types::{
    RHIAccessFlags, RHIAttachmentLoadOp, RHIAttachmentStoreOp, RHIBlendFactor, RHIBlendOp,
    RHIClearColorValue, RHIClearValue, RHIColorComponentFlags, RHICullModeFlags, RHIDynamicState,
    RHIFrontFace, RHIImageLayout, RHILogicOp, RHIOffset2D, RHIPipelineColorBlendAttachmentState,
    RHIPipelineStageFlags, RHIPolygonMode, RHIPrimitiveTopology, RHIRect2D, RHISampleCountFlagBits,
    RHISubpassContents,
};

use crate::passes::{RenderPass, RenderPassAttachmentType, RenderPassInitInfo, RenderPipelineBase};
use crate::shader::{Shader, ShaderDescriptor, ShaderUtil};
use crate::RendererError;

pub struct MainCameraPass<R: RHI> {
    pub framebuffers: Vec<R::Framebuffer>,
    pub render_pass: R::RenderPass,
    pub pipeline: RenderPipelineBase<R>,
    shaders: Vec<Shader<R>>,
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
        let mut attachments_map = HashMap::new();
        let render_pass = Self::setup_render_pass(&rhi, &mut attachments_map)?;
        let framebuffers = Self::setup_framebuffers(&rhi, &render_pass, attachments_map)?;
        let shaders = Self::setup_shaders(&rhi)?;
        let pipeline = Self::setup_pipelines(&rhi, &shaders, render_pass)?;
        // Self::destroy_shaders(&rhi, shaders);

        Ok(Self {
            rhi,
            framebuffers,
            render_pass,
            pipeline,
            shaders,
        })
    }
}

impl<R: RHI> MainCameraPass<R> {
    pub fn draw(&self, current_image_index: usize) -> Result<(), RendererError> {
        let swapchain_desc = self.rhi.get_swapchain_info();
        let clear_values = &[RHIClearValue {
            color: RHIClearColorValue {
                int32: [0, 0, 0, 0],
            },
        }];
        let render_pass_begin_info = RHIRenderPassBeginInfo::builder()
            .render_pass(&self.render_pass)
            .framebuffer(&self.framebuffers[current_image_index])
            .render_area(RHIRect2D {
                offset: RHIOffset2D { x: 0, y: 0 },
                extent: swapchain_desc.extent,
            })
            .clear_values(clear_values)
            .build();

        let cb = self.rhi.get_current_command_buffer();

        unsafe {
            self.rhi
                .cmd_begin_render_pass(cb, &render_pass_begin_info, RHISubpassContents::INLINE);
        }

        // ...
        unsafe {
            self.rhi.cmd_set_scissor(cb, 0, &[swapchain_desc.scissor]);
            self.rhi.cmd_set_viewport(cb, 0, &[swapchain_desc.viewport]);

            self.rhi
                .cmd_bind_pipeline(cb, RHIPipelineBindPoint::GRAPHICS, self.pipeline.pipeline);
            self.rhi.cmd_draw(cb, 3, 1, 0, 0);
        }

        unsafe {
            self.rhi.cmd_end_render_pass(cb);
        }

        Ok(())
    }
}

impl<R: RHI> MainCameraPass<R> {
    fn setup_attachments(rhi: &R) -> Result<(), RendererError> {
        todo!()
    }

    fn setup_render_pass(
        rhi: &R,
        attachments_map: &mut HashMap<RenderPassAttachmentType, RHIAttachmentDescription>,
    ) -> Result<R::RenderPass, RendererError> {
        let swapchain_desc = rhi.get_swapchain_info();

        let swapchain_image_attachment = RHIAttachmentDescription::builder()
            .format(swapchain_desc.image_format)
            .samples(RHISampleCountFlagBits::TYPE_1)
            .load_op(RHIAttachmentLoadOp::CLEAR)
            .store_op(RHIAttachmentStoreOp::STORE)
            .stecil_load_op(RHIAttachmentLoadOp::DONT_CARE)
            .stecil_store_op(RHIAttachmentStoreOp::DONT_CARE)
            .initial_layout(RHIImageLayout::UNDEFINED)
            .final_layout(RHIImageLayout::PRESENT_SRC_KHR)
            .build();

        let swapchain_image_attachment_ref = RHIAttachmentReference::builder()
            .attachment(0)
            .layout(RHIImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build();
        // layout(location = 0) out vec4 outColor
        let color_attachments = &[swapchain_image_attachment_ref];
        let subpass = RHISubpassDescription::builder()
            .pipeline_bind_point(RHIPipelineBindPoint::GRAPHICS)
            .color_attachments(color_attachments)
            .build();
        let dependency = RHISubpassDependency::builder()
            .src_subpass(RHI_SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(RHIPipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(RHIAccessFlags::empty())
            .dst_stage_mask(RHIPipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(RHIAccessFlags::COLOR_ATTACHMENT_WRITE)
            .build();
        let attachments = &[swapchain_image_attachment];
        let subpasses = &[subpass];
        let dependencies = &[dependency];
        let create_info = RHIRenderPassCreateInfo::builder()
            .attachments(attachments)
            .subpasses(subpasses)
            .dependencies(dependencies)
            .build();
        let render_pass = unsafe { rhi.create_render_pass(&create_info)? };
        Ok(render_pass)
    }

    fn setup_framebuffers(
        rhi: &R,
        render_pass: &R::RenderPass,
        attachments_map: HashMap<RenderPassAttachmentType, RHIAttachmentDescription>,
    ) -> Result<Vec<R::Framebuffer>, RendererError> {
        let swapchain_desc = rhi.get_swapchain_info();

        let image_view_counts = swapchain_desc.image_views.len();
        let mut framebuffers = Vec::with_capacity(image_view_counts);
        for image_view in swapchain_desc.image_views {
            let attachments = &[
                // gbuffer...
                // depth_image_view...
                image_view,
            ];
            let create_info = RHIFramebufferCreateInfo::builder()
                .render_pass(render_pass)
                .attachments(attachments)
                .width(swapchain_desc.extent.width)
                .height(swapchain_desc.extent.height)
                .layers(1)
                .build();
            let framebuffer = unsafe { rhi.create_framebuffer(&create_info)? };
            framebuffers.push(framebuffer);
        }

        Ok(framebuffers)
    }

    fn setup_shaders(rhi: &R) -> Result<Vec<Shader<R>>, RendererError> {
        let mut shaders = Vec::new();
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
        shaders.push(vert_shader);
        let frag_desc = ShaderDescriptor {
            label: Some("Fragment Shader"),
            spv_bytes: &frag_bytes,
            entry_name: "main",
        };
        let frag_shader = Shader::<R>::new(rhi, &frag_desc)?;
        shaders.push(frag_shader);
        Ok(shaders)
    }

    fn destroy_shaders(rhi: &R, shaders: Vec<Shader<R>>) {
        for shader in shaders {
            shader.destroy(rhi)
        }
    }

    fn setup_pipelines(
        rhi: &R,
        shaders: &[Shader<R>],
        render_pass: R::RenderPass,
    ) -> Result<RenderPipelineBase<R>, RendererError> {
        let shader_states = shaders
            .iter()
            .map(|shader| {
                RHIPipelineShaderStageCreateInfo::builder()
                    .stage(shader.stage)
                    .name(&shader.name)
                    .shader_module(shader.shader)
                    .build()
            })
            .collect::<Vec<_>>();

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
        let depth_stencil_state = RHIPipelineDepthStencilStateCreateInfo::builder().build();
        let tessellation_state = RHIPipelineTessellationStateCreateInfo::builder().build();
        let dynamic_states = &[RHIDynamicState::VIEWPORT, RHIDynamicState::SCISSOR];
        let dynamic_state = RHIPipelineDynamicStateCreateInfo::builder()
            .dynamic_states(dynamic_states)
            .build();
        let layout_info = RHIPipelineLayoutCreateInfo::builder().build();
        let pipeline_layout = unsafe { rhi.create_pipeline_layout(&layout_info)? };

        let create_info = RHIGraphicsPipelineCreateInfo::builder()
            .stages(&shader_states)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .color_blend_state(&color_blend_state)
            .depth_stencil_state(&depth_stencil_state)
            .dynamic_state(&dynamic_state)
            .tessellation_state(&tessellation_state)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .build();
        let pipeline = unsafe { rhi.create_graphics_pipeline(&create_info)? };
        Ok(RenderPipelineBase {
            pipeline_layout,
            pipeline,
        })
    }
}
