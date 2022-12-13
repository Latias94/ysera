use crate::vulkan::device::Device;
use crate::DeviceError;
use ash::vk;
use std::rc::Rc;

pub struct RenderPass {
    raw: vk::RenderPass,
    device: Rc<Device>,
}

impl RenderPass {
    pub fn raw(&self) -> vk::RenderPass {
        self.raw
    }

    pub fn new(
        device: &Rc<Device>,
        surface_format: vk::Format, /* depth_format: vk::Format */
    ) -> Result<Self, DeviceError> {
        profiling::scope!("create_render_pass");

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

        // 我们把 finalLayout 从 vk::ImageLayout::PRESENT_SRC_KHR 改为 vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL。
        // 这是因为多采样图像不能直接呈现。我们首先需要将它们解析为普通图像。这个要求并不适用于深度缓冲区，
        // 因为它不会在任何时候被呈现。因此，我们只需要为颜色添加一个新的附件，这是一个 resolve attachment。
        // let depth_stencil_attachment = vk::AttachmentDescription::builder()
        //     .format(depth_format)
        //     .samples(msaa_samples)
        //     .load_op(vk::AttachmentLoadOp::CLEAR)
        //     .store_op(vk::AttachmentStoreOp::DONT_CARE)
        //     .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        //     .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        //     .initial_layout(vk::ImageLayout::UNDEFINED)
        //     .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
        //     .build();

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

        let color_attachment_ref = vk::AttachmentReference::builder()
            .attachment(0)
            // 布局指定我们希望附件在使用此引用的子通道中具有的布局。当子通道启动时，Vulkan 会自动将附件过渡到这个布局
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build();

        // let depth_stencil_attachment_ref = vk::AttachmentReference::builder()
        //     .attachment(1)
        //     .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
        //     .build();

        // 现在必须指示渲染通道将多采样的彩色图像解析为普通附件。创建一个新的附件引用，它将指向颜色缓冲区，作为解析目标。
        // let color_resolve_attachment_ref = vk::AttachmentReference::builder()
        //     .attachment(2)
        //     .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        //     .build();

        let subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&[color_attachment_ref])
            // .depth_stencil_attachment(&depth_stencil_attachment_ref)
            // .resolve_attachments(&[color_resolve_attachment_ref])
            .build();

        let attachments = &[
            color_attachment,
            // depth_stencil_attachment,
            // color_resolve_attachment,
        ];

        let create_info = vk::RenderPassCreateInfo::builder()
            .subpasses(&[subpass])
            .attachments(attachments)
            .build();

        let raw = device.create_render_pass(&create_info)?;

        Ok(Self {
            raw,
            device: device.clone(),
        })
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        self.device.destroy_render_pass(self.raw);
        log::debug!("Render Pass destroyed.");
    }
}
