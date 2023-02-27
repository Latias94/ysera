use crate::types_v2::{
    RHIAttachmentDescriptionFlags, RHIAttachmentReference, RHICommandPoolCreateFlags,
    RHIDescriptorSetLayoutCreateFlags, RHIFramebufferCreateFlags, RHIPipelineBindPoint,
    RHIPipelineCreateFlags, RHIPipelineDepthStencilStateCreateFlags,
    RHIPipelineDynamicStateCreateFlags, RHIPipelineInputAssemblyStateCreateFlags,
    RHIPipelineLayoutCreateFlags, RHIPipelineMultisampleStateCreateFlags,
    RHIPipelineRasterizationStateCreateFlags, RHIPipelineVertexInputStateCreateFlags,
    RHIPipelineViewportStateCreateFlags, RHIRenderPassCreateFlags, RHISampleMask,
    RHISubpassDescriptionFlags, RHIVertexInputRate,
};
use crate::vulkan_v2::conv;
use ash::vk;
use num_traits::{FromPrimitive, ToPrimitive};
use rhi_types::{
    RHIAccessFlags, RHIAttachmentLoadOp, RHIAttachmentStoreOp, RHIBlendFactor, RHIBlendOp,
    RHIColorComponentFlags, RHICompareOp, RHICullModeFlags, RHIDescriptorType, RHIDynamicState,
    RHIExtent2D, RHIExtent3D, RHIFormat, RHIFrontFace, RHIImageLayout, RHILogicOp, RHIOffset2D,
    RHIPipelineStageFlags, RHIPolygonMode, RHIPrimitiveTopology, RHIRect2D, RHISampleCountFlagBits,
    RHIShaderStageFlags, RHIStencilOp, RHIStencilOpState,
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
        None => vk::Format::default(),
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
        None => vk::SampleCountFlags::default(),
        Some(x) => vk::SampleCountFlags::from_raw(x),
    }
}

pub fn map_attachment_load_op(value: RHIAttachmentLoadOp) -> vk::AttachmentLoadOp {
    match value.to_i32() {
        None => vk::AttachmentLoadOp::default(),
        Some(x) => vk::AttachmentLoadOp::from_raw(x),
    }
}

pub fn map_attachment_store_op(value: RHIAttachmentStoreOp) -> vk::AttachmentStoreOp {
    match value.to_i32() {
        None => vk::AttachmentStoreOp::default(),
        Some(x) => vk::AttachmentStoreOp::from_raw(x),
    }
}

pub fn map_image_layout(value: RHIImageLayout) -> vk::ImageLayout {
    match value.to_i32() {
        None => vk::ImageLayout::default(),
        Some(x) => vk::ImageLayout::from_raw(x),
    }
}

pub fn map_pipeline_bind_point(value: RHIPipelineBindPoint) -> vk::PipelineBindPoint {
    match value.to_i32() {
        None => vk::PipelineBindPoint::default(),
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
    _value: RHIPipelineLayoutCreateFlags,
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
    _value: RHIFramebufferCreateFlags,
) -> vk::FramebufferCreateFlags {
    let flags = vk::FramebufferCreateFlags::empty();
    // if value.contains(RHIFramebufferCreateFlags::IMAGELESS) {
    //     flags |= vk::FramebufferCreateFlags::IMAGELESS;
    // }
    flags
}

pub fn map_descriptor_set_layout_create_flags(
    _value: RHIDescriptorSetLayoutCreateFlags,
) -> vk::DescriptorSetLayoutCreateFlags {
    let flags = vk::DescriptorSetLayoutCreateFlags::empty();
    flags
}

pub fn map_descriptor_type(value: RHIDescriptorType) -> vk::DescriptorType {
    match value.to_i32() {
        None => vk::DescriptorType::default(),
        Some(x) => vk::DescriptorType::from_raw(x),
    }
}

pub fn map_pipeline_create_flags(value: RHIPipelineCreateFlags) -> vk::PipelineCreateFlags {
    let mut flags = vk::PipelineCreateFlags::empty();
    if value.contains(RHIPipelineCreateFlags::DISABLE_OPTIMIZATION) {
        flags |= vk::PipelineCreateFlags::DISABLE_OPTIMIZATION;
    }
    if value.contains(RHIPipelineCreateFlags::ALLOW_DERIVATIVES) {
        flags |= vk::PipelineCreateFlags::ALLOW_DERIVATIVES;
    }
    if value.contains(RHIPipelineCreateFlags::DERIVATIVE) {
        flags |= vk::PipelineCreateFlags::DERIVATIVE;
    }
    flags
}

pub fn map_vertex_input_rate(value: RHIVertexInputRate) -> vk::VertexInputRate {
    match value.to_i32() {
        None => vk::VertexInputRate::default(),
        Some(x) => vk::VertexInputRate::from_raw(x),
    }
}

pub fn map_primitive_topology(value: RHIPrimitiveTopology) -> vk::PrimitiveTopology {
    match value.to_i32() {
        None => vk::PrimitiveTopology::default(),
        Some(x) => vk::PrimitiveTopology::from_raw(x),
    }
}

pub fn map_pipeline_input_assembly_state_create_flags(
    _value: RHIPipelineInputAssemblyStateCreateFlags,
) -> vk::PipelineInputAssemblyStateCreateFlags {
    let flags = vk::PipelineInputAssemblyStateCreateFlags::empty();
    flags
}

pub fn map_pipeline_vertex_input_state_create_flags(
    _value: RHIPipelineVertexInputStateCreateFlags,
) -> vk::PipelineVertexInputStateCreateFlags {
    let flags = vk::PipelineVertexInputStateCreateFlags::empty();
    flags
}

pub fn map_pipeline_viewport_state_create_flags(
    _value: RHIPipelineViewportStateCreateFlags,
) -> vk::PipelineViewportStateCreateFlags {
    let flags = vk::PipelineViewportStateCreateFlags::empty();
    flags
}

pub fn map_rect_2d(value: RHIRect2D) -> vk::Rect2D {
    vk::Rect2D {
        offset: map_offset_2d(value.offset),
        extent: map_extent_2d(value.extent),
    }
}

pub fn map_offset_2d(value: RHIOffset2D) -> vk::Offset2D {
    vk::Offset2D {
        x: value.x,
        y: value.y,
    }
}

pub fn map_extent_2d(value: RHIExtent2D) -> vk::Extent2D {
    vk::Extent2D {
        width: value.width,
        height: value.height,
    }
}

pub fn map_extent_3d(value: RHIExtent3D) -> vk::Extent3D {
    vk::Extent3D {
        width: value.width,
        height: value.height,
        depth: value.depth,
    }
}

pub fn map_polygon_mode(value: RHIPolygonMode) -> vk::PolygonMode {
    match value.to_i32() {
        None => vk::PolygonMode::default(),
        Some(x) => vk::PolygonMode::from_raw(x),
    }
}

pub fn map_cull_mode_flags(value: RHICullModeFlags) -> vk::CullModeFlags {
    let mut flags = vk::CullModeFlags::empty();
    if value.contains(RHICullModeFlags::NONE) {
        flags |= vk::CullModeFlags::NONE;
    }
    if value.contains(RHICullModeFlags::FRONT) {
        flags |= vk::CullModeFlags::FRONT;
    }
    if value.contains(RHICullModeFlags::BACK) {
        flags |= vk::CullModeFlags::BACK;
    }
    flags
}

pub fn map_front_face(value: RHIFrontFace) -> vk::FrontFace {
    match value.to_i32() {
        None => vk::FrontFace::default(),
        Some(x) => vk::FrontFace::from_raw(x),
    }
}

pub fn map_pipeline_rasterization_state_create_flags(
    _value: RHIPipelineRasterizationStateCreateFlags,
) -> vk::PipelineRasterizationStateCreateFlags {
    let flags = vk::PipelineRasterizationStateCreateFlags::empty();
    flags
}

pub fn map_pipeline_multisample_state_create_flags(
    _value: RHIPipelineMultisampleStateCreateFlags,
) -> vk::PipelineMultisampleStateCreateFlags {
    let flags = vk::PipelineMultisampleStateCreateFlags::empty();
    flags
}

pub fn map_pipeline_depth_stencil_state_create_flags(
    _value: RHIPipelineDepthStencilStateCreateFlags,
) -> vk::PipelineDepthStencilStateCreateFlags {
    let flags = vk::PipelineDepthStencilStateCreateFlags::empty();
    flags
}

pub fn map_sample_mask(value: RHISampleMask) -> vk::SampleMask {
    value
}

pub fn map_compare_op(value: RHICompareOp) -> vk::CompareOp {
    match value.to_i32() {
        None => vk::CompareOp::default(),
        Some(x) => vk::CompareOp::from_raw(x),
    }
}

pub fn map_stencil_op_state(value: RHIStencilOpState) -> vk::StencilOpState {
    vk::StencilOpState {
        fail_op: map_stencil_op(value.fail_op),
        pass_op: map_stencil_op(value.pass_op),
        depth_fail_op: map_stencil_op(value.depth_fail_op),
        compare_op: map_compare_op(value.compare_op),
        compare_mask: value.compare_mask,
        write_mask: value.write_mask,
        reference: value.reference,
    }
}

pub fn map_stencil_op(value: RHIStencilOp) -> vk::StencilOp {
    match value.to_i32() {
        None => vk::StencilOp::default(),
        Some(x) => vk::StencilOp::from_raw(x),
    }
}

pub fn map_logic_op(value: RHILogicOp) -> vk::LogicOp {
    match value.to_i32() {
        None => vk::LogicOp::default(),
        Some(x) => vk::LogicOp::from_raw(x),
    }
}

pub fn map_blend_factor(value: RHIBlendFactor) -> vk::BlendFactor {
    match value.to_i32() {
        None => vk::BlendFactor::default(),
        Some(x) => vk::BlendFactor::from_raw(x),
    }
}

pub fn map_blend_op(value: RHIBlendOp) -> vk::BlendOp {
    match value.to_i32() {
        None => vk::BlendOp::default(),
        Some(x) => vk::BlendOp::from_raw(x),
    }
}

pub fn map_color_component_flags(value: RHIColorComponentFlags) -> vk::ColorComponentFlags {
    let mut flags = vk::ColorComponentFlags::empty();
    if value.contains(RHIColorComponentFlags::R) {
        flags |= vk::ColorComponentFlags::R;
    }
    if value.contains(RHIColorComponentFlags::G) {
        flags |= vk::ColorComponentFlags::G;
    }
    if value.contains(RHIColorComponentFlags::B) {
        flags |= vk::ColorComponentFlags::B;
    }
    if value.contains(RHIColorComponentFlags::A) {
        flags |= vk::ColorComponentFlags::A;
    }
    flags
}

pub fn map_pipeline_dynamic_state_create_flags(
    _value: RHIPipelineDynamicStateCreateFlags,
) -> vk::PipelineDynamicStateCreateFlags {
    let flags = vk::PipelineDynamicStateCreateFlags::empty();
    flags
}

pub fn map_dynamic_state(value: RHIDynamicState) -> vk::DynamicState {
    match value.to_i32() {
        None => vk::DynamicState::default(),
        Some(x) => vk::DynamicState::from_raw(x),
    }
}
