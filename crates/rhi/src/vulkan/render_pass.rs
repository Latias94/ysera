use std::sync::Arc;

use ash::vk;
use typed_builder::TypedBuilder;

use crate::types::{
    RenderPassClearFlags, RenderPassDescriptor, RenderTargetAttachment, RenderTargetAttachmentType,
};
use crate::vulkan::command_buffer::CommandBuffer;
use crate::vulkan::conv;
use crate::vulkan::device::Device;
use crate::vulkan::render_pass::RenderPassState::{InRenderPass, Recording};
use crate::DeviceError;

pub struct RenderPass {
    raw: vk::RenderPass,
    device: Arc<Device>,
    state: RenderPassState,
    depth: f32,
    stencil: u32,

    render_area: math::Rect2D,
    clear_values: Vec<vk::ClearValue>,
    clear_flags: RenderPassClearFlags,
    targets: Vec<RenderTargetAttachment>,
}

// pub struct RenderTarget {
//     pub attachments: Vec<RenderTargetAttachment>,
//     // pub framebuffer: Framebuffer,
// }

pub enum RenderPassState {
    /// ready to begin
    Ready,
    /// command buffer
    Recording,
    /// render pass started
    InRenderPass,
    /// render pass ended
    RecordingEnded,
    /// render pass submitted
    Submitted,
    NotAllocated,
}

#[derive(Clone, TypedBuilder)]
pub struct ImguiRenderPassDescriptor<'a> {
    pub device: &'a Arc<Device>,
    pub render_area: math::Rect2D,
    pub surface_format: vk::Format,
}

impl RenderPass {
    pub fn raw(&self) -> vk::RenderPass {
        self.raw
    }

    pub unsafe fn new(
        device: &Arc<Device>,
        desc: &RenderPassDescriptor,
    ) -> Result<Self, DeviceError> {
        profiling::scope!("create_render_pass");

        let mut attachment_descriptions = vec![];
        let mut color_attachment_refs = vec![];
        let mut depth_attachment_ref = None;
        let mut clear_values = Vec::with_capacity(desc.attachments.len());

        for (i, attachment) in desc.attachments.iter().enumerate() {
            let mut attachment_description = vk::AttachmentDescription::builder()
                .format(attachment.format.to_vk())
                // .samples(desc.max_msaa_samples)
                .load_op(attachment.load_op.to_vk())
                .store_op(attachment.store_op.to_vk())
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE);

            let mut attachment_ref = vk::AttachmentReference::builder().attachment(i as u32);
            if attachment.ty == RenderTargetAttachmentType::Depth {
                attachment_description =
                    // initial_layout 指定在渲染通道开始之前图像将具有的布局。 final_layout 指定渲染过程完成时自动转换到的布局。
                    // 对 initial_layout 使用 vk::ImageLayout::UNDEFINED 意味着我们不关心图像之前的布局。
                    attachment_description
                        .initial_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                        // 多采样图像不能直接显示。我们首先需要将它们解析为常规图像。此要求不适用于深度缓冲区，因为它不会在任何时候显示。
                        .final_layout(vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL);
                attachment_ref =
                    attachment_ref.layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
                clear_values.insert(
                    i,
                    conv::convert_clear_depth_stencil(desc.depth, desc.stencil),
                );
                depth_attachment_ref = Some(attachment_ref.build());
            } else {
                // RenderTargetAttachmentType::Color
                // todo Stencil not support yet
                attachment_description = attachment_description
                    .initial_layout(vk::ImageLayout::UNDEFINED)
                    .final_layout(vk::ImageLayout::READ_ONLY_OPTIMAL);
                attachment_ref = attachment_ref.layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
                clear_values.insert(i, conv::convert_clear_color(desc.clear_color));
                color_attachment_refs.push(attachment_ref.build());
            };
            attachment_descriptions.push(attachment_description.build());
        }

        // .depth_stencil_attachment(&depth_attachment_ref)
        // multi-sampling
        // .resolve_attachments(&color_resolve_attachments)
        // Input from shader
        // .input_attachments()
        // .build();

        let subpasses = [{
            let mut subpass = vk::SubpassDescription::builder()
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .color_attachments(&color_attachment_refs);

            if let Some(ref reference) = depth_attachment_ref {
                subpass = subpass.depth_stencil_attachment(reference);
            }
            subpass.build()
        }];

        let has_color_attachment = !color_attachment_refs.is_empty();
        let has_depth_attachment = depth_attachment_ref.is_some();
        let dependencies = Self::get_dependency(has_color_attachment, has_depth_attachment);

        // don't do the `.subpasses(&[subpass])` + `build()` will cause the temporary array pointer
        let create_info = vk::RenderPassCreateInfo::builder()
            .subpasses(&subpasses)
            .attachments(&attachment_descriptions)
            .dependencies(&dependencies);
        let raw = unsafe { device.raw().create_render_pass(&create_info, None)? };

        if let Some(label) = desc.label {
            unsafe { device.set_object_name(vk::ObjectType::RENDER_PASS, raw, label) };
        }

        Ok(Self {
            raw,
            device: device.clone(),
            state: InRenderPass,
            depth: desc.depth,
            stencil: desc.stencil,
            render_area: desc.render_area,
            clear_values,
            clear_flags: desc.clear_flags,
            targets: desc.attachments.to_vec(),
        })
    }

    unsafe fn get_dependency(
        has_color_attachment: bool,
        has_depth_attachment: bool,
    ) -> Vec<vk::SubpassDependency> {
        let mut dependencies = vec![];
        if has_color_attachment {
            let dep1 = vk::SubpassDependency::builder()
                .src_subpass(vk::SUBPASS_EXTERNAL)
                .dst_subpass(0)
                .src_stage_mask(vk::PipelineStageFlags::FRAGMENT_SHADER)
                .src_access_mask(vk::AccessFlags::SHADER_READ)
                .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                .build();
            dependencies.push(dep1);
            let dep2 = vk::SubpassDependency::builder()
                .src_subpass(0)
                .dst_subpass(vk::SUBPASS_EXTERNAL)
                .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                .dst_stage_mask(vk::PipelineStageFlags::FRAGMENT_SHADER)
                .dst_access_mask(vk::AccessFlags::SHADER_READ)
                .build();
            dependencies.push(dep2);
        }

        if has_depth_attachment {
            let dep1 = vk::SubpassDependency::builder()
                .src_subpass(vk::SUBPASS_EXTERNAL)
                .dst_subpass(0)
                .src_stage_mask(vk::PipelineStageFlags::FRAGMENT_SHADER)
                .src_access_mask(vk::AccessFlags::SHADER_READ)
                .dst_stage_mask(vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
                .dst_access_mask(vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE)
                .build();
            dependencies.push(dep1);
            let dep2 = vk::SubpassDependency::builder()
                .src_subpass(0)
                .dst_subpass(vk::SUBPASS_EXTERNAL)
                .src_stage_mask(vk::PipelineStageFlags::LATE_FRAGMENT_TESTS)
                .src_access_mask(vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE)
                .dst_stage_mask(vk::PipelineStageFlags::FRAGMENT_SHADER)
                .dst_access_mask(vk::AccessFlags::SHADER_READ)
                .build();
            dependencies.push(dep2);
        }

        dependencies
    }

    // pub unsafe fn new_imgui_render_pass(
    //     desc: &ImguiRenderPassDescriptor,
    // ) -> Result<Self, DeviceError> {
    //     profiling::scope!("create_render_pass imgui");
    //
    //     log::debug!("Creating imgui render pass!");
    //     let attachment_descs = [vk::AttachmentDescription::builder()
    //         .format(desc.surface_format)
    //         .samples(vk::SampleCountFlags::TYPE_1)
    //         .load_op(vk::AttachmentLoadOp::LOAD)
    //         .store_op(vk::AttachmentStoreOp::STORE)
    //         .initial_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
    //         .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
    //         .build()];
    //
    //     let color_attachment_refs = [vk::AttachmentReference::builder()
    //         .attachment(0)
    //         .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
    //         .build()];
    //
    //     let subpass_descs = [vk::SubpassDescription::builder()
    //         .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
    //         .color_attachments(&color_attachment_refs)
    //         .build()];
    //
    //     let subpass_deps = [vk::SubpassDependency::builder()
    //         .src_subpass(vk::SUBPASS_EXTERNAL)
    //         .dst_subpass(0)
    //         .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
    //         .src_access_mask(vk::AccessFlags::empty())
    //         .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
    //         .dst_access_mask(
    //             vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
    //         )
    //         .build()];
    //
    //     let render_pass_info = vk::RenderPassCreateInfo::builder()
    //         .attachments(&attachment_descs)
    //         .subpasses(&subpass_descs)
    //         .dependencies(&subpass_deps);
    //
    //     let raw = unsafe {
    //         desc.device
    //             .raw()
    //             .create_render_pass(&render_pass_info, None)?
    //     };
    //     Ok(Self {
    //         raw,
    //         device: desc.device.clone(),
    //         state: InRenderPass,
    //         clear_values: vec![vk::ClearValue {
    //             color: vk::ClearColorValue {
    //                 float32: [1.0, 1.0, 1.0, 1.0],
    //             },
    //         }],
    //     })
    // }

    pub unsafe fn begin(&mut self, command_buffer: &CommandBuffer, framebuffer: vk::Framebuffer) {
        let begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.raw)
            .framebuffer(framebuffer)
            .render_area(conv::convert_rect2d(self.render_area))
            .clear_values(&self.clear_values)
            .build();
        unsafe {
            self.device.raw().cmd_begin_render_pass(
                command_buffer.raw(),
                &begin_info,
                vk::SubpassContents::INLINE,
            );
        }
        self.state = InRenderPass;
    }

    pub unsafe fn end(&mut self, command_buffer: &CommandBuffer) {
        unsafe {
            self.device.raw().cmd_end_render_pass(command_buffer.raw());
        }
        self.state = Recording;
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        unsafe {
            self.device.raw().destroy_render_pass(self.raw, None);
        }
        log::debug!("Render Pass destroyed.");
    }
}
