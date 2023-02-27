use crate::types_v2::{
    RHIAttachmentDescriptionFlags, RHIAttachmentReference, RHICommandPoolCreateFlags,
    RHIDescriptorSetLayoutCreateFlags, RHIFramebufferCreateFlags, RHIPipelineBindPoint,
    RHIPipelineLayoutCreateFlags, RHIRenderPassCreateFlags, RHISubpassDescriptionFlags,
};
use ash::vk;
use num_traits::{FromPrimitive, ToPrimitive};
use rhi_types::{
    RHIAccessFlags, RHIAttachmentLoadOp, RHIAttachmentStoreOp, RHIDescriptorType, RHIFormat,
    RHIImageLayout, RHIPipelineStageFlags, RHISampleCountFlagBits, RHIShaderStageFlags,
};

pub fn map_command_pool_create_flags(
    value: RHICommandPoolCreateFlags,
) -> vk::CommandPoolCreateFlags {
    let mut flags = vk::CommandPoolCreateFlags::empty();
    if value.contains(RHICommandPoolCreateFlags::TRANSIENT) {
        flags |= vk::CommandPoolCreateFlags::TRANSIENT;
    }
    if value.contains(RHICommandPoolCreateFlags::RESET_COMMAND_BUFFER) {
        flags |= vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER;
    }
    flags
}

pub fn map_vk_format(value: vk::Format) -> RHIFormat {
    match RHIFormat::from_i32(value.as_raw()) {
        None => RHIFormat::UNDEFINED,
        Some(x) => x,
    }
}

pub fn map_format(value: RHIFormat) -> vk::Format {
    match value.to_i32() {
        None => vk::Format::UNDEFINED,
        Some(x) => vk::Format::from_raw(x),
    }
}

pub fn map_attachment_description_flags(
    value: RHIAttachmentDescriptionFlags,
) -> vk::AttachmentDescriptionFlags {
    let mut flags = vk::AttachmentDescriptionFlags::empty();
    if value.contains(RHIAttachmentDescriptionFlags::MAY_ALIAS) {
        flags |= vk::AttachmentDescriptionFlags::MAY_ALIAS;
    }
    flags
}

pub fn map_subpass_description_flags(
    value: RHISubpassDescriptionFlags,
) -> vk::SubpassDescriptionFlags {
    let mut flags = vk::SubpassDescriptionFlags::empty();
    flags
}

pub fn map_sample_count_flag_bits(value: RHISampleCountFlagBits) -> vk::SampleCountFlags {
    match value.to_u32() {
        None => vk::SampleCountFlags::TYPE_1,
        Some(x) => vk::SampleCountFlags::from_raw(x),
    }
}

pub fn map_attachment_load_op(value: RHIAttachmentLoadOp) -> vk::AttachmentLoadOp {
    match value.to_i32() {
        None => vk::AttachmentLoadOp::DONT_CARE,
        Some(x) => vk::AttachmentLoadOp::from_raw(x),
    }
}

pub fn map_attachment_store_op(value: RHIAttachmentStoreOp) -> vk::AttachmentStoreOp {
    match value.to_i32() {
        None => vk::AttachmentStoreOp::DONT_CARE,
        Some(x) => vk::AttachmentStoreOp::from_raw(x),
    }
}

pub fn map_image_layout(value: RHIImageLayout) -> vk::ImageLayout {
    match value.to_i32() {
        None => vk::ImageLayout::UNDEFINED,
        Some(x) => vk::ImageLayout::from_raw(x),
    }
}

pub fn map_pipeline_bind_point(value: RHIPipelineBindPoint) -> vk::PipelineBindPoint {
    match value.to_i32() {
        None => vk::PipelineBindPoint::GRAPHICS,
        Some(x) => vk::PipelineBindPoint::from_raw(x),
    }
}

pub fn map_attachment_reference(value: &RHIAttachmentReference) -> vk::AttachmentReference {
    vk::AttachmentReference::builder()
        .attachment(value.attachment)
        .layout(map_image_layout(value.layout))
        .build()
}

pub fn map_pipeline_stage_flags(value: RHIPipelineStageFlags) -> vk::PipelineStageFlags {
    let mut flags = vk::PipelineStageFlags::empty();
    if value.contains(RHIPipelineStageFlags::TOP_OF_PIPE) {
        flags |= vk::PipelineStageFlags::TOP_OF_PIPE;
    }
    if value.contains(RHIPipelineStageFlags::DRAW_INDIRECT) {
        flags |= vk::PipelineStageFlags::DRAW_INDIRECT;
    }
    if value.contains(RHIPipelineStageFlags::VERTEX_INPUT) {
        flags |= vk::PipelineStageFlags::VERTEX_INPUT;
    }
    if value.contains(RHIPipelineStageFlags::VERTEX_SHADER) {
        flags |= vk::PipelineStageFlags::VERTEX_SHADER;
    }
    if value.contains(RHIPipelineStageFlags::TESSELLATION_CONTROL_SHADER) {
        flags |= vk::PipelineStageFlags::TESSELLATION_CONTROL_SHADER;
    }
    if value.contains(RHIPipelineStageFlags::TESSELLATION_EVALUATION_SHADER) {
        flags |= vk::PipelineStageFlags::TESSELLATION_EVALUATION_SHADER;
    }
    if value.contains(RHIPipelineStageFlags::GEOMETRY_SHADER) {
        flags |= vk::PipelineStageFlags::GEOMETRY_SHADER;
    }
    if value.contains(RHIPipelineStageFlags::FRAGMENT_SHADER) {
        flags |= vk::PipelineStageFlags::FRAGMENT_SHADER;
    }
    if value.contains(RHIPipelineStageFlags::EARLY_FRAGMENT_TESTS) {
        flags |= vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS;
    }
    if value.contains(RHIPipelineStageFlags::LATE_FRAGMENT_TESTS) {
        flags |= vk::PipelineStageFlags::LATE_FRAGMENT_TESTS;
    }
    if value.contains(RHIPipelineStageFlags::COLOR_ATTACHMENT_OUTPUT) {
        flags |= vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;
    }
    if value.contains(RHIPipelineStageFlags::COMPUTE_SHADER) {
        flags |= vk::PipelineStageFlags::COMPUTE_SHADER;
    }
    if value.contains(RHIPipelineStageFlags::TRANSFER) {
        flags |= vk::PipelineStageFlags::TRANSFER;
    }
    if value.contains(RHIPipelineStageFlags::BOTTOM_OF_PIPE) {
        flags |= vk::PipelineStageFlags::BOTTOM_OF_PIPE;
    }
    if value.contains(RHIPipelineStageFlags::HOST) {
        flags |= vk::PipelineStageFlags::HOST;
    }
    if value.contains(RHIPipelineStageFlags::ALL_GRAPHICS) {
        flags |= vk::PipelineStageFlags::ALL_GRAPHICS;
    }
    if value.contains(RHIPipelineStageFlags::ALL_COMMANDS) {
        flags |= vk::PipelineStageFlags::ALL_COMMANDS;
    }
    flags
}

pub fn map_access_flags(value: RHIAccessFlags) -> vk::AccessFlags {
    let mut flags = vk::AccessFlags::empty();
    if value.contains(RHIAccessFlags::INDIRECT_COMMAND_READ) {
        flags |= vk::AccessFlags::INDIRECT_COMMAND_READ;
    }
    if value.contains(RHIAccessFlags::INDEX_READ) {
        flags |= vk::AccessFlags::INDEX_READ;
    }
    if value.contains(RHIAccessFlags::VERTEX_ATTRIBUTE_READ) {
        flags |= vk::AccessFlags::VERTEX_ATTRIBUTE_READ;
    }
    if value.contains(RHIAccessFlags::UNIFORM_READ) {
        flags |= vk::AccessFlags::UNIFORM_READ;
    }
    if value.contains(RHIAccessFlags::INPUT_ATTACHMENT_READ) {
        flags |= vk::AccessFlags::INPUT_ATTACHMENT_READ;
    }
    if value.contains(RHIAccessFlags::SHADER_READ) {
        flags |= vk::AccessFlags::SHADER_READ;
    }
    if value.contains(RHIAccessFlags::SHADER_WRITE) {
        flags |= vk::AccessFlags::SHADER_WRITE;
    }
    if value.contains(RHIAccessFlags::COLOR_ATTACHMENT_READ) {
        flags |= vk::AccessFlags::COLOR_ATTACHMENT_READ;
    }
    if value.contains(RHIAccessFlags::COLOR_ATTACHMENT_WRITE) {
        flags |= vk::AccessFlags::COLOR_ATTACHMENT_WRITE;
    }
    if value.contains(RHIAccessFlags::DEPTH_STENCIL_ATTACHMENT_READ) {
        flags |= vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ;
    }
    if value.contains(RHIAccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE) {
        flags |= vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE;
    }
    if value.contains(RHIAccessFlags::TRANSFER_READ) {
        flags |= vk::AccessFlags::TRANSFER_READ;
    }
    if value.contains(RHIAccessFlags::TRANSFER_WRITE) {
        flags |= vk::AccessFlags::TRANSFER_WRITE;
    }
    if value.contains(RHIAccessFlags::HOST_READ) {
        flags |= vk::AccessFlags::HOST_READ;
    }
    if value.contains(RHIAccessFlags::HOST_WRITE) {
        flags |= vk::AccessFlags::HOST_WRITE;
    }
    if value.contains(RHIAccessFlags::MEMORY_READ) {
        flags |= vk::AccessFlags::MEMORY_READ;
    }
    if value.contains(RHIAccessFlags::MEMORY_WRITE) {
        flags |= vk::AccessFlags::MEMORY_WRITE;
    }
    flags
}

pub fn map_render_pass_create_flags(value: RHIRenderPassCreateFlags) -> vk::RenderPassCreateFlags {
    let mut flags = vk::RenderPassCreateFlags::empty();
    flags
}

pub fn map_pipeline_layout_create_flags(
    value: RHIPipelineLayoutCreateFlags,
) -> vk::PipelineLayoutCreateFlags {
    let mut flags = vk::PipelineLayoutCreateFlags::empty();
    flags
}

pub fn map_shader_stage_flags(value: RHIShaderStageFlags) -> vk::ShaderStageFlags {
    let mut flags = vk::ShaderStageFlags::empty();
    if value.contains(RHIShaderStageFlags::VERTEX) {
        flags |= vk::ShaderStageFlags::VERTEX;
    }
    if value.contains(RHIShaderStageFlags::TESSELLATION_CONTROL) {
        flags |= vk::ShaderStageFlags::TESSELLATION_CONTROL;
    }
    if value.contains(RHIShaderStageFlags::TESSELLATION_EVALUATION) {
        flags |= vk::ShaderStageFlags::TESSELLATION_EVALUATION;
    }
    if value.contains(RHIShaderStageFlags::GEOMETRY) {
        flags |= vk::ShaderStageFlags::GEOMETRY;
    }
    if value.contains(RHIShaderStageFlags::FRAGMENT) {
        flags |= vk::ShaderStageFlags::FRAGMENT;
    }
    if value.contains(RHIShaderStageFlags::COMPUTE) {
        flags |= vk::ShaderStageFlags::COMPUTE;
    }
    flags
}

pub fn map_framebuffer_create_flags(
    value: RHIFramebufferCreateFlags,
) -> vk::FramebufferCreateFlags {
    let flags = vk::FramebufferCreateFlags::empty();
    // if value.contains(RHIFramebufferCreateFlags::IMAGELESS) {
    //     flags |= vk::FramebufferCreateFlags::IMAGELESS;
    // }
    flags
}

pub fn map_descriptor_set_layout_create_flags(
    value: RHIDescriptorSetLayoutCreateFlags,
) -> vk::DescriptorSetLayoutCreateFlags {
    let flags = vk::DescriptorSetLayoutCreateFlags::empty();
    flags
}

pub fn map_descriptor_type(value: RHIDescriptorType) -> vk::DescriptorType {
    match value.to_i32() {
        None => vk::DescriptorType::SAMPLER,
        Some(x) => vk::DescriptorType::from_raw(x),
    }
}
