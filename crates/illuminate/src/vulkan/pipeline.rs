use alloc::ffi::CString;
use std::rc::Rc;

use ash::vk;
use typed_builder::TypedBuilder;

use crate::vulkan::descriptor_set_layout::DescriptorSetLayout;
use crate::vulkan::shader::Shader;
use crate::{DeviceError, Label};

use super::{device::Device, pipeline_layout::PipelineLayout};

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

    pub fn raw_pipeline_layout(&self) -> vk::PipelineLayout {
        self.pipeline_layout.raw()
    }

    pub fn new(
        device: &Rc<Device>,
        render_pass: vk::RenderPass,
        // msaa_samples: vk::SampleCountFlags,
        descriptor_set_layouts: &[vk::DescriptorSetLayout],
        shader: Shader,
    ) -> Result<Self, DeviceError> {
        let pipeline_layout = PipelineLayout::new(device, descriptor_set_layouts)?;
        let raw =
            Self::create_graphics_pipeline(device, render_pass, pipeline_layout.raw(), shader)?[0];

        Ok(Self {
            raw,
            device: device.clone(),
            pipeline_layout,
        })
    }

    pub fn create_graphics_pipeline(
        device: &Rc<Device>,
        render_pass: vk::RenderPass,
        pipeline_layout: vk::PipelineLayout,
        // msaa_samples: vk::SampleCountFlags,
        shader: Shader,
    ) -> Result<Vec<vk::Pipeline>, DeviceError> {
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

        let binding_descriptions = &[shader.get_binding_description()];
        let attribute_descriptions = shader.get_attribute_descriptions();
        let vertex_input_state_create_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(binding_descriptions)
            .vertex_attribute_descriptions(&attribute_descriptions);

        let vertex_input_assembly_state_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            // Normally, the vertices are loaded from the vertex buffer by index in sequential order,
            // but with an element buffer you can specify the indices to use yourself. This allows
            // you to perform optimizations like reusing vertices. If you set the `primitive_restart_enable`
            // member to true, then it's possible to break up lines and triangles in the STRIP
            // topology modes by using a special index of 0xFFFF or 0xFFFFFFFF.
            .primitive_restart_enable(false)
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

        let viewport_state_create_info = vk::PipelineViewportStateCreateInfo::builder()
            .scissor_count(1)
            .viewport_count(1);

        let rasterization_state_create_info = vk::PipelineRasterizationStateCreateInfo::builder()
            // If depth_clamp_enable is set to true, then fragments that are beyond the near and far
            // planes are clamped to them as opposed to discarding them. This is useful in some special
            // cases like shadow maps. Using this requires enabling a GPU feature.
            .depth_clamp_enable(false)
            // If rasterizer_discard_enable is set to true, then geometry never passes through the
            // rasterizer stage. This basically disables any output to the framebuffer.
            .rasterizer_discard_enable(false)
            // Using any mode other than fill requires enabling a GPU feature.
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            // 光栅化器可以通过添加一个常数值或根据片段的斜率偏置它们来改变深度值。这有时用于阴影映射，但我们不会使用它。
            .depth_bias_enable(false);

        let multisample_state_create_info = vk::PipelineMultisampleStateCreateInfo::builder()
            // .sample_shading_enable(true)
            // .min_sample_shading(0.2)
            // .rasterization_samples(msaa_samples)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false);

        // let stencil_state = vk::StencilOpState {
        //     fail_op: vk::StencilOp::KEEP,
        //     pass_op: vk::StencilOp::KEEP,
        //     depth_fail_op: vk::StencilOp::KEEP,
        //     compare_op: vk::CompareOp::ALWAYS,
        //     compare_mask: 0,
        //     write_mask: 0,
        //     reference: 0,
        // };

        // let depth_stencil_state_create_info = vk::PipelineDepthStencilStateCreateInfo::builder()
        //     // depth_test_enable 字段指定是否应将新片段的深度与深度缓冲区进行比较，看它们是否应被丢弃。
        //     .depth_test_enable(true)
        //     // depth_write_enable 字段指定是否应将通过深度测试的新片段的深度实际写入深度缓冲区。
        //     .depth_write_enable(true)
        //     // depth_compare_op 字段指定了为保留或丢弃片段所进行的比较。我们坚持较低的深度 = 较近的惯例，所以新片段的深度应该较小。
        //     .depth_compare_op(vk::CompareOp::LESS)
        //     // depth_bounds_test_enable、min_depth_bounds 和 max_depth_bounds 字段用于可选的深度边界测试。
        //     // 基本上，这允许你只保留落在指定深度范围内的片段。我们将不会使用这个功能。
        //     .depth_bounds_test_enable(false)
        //     .min_depth_bounds(0.0) // Optional.
        //     .max_depth_bounds(1.0) // Optional.
        //     // 最后三个字段配置了模板缓冲区的操作，
        //     // 如果你想使用这些操作，那么你必须确保深度 / 模板图像的格式包含一个模板组件。
        //     .stencil_test_enable(false)
        //     // .front(/* vk::StencilOpState */) // Optional.
        //     // .back(/* vk::StencilOpState */); // Optional.
        //     .build();

        // pseudocode:
        // if blend_enable {
        //     final_color.rgb = (src_color_blend_factor * new_color.rgb)
        //         <color_blend_op> (dst_color_blend_factor * old_color.rgb);
        //     final_color.a = (src_alpha_blend_factor * new_color.a)
        //         <alpha_blend_op> (dst_alpha_blend_factor * old_color.a);
        // } else {
        //     final_color = new_color;
        // }
        //
        // final_color = final_color & color_write_mask;

        let color_blend_attachment_state = vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(false)
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .build();

        let color_blend_attachment_states = &[color_blend_attachment_state];
        let color_blend_state_create_info = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(color_blend_attachment_states)
            .blend_constants([0.0, 0.0, 0.0, 0.0]);

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state_create_info =
            vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_states);

        let graphic_pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_state_create_info)
            .input_assembly_state(&vertex_input_assembly_state_info)
            .viewport_state(&viewport_state_create_info)
            .rasterization_state(&rasterization_state_create_info)
            .multisample_state(&multisample_state_create_info)
            // .depth_stencil_state(&depth_stencil_state_create_info)
            .color_blend_state(&color_blend_state_create_info)
            .dynamic_state(&dynamic_state_create_info)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0)
            .build();

        let graphic_pipeline_create_infos = [graphic_pipeline_create_info];
        let pipelines = device.create_graphics_pipelines(&graphic_pipeline_create_infos)?;
        log::debug!("Vulkan pipelines created.");
        Ok(pipelines)
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        self.device.destroy_pipeline(self.raw);
        log::debug!("Pipeline destroyed.");
    }
}
