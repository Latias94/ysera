use crate::vulkan::command_buffer::CommandBuffer;
use crate::vulkan::conv;
use crate::vulkan::device::Device;
use crate::vulkan::render_pass::RenderPassState::{InRenderPass, Recording};
use crate::{Color, DeviceError};
use ash::vk;
use std::rc::Rc;
use typed_builder::TypedBuilder;

pub struct RenderPass {
    raw: vk::RenderPass,
    device: Rc<Device>,
    state: RenderPassState,
    render_area: math::Rect2D,
    clear_values: Vec<vk::ClearValue>,
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

#[derive(Clone, TypedBuilder)]
pub struct RenderPassDescriptor<'a> {
    pub device: &'a Rc<Device>,
    pub surface_format: vk::Format,
    pub depth_format: vk::Format,
    pub render_area: math::Rect2D,
    pub clear_color: Color,
    pub max_msaa_samples: vk::SampleCountFlags,
    pub depth: f32,
    pub stencil: u32,
}

#[derive(Clone, TypedBuilder)]
pub struct ImguiRenderPassDescriptor<'a> {
    pub device: &'a Rc<Device>,
    pub render_area: math::Rect2D,
    pub surface_format: vk::Format,
}

impl RenderPass {
    pub fn raw(&self) -> vk::RenderPass {
        self.raw
    }

    pub fn new(desc: &RenderPassDescriptor) -> Result<Self, DeviceError> {
        profiling::scope!("create_render_pass");

        // todo configurable
        let color_attachment = vk::AttachmentDescription::builder()
            .format(desc.surface_format)
            .samples(desc.max_msaa_samples)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            // initial_layout 指定在渲染通道开始之前图像将具有的布局。 final_layout 指定渲染过程完成时自动转换到的布局。
            // 对 initial_layout 使用 vk::ImageLayout::UNDEFINED 意味着我们不关心图像之前的布局。
            .initial_layout(vk::ImageLayout::UNDEFINED)
            // 多采样图像不能直接显示。我们首先需要将它们解析为常规图像。此要求不适用于深度缓冲区，因为它不会在任何时候显示。
            .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL) // msaa
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
            .format(desc.depth_format)
            .samples(desc.max_msaa_samples)
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

        // multisampling
        let color_resolve_attachment = vk::AttachmentDescription::builder()
            .format(desc.surface_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::DONT_CARE)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build();

        // 现在必须指示渲染通道将多采样的彩色图像解析为普通附件。创建一个新的附件引用，它将指向颜色缓冲区，作为解析目标。
        let color_resolve_attachment_ref = vk::AttachmentReference::builder()
            .attachment(2)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build();

        let color_attachments = [color_attachment_ref];
        let color_resolve_attachments = [color_resolve_attachment_ref];
        let subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachments)
            .depth_stencil_attachment(&depth_stencil_attachment_ref)
            // multi-sampling
            .resolve_attachments(&color_resolve_attachments)
            // Input from shader
            // .input_attachments()
            .build();

        let dependency = vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            )
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            )
            .dst_access_mask(
                vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            )
            .build();

        let attachments = &[
            color_attachment,
            depth_stencil_attachment,
            color_resolve_attachment,
        ];

        // don't do the `.subpasses(&[subpass])` + `build()` will cause the temporary array pointer
        // live shorter before the vulkan call  https://github.com/ash-rs/ash/issues/158
        let subpasses = [subpass];
        let dependencies = [dependency];
        let create_info = vk::RenderPassCreateInfo::builder()
            .subpasses(&subpasses)
            .attachments(attachments)
            .dependencies(&dependencies);
        let raw = desc.device.create_render_pass(&create_info)?;
        let clear_values = vec![
            conv::convert_clear_color(desc.clear_color),
            conv::convert_clear_depth_stencil(desc.depth, desc.stencil),
        ];
        Ok(Self {
            raw,
            device: desc.device.clone(),
            state: InRenderPass,
            render_area: desc.render_area,
            clear_values,
        })
    }

    pub fn new_imgui_render_pass(desc: &ImguiRenderPassDescriptor) -> Result<Self, DeviceError> {
        profiling::scope!("create_render_pass imgui");

        log::debug!("Creating imgui render pass!");
        let attachment_descs = [vk::AttachmentDescription::builder()
            .format(desc.surface_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::LOAD)
            .store_op(vk::AttachmentStoreOp::STORE)
            .initial_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .build()];

        let color_attachment_refs = [vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build()];

        let subpass_descs = [vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachment_refs)
            .build()];

        let subpass_deps = [vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(
                vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            )
            .build()];

        let render_pass_info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachment_descs)
            .subpasses(&subpass_descs)
            .dependencies(&subpass_deps);

        let raw = desc.device.create_render_pass(&render_pass_info)?;
        Ok(Self {
            raw,
            device: desc.device.clone(),
            state: InRenderPass,
            render_area: desc.render_area,
            clear_values: vec![vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [1.0, 1.0, 1.0, 1.0],
                },
            }],
        })
    }

    pub fn begin(&mut self, command_buffer: &CommandBuffer, framebuffer: vk::Framebuffer) {
        let begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.raw)
            .framebuffer(framebuffer)
            .render_area(conv::convert_rect2d(self.render_area))
            .clear_values(&self.clear_values)
            .build();
        self.device.cmd_begin_render_pass(
            command_buffer.raw(),
            &begin_info,
            vk::SubpassContents::INLINE,
        );
        self.state = InRenderPass;
    }
    pub fn end(&mut self, command_buffer: &CommandBuffer) {
        self.device.cmd_end_render_pass(command_buffer.raw());
        self.state = Recording;
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        self.device.destroy_render_pass(self.raw);
        log::debug!("Render Pass destroyed.");
    }
}
