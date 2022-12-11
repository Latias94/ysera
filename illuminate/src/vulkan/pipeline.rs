use super::{device::Device, pipeline_layout::PipelineLayout};
use crate::vulkan::shader::Shader;
use crate::{DeviceError, Label};
use ash::vk;
use std::ffi::CString;
use std::rc::Rc;
use typed_builder::TypedBuilder;

pub struct Pipeline {
    raw: vk::Pipeline,
    device: Rc<Device>,
    pipeline_layout: PipelineLayout,
}

#[derive(Clone, TypedBuilder)]
pub struct PipelineDescriptor<'a> {
    pub label: Label<'a>,
}

impl Pipeline {
    pub fn raw(&self) -> vk::Pipeline {
        self.raw
    }

    pub fn new() -> Result<Self, DeviceError> {
        todo!()
    }

    pub fn create_graphics_pipeline(
        device: &ash::Device,
        render_pass: vk::RenderPass,
        swapchain_extent: vk::Extent2D,
        descriptor_set_layout: vk::DescriptorSetLayout,
        msaa_samples: vk::SampleCountFlags,
        shader: Shader,
    ) -> Result<(vk::Pipeline, vk::PipelineLayout), DeviceError> {
        profiling::scope!("create_graphics_pipeline");

        // the beginning function name in shader code.
        let vert_entry_name = CString::new(shader.vert_entry_name()).unwrap();
        let frag_entry_name = CString::new(shader.frag_entry_name()).unwrap();

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::builder()
                .module(shader.vert_shader_module())
                .name(&vert_entry_name)
                .stage(vk::ShaderStageFlags::VERTEX)
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .module(shader.frag_shader_module())
                .name(&frag_entry_name)
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .build(),
        ];

        // let binding_descriptions = &[Vertex::binding_description()];
        // let attribute_descriptions = Vertex::attribute_descriptions();
        // let vertex_input_state_create_info = vk::PipelineVertexInputStateCreateInfo::builder()
        //     .vertex_binding_descriptions(binding_descriptions)
        //     .vertex_attribute_descriptions(&attribute_descriptions)
        //     .build();

        let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            // Normally, the vertices are loaded from the vertex buffer by index in sequential order,
            // but with an element buffer you can specify the indices to use yourself. This allows
            // you to perform optimizations like reusing vertices. If you set the `primitive_restart_enable`
            // member to true, then it's possible to break up lines and triangles in the STRIP
            // topology modes by using a special index of 0xFFFF or 0xFFFFFFFF.
            .primitive_restart_enable(false)
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .build();

        let viewports = [vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: swapchain_extent.width as f32,
            height: swapchain_extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        }];

        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: swapchain_extent,
        }];
        let viewport_state_create_info = vk::PipelineViewportStateCreateInfo::builder()
            .scissors(&scissors)
            .viewports(&viewports)
            .build();

        let rasterization_state_create_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            // 由于我们在投影矩阵中做了 Y 方向的翻转，顶点现在是以逆时针顺序而不是顺时针顺序被绘制的。
            // 这导致了背面剔除的启动，并阻止了任何几何体的绘制。
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false)
            .build();

        // let multisample_state_create_info = vk::PipelineMultisampleStateCreateInfo::builder()
        //     .sample_shading_enable(true)
        //     .min_sample_shading(0.2)
        //     .rasterization_samples(msaa_samples)
        //     .build();

        // let stencil_state = vk::StencilOpState {
        //     fail_op: vk::StencilOp::KEEP,
        //     pass_op: vk::StencilOp::KEEP,
        //     depth_fail_op: vk::StencilOp::KEEP,
        //     compare_op: vk::CompareOp::ALWAYS,
        //     compare_mask: 0,
        //     write_mask: 0,
        //     reference: 0,
        // };

        let depth_stencil_state_create_info = vk::PipelineDepthStencilStateCreateInfo::builder()
            // depth_test_enable 字段指定是否应将新片段的深度与深度缓冲区进行比较，看它们是否应被丢弃。
            .depth_test_enable(true)
            // depth_write_enable 字段指定是否应将通过深度测试的新片段的深度实际写入深度缓冲区。
            .depth_write_enable(true)
            // depth_compare_op 字段指定了为保留或丢弃片段所进行的比较。我们坚持较低的深度 = 较近的惯例，所以新片段的深度应该较小。
            .depth_compare_op(vk::CompareOp::LESS)
            // depth_bounds_test_enable、min_depth_bounds 和 max_depth_bounds 字段用于可选的深度边界测试。
            // 基本上，这允许你只保留落在指定深度范围内的片段。我们将不会使用这个功能。
            .depth_bounds_test_enable(false)
            .min_depth_bounds(0.0) // Optional.
            .max_depth_bounds(1.0) // Optional.
            // 最后三个字段配置了模板缓冲区的操作，
            // 如果你想使用这些操作，那么你必须确保深度 / 模板图像的格式包含一个模板组件。
            .stencil_test_enable(false)
            // .front(/* vk::StencilOpState */) // Optional.
            // .back(/* vk::StencilOpState */); // Optional.
            .build();

        let color_blend_attachment_states = [vk::PipelineColorBlendAttachmentState {
            blend_enable: vk::FALSE,
            color_write_mask: vk::ColorComponentFlags::RGBA,
            src_color_blend_factor: vk::BlendFactor::ONE,
            dst_color_blend_factor: vk::BlendFactor::ZERO,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ONE,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
        }];

        let color_blend_state_create_info = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&color_blend_attachment_states)
            .blend_constants([0.0, 0.0, 0.0, 0.0])
            .build();

        let set_layouts = &[descriptor_set_layout];
        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(set_layouts)
            .build();

        let pipeline_layout =
            unsafe { device.create_pipeline_layout(&pipeline_layout_create_info, None)? };

        log::debug!("Vulkan pipeline layout created.");

        let dynamic_state = Box::new([vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR]);
        let dynamic_state_create_info = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(dynamic_state.as_ref())
            .build();

        let graphic_pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            // .vertex_input_state(&vertex_input_state_create_info)
            .input_assembly_state(&vertex_input_assembly_state_info)
            .viewport_state(&viewport_state_create_info)
            .rasterization_state(&rasterization_state_create_info)
            // .multisample_state(&multisample_state_create_info)
            .depth_stencil_state(&depth_stencil_state_create_info)
            .color_blend_state(&color_blend_state_create_info)
            .dynamic_state(&dynamic_state_create_info)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0)
            .build();

        let graphic_pipeline_create_infos = &[graphic_pipeline_create_info];

        let graphics_pipelines = unsafe {
            device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    graphic_pipeline_create_infos,
                    None,
                )
                .map_err(|(_, e)| crate::DeviceError::from(e))
        }?;
        log::debug!("Vulkan pipelines created.");

        Ok((graphics_pipelines[0], pipeline_layout))
    }
}
