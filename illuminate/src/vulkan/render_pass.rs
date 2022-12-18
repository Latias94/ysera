use crate::vulkan::command_buffer::CommandBuffer;
use crate::vulkan::conv;
use crate::vulkan::device::Device;
use crate::vulkan::render_pass::RenderPassState::InRenderPass;
use crate::{Color, DeviceError};
use ash::vk;
use std::rc::Rc;

pub struct RenderPass {
    raw: vk::RenderPass,
    device: Rc<Device>,
    state: RenderPassState,
    render_area: math::Rect2D,
    clear_color: Color,
}

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

impl RenderPass {
    pub fn raw(&self) -> vk::RenderPass {
        self.raw
    }

    pub fn new(
        device: &Rc<Device>,
        surface_format: vk::Format,
        depth_format: vk::Format,
        render_area: math::Rect2D,
        clear_color: Color,
    ) -> Result<Self, DeviceError> {
        profiling::scope!("create_render_pass");

        // todo configurable
        let color_attachment = vk::AttachmentDescription::builder()
            .format(surface_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            // initial_layout 指定在渲染通道开始之前图像将具有的布局。 final_layout 指定渲染过程完成时自动转换到的布局。
            // 对 initial_layout 使用 vk::ImageLayout::UNDEFINED 意味着我们不关心图像之前的布局。
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .build();
        let color_attachment_ref = vk::AttachmentReference::builder()
            .attachment(0)
            // 布局指定我们希望附件在使用此引用的子通道中具有的布局。当子通道启动时，Vulkan 会自动将附件过渡到这个布局
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build();

        // 我们把 finalLayout 从 vk::ImageLayout::PRESENT_SRC_KHR 改为 vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL。
        // 这是因为多采样图像不能直接呈现。我们首先需要将它们解析为普通图像。这个要求并不适用于深度缓冲区，
        // 因为它不会在任何时候被呈现。因此，我们只需要为颜色添加一个新的附件，这是一个 resolve attachment。
        let depth_stencil_attachment = vk::AttachmentDescription::builder()
            .format(depth_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .build();
        let depth_stencil_attachment_ref = vk::AttachmentReference::builder()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .build();

        // todo other attachment type
        // let color_resolve_attachment = vk::AttachmentDescription::builder()
        //     .format(surface_format)
        //     .samples(vk::SampleCountFlags::TYPE_1)
        //     .load_op(vk::AttachmentLoadOp::DONT_CARE)
        //     .store_op(vk::AttachmentStoreOp::STORE)
        //     .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        //     .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        //     .initial_layout(vk::ImageLayout::UNDEFINED)
        //     .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
        //     .build();

        // 现在必须指示渲染通道将多采样的彩色图像解析为普通附件。创建一个新的附件引用，它将指向颜色缓冲区，作为解析目标。
        // let color_resolve_attachment_ref = vk::AttachmentReference::builder()
        //     .attachment(2)
        //     .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        //     .build();

        let color_attachments = [color_attachment_ref];
        let subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachments)
            // .depth_stencil_attachment(&depth_stencil_attachment_ref)
            // Input from shader
            // .input_attachments()
            // multi-sampling
            // .resolve_attachments()
            .build();

        let dependency = vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .build();

        let attachments = &[
            color_attachment,
            // depth_stencil_attachment,
            // color_resolve_attachment,
        ];

        // dont's do the `.subpasses(&[subpass])` + `build()` will cause the temporary array pointer
        //  live shorter before the vulkan call  https://github.com/ash-rs/ash/issues/158
        let subpasses = [subpass];
        let dependencies = [dependency];
        let create_info = vk::RenderPassCreateInfo::builder()
            .subpasses(&subpasses)
            .attachments(attachments)
            .dependencies(&dependencies);
        let raw = device.create_render_pass(&create_info)?;

        Ok(Self {
            raw,
            device: device.clone(),
            state: InRenderPass,
            render_area,
            clear_color,
        })
    }

    pub fn begin(&self, command_buffer: &CommandBuffer, framebuffer: vk::Framebuffer) {
        let clear_values = [conv::convert_clear_color(self.clear_color)];
        let begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.raw)
            .framebuffer(framebuffer)
            .render_area(conv::convert_rect2d(self.render_area))
            .clear_values(&clear_values)
            .build();
        self.device.cmd_begin_render_pass(
            command_buffer.raw(),
            &begin_info,
            vk::SubpassContents::INLINE,
        );
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        self.device.destroy_render_pass(self.raw);
        log::debug!("Render Pass destroyed.");
    }
}
