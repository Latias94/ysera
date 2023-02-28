#![allow(non_camel_case_types)]

use std::ffi::CString;

use bitflags::bitflags;

use rhi_types::{
    RHIAccessFlags, RHIAttachmentLoadOp, RHIAttachmentStoreOp, RHICompareOp, RHICullModeFlags,
    RHIDescriptorType, RHIDynamicState, RHIFormat, RHIFrontFace, RHIImageLayout, RHILogicOp,
    RHIPipelineColorBlendAttachmentState, RHIPipelineStageFlags, RHIPolygonMode,
    RHIPrimitiveTopology, RHIRect2D, RHISampleCountFlagBits, RHIShaderStageFlags,
    RHIStencilOpState,
};

use crate::RHI;

pub enum RHICommandBufferLevel {
    PRIMARY,
    SECONDARY,
}

#[derive(Copy, Clone)]
pub struct RHICommandPoolCreateInfo {
    pub flags: RHICommandPoolCreateFlags,
    pub queue_family_index: u32,
}

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    #[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkCommandPoolCreateFlagBits.html>"]
    pub struct RHICommandPoolCreateFlags: u16 {
        #[doc = "Command buffers have a short lifetime"]
        const TRANSIENT = 1 << 0;
        #[doc = "Command buffers may release their memory individually"]
        const RESET_COMMAND_BUFFER = 1 << 1;
    }
}

#[derive(Copy, Clone)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkRenderPassCreateInfo.html>"]
pub struct RHIRenderPassCreateInfo<'a> {
    pub flags: RHIRenderPassCreateFlags,
    pub attachments: &'a [RHIAttachmentDescription],
    pub subpasses: &'a [RHISubpassDescription<'a>],
    pub dependencies: &'a [RHISubpassDependency],
}

#[derive(Copy, Clone)]
pub struct RHIAttachmentDescription {
    pub flags: RHIAttachmentDescriptionFlags,
    pub format: RHIFormat,
    pub samples: RHISampleCountFlagBits,
    pub load_op: RHIAttachmentLoadOp,
    pub store_op: RHIAttachmentStoreOp,
    pub stecil_load_op: RHIAttachmentLoadOp,
    pub stecil_store_op: RHIAttachmentStoreOp,
    pub initial_layout: RHIImageLayout,
    pub final_layout: RHIImageLayout,
}

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    #[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkAttachmentDescriptionFlagBits.html>"]
    pub struct RHIAttachmentDescriptionFlags: u16 {
        #[doc = "The attachment may alias physical memory of another attachment in the same render pass"]
        const MAY_ALIAS = 1 << 0;
    }
}

#[derive(Copy, Clone)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkSubpassDescription.html>"]
pub struct RHISubpassDescription<'a> {
    pub flags: RHISubpassDescriptionFlags,
    pub pipeline_bind_point: RHIPipelineBindPoint,
    pub input_attachments: &'a [RHIAttachmentReference],
    pub color_attachments: &'a [RHIAttachmentReference],
    pub resolve_attachments: &'a [RHIAttachmentReference],
    pub depth_stencil_attachment: RHIAttachmentReference,
}

#[derive(Copy, Clone, Default)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkAttachmentReference.html>"]
pub struct RHIAttachmentReference {
    pub attachment: u32,
    pub layout: RHIImageLayout,
}

#[derive(FromPrimitive, ToPrimitive, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPipelineBindPoint.html>"]
pub enum RHIPipelineBindPoint {
    GRAPHICS = 0,
    COMPUTE = 1,
}

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    #[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkRenderPassCreateFlagBits.html>"]
    pub struct RHIRenderPassCreateFlags: u16 {
        #[doc = "Provided by VK_QCOM_render_pass_transform"]
        const CREATE_TRANSFORM_QCOM = 1 << 0;
    }
}

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    #[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkSubpassDescriptionFlagBits.html>"]
    pub struct RHISubpassDescriptionFlags: u16 {
    }
}

#[derive(Copy, Clone)]
pub struct RHISubpassDependency {
    pub src_subpass: u32,
    pub dst_subpass: u32,
    pub src_stage_mask: RHIPipelineStageFlags,
    pub dst_stage_mask: RHIPipelineStageFlags,
    pub src_access_mask: RHIAccessFlags,
    pub dst_access_mask: RHIAccessFlags,
    pub dependency_flags: RHIDependencyFlags,
}

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    #[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkDependencyFlagBits.html>"]
    pub struct RHIDependencyFlags: u16 {
        #[doc = "Dependency is per pixel region "]
        const BY_REGION = 1 << 0;
    }
}

#[derive(Copy, Clone)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPipelineLayoutCreateInfo.html>"]
pub struct RHIPipelineLayoutCreateInfo<'a, R: RHI> {
    pub flags: RHIPipelineLayoutCreateFlags,
    pub descriptor_set_layouts: &'a [R::DescriptorSetLayout],
    pub push_constant_ranges: &'a [RHIPushConstantRange],
}

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    #[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPipelineLayoutCreateFlagBits.html>"]
    pub struct RHIPipelineLayoutCreateFlags: u32 {
    }
}

#[derive(Copy, Clone)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPushConstantRange.html>"]
pub struct RHIPushConstantRange {
    pub stage_flags: RHIShaderStageFlags,
    pub offset: u32,
    pub size: u32,
}

#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkDescriptorSetLayoutCreateInfo.html>"]
pub struct RHIDescriptorSetLayoutCreateInfo<'a> {
    pub flags: RHIDescriptorSetLayoutCreateFlags,
    pub bindings: &'a [RHIDescriptorSetLayoutBinding],
}

#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkDescriptorSetLayoutBinding.html>"]
pub struct RHIDescriptorSetLayoutBinding {
    pub binding: u32,
    pub descriptor_type: RHIDescriptorType,
    pub descriptor_count: u32,
    pub stage_flags: RHIShaderStageFlags,
}

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    #[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkDescriptorSetLayoutCreateFlagBits.html>"]
    pub struct RHIDescriptorSetLayoutCreateFlags: u32 {
    }
}

#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkFramebufferCreateInfo.html>"]
pub struct RHIFramebufferCreateInfo<'a, R: RHI> {
    pub flags: RHIFramebufferCreateFlags,
    pub render_pass: &'a R::RenderPass,
    pub attachments: &'a [R::ImageView],
    pub width: u32,
    pub height: u32,
    pub layers: u32,
}

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    #[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkFramebufferCreateFlagBits.html>"]
    pub struct RHIFramebufferCreateFlags: u32 {
        #[doc = "Image views are not specified, and only attachment compatibility information will be provided via a VkFramebufferAttachmentImageInfo structure."]
        const IMAGELESS = 1 << 0;
    }
}

#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkGraphicsPipelineCreateInfo.html>"]
pub struct RHIGraphicsPipelineCreateInfo<'a, R: RHI> {
    pub flags: RHIPipelineCreateFlags,
    pub stages: &'a [RHIPipelineShaderStageCreateInfo<'a, R>],
    pub vertex_input_stage: &'a RHIPipelineVertexInputStateCreateInfo<'a>,
    pub input_assembly_stage: &'a RHIPipelineInputAssemblyStateCreateInfo,
    pub tessellation_stage: &'a RHIPipelineTessellationStateCreateInfo,
    pub viewport_stage: &'a RHIPipelineViewportStateCreateInfo<'a, R>,
    pub rasterization_stage: &'a RHIPipelineRasterizationStateCreateInfo,
    pub multisample_stage: &'a RHIPipelineMultisampleStateCreateInfo<'a>,
    pub depth_stencil_stage: &'a RHIPipelineDepthStencilStateCreateInfo,
    pub color_blend_stage: &'a RHIPipelineColorBlendStateCreateInfo<'a>,
    pub dynamic_stage: &'a RHIPipelineDynamicStateCreateInfo<'a>,
    pub layout: R::PipelineLayout,
    pub render_pass: R::RenderPass,
    pub subpass: u32,
    pub base_pipeline_index: u32,
}

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    #[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPipelineCreateFlagBits.html>"]
    pub struct RHIPipelineCreateFlags: u32 {
        const DISABLE_OPTIMIZATION = 1 << 0;
        const ALLOW_DERIVATIVES = 1 << 1;
        const DERIVATIVE = 1 << 2;
    }
}

#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPipelineShaderStageCreateInfo.html>"]
pub struct RHIPipelineShaderStageCreateInfo<'a, R: RHI> {
    pub flags: RHIPipelineShaderStageCreateFlags,
    pub stage: RHIShaderStageFlags,
    pub shader: R::Shader,
    pub name: &'a CString,
}

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    #[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPipelineShaderStageCreateFlagBits.html>"]
    pub struct RHIPipelineShaderStageCreateFlags: u32 {
    }
}

#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPipelineTessellationStateCreateInfo.html>"]
pub struct RHIPipelineTessellationStateCreateInfo {
    pub flags: RHIPipelineTessellationStateCreateFlags,
    pub patch_control_points: u32,
}

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    #[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPipelineTessellationStateCreateFlagBits.html>"]
    pub struct RHIPipelineTessellationStateCreateFlags: u32 {
    }
}

#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPipelineVertexInputStateCreateInfo.html>"]
pub struct RHIPipelineVertexInputStateCreateInfo<'a> {
    pub flags: RHIPipelineVertexInputStateCreateFlags,
    pub vertex_binding_descriptions: &'a [RHIVertexInputBindingDescription],
    pub vertex_input_attribute_descriptions: &'a [RHIVertexInputAttributeDescription],
}

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    #[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPipelineVertexInputStateCreateFlags.html>"]
    pub struct RHIPipelineVertexInputStateCreateFlags: u32 {
    }
}

#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkVertexInputBindingDescription.html>"]
pub struct RHIVertexInputBindingDescription {
    pub binding: u32,
    pub stride: u32,
    pub input_rate: RHIVertexInputRate,
}

#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkVertexInputAttributeDescription.html>"]
pub struct RHIVertexInputAttributeDescription {
    pub location: u32,
    pub binding: u32,
    pub format: RHIFormat,
    pub offset: u32,
}

#[derive(FromPrimitive, ToPrimitive, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkVertexInputRate.html>"]
pub enum RHIVertexInputRate {
    #[doc = "Vertex attribute addressing is a function of the vertex index."]
    VERTEX,
    #[doc = "Vertex attribute addressing is a function of the instance index."]
    INSTANCE,
}

#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPipelineInputAssemblyStateCreateInfo.html>"]
pub struct RHIPipelineInputAssemblyStateCreateInfo {
    pub flags: RHIPipelineInputAssemblyStateCreateFlags,
    pub primitive_topology: RHIPrimitiveTopology,
    #[doc = "Controls whether a special vertex index value is treated as restarting the assembly of primitives."]
    pub primitive_restart_enable: bool,
}

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    #[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPipelineInputAssemblyStateCreateFlags.html>"]
    pub struct RHIPipelineInputAssemblyStateCreateFlags: u32 {
    }
}

#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPipelineViewportStateCreateInfo.html>"]
pub struct RHIPipelineViewportStateCreateInfo<'a, R: RHI> {
    pub flags: RHIPipelineViewportStateCreateFlags,
    pub viewports: &'a [R::Viewport],
    pub scissors: &'a [RHIRect2D],
}

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    #[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPipelineViewportStateCreateFlags.html>"]
    pub struct RHIPipelineViewportStateCreateFlags: u32 {
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Default)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPipelineRasterizationStateCreateInfo.html>"]
pub struct RHIPipelineRasterizationStateCreateInfo {
    pub flags: RHIPipelineRasterizationStateCreateFlags,
    pub depth_clamp_enable: bool,
    pub rasterizer_discard_enable: bool,
    pub polygon_mode: RHIPolygonMode,
    pub cull_mode: RHICullModeFlags,
    pub front_face: RHIFrontFace,
    pub depth_bias_enable: bool,
    pub depth_bias_constant_factor: f32,
    pub depth_bias_clamp: f32,
    pub depth_bias_slope_factor: f32,
    pub line_width: f32,
}

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    #[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPipelineRasterizationStateCreateFlags.html>"]
    pub struct RHIPipelineRasterizationStateCreateFlags: u32 {
    }
}

#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkSampleMask.html>"]
pub type RHISampleMask = u32;

#[derive(Clone, Copy, PartialEq, PartialOrd, Default)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPipelineMultisampleStateCreateInfo.html>"]
pub struct RHIPipelineMultisampleStateCreateInfo<'a> {
    pub flags: RHIPipelineMultisampleStateCreateFlags,
    pub rasterization_samples: RHISampleCountFlagBits,
    pub sample_shading_enable: bool,
    pub min_sample_shading: f32,
    pub sample_masks: &'a [RHISampleMask],
    pub alpha_to_coverage_enable: bool,
    pub alpha_to_one_enable: bool,
}

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    #[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPipelineMultisampleStateCreateFlags.html>"]
    pub struct RHIPipelineMultisampleStateCreateFlags: u32 {
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPipelineDepthStencilStateCreateInfo.html>"]
pub struct RHIPipelineDepthStencilStateCreateInfo {
    pub flags: RHIPipelineDepthStencilStateCreateFlags,
    pub depth_test_enable: bool,
    pub depth_write_enable: bool,
    pub depth_compare_op: RHICompareOp,
    pub depth_bounds_test_enable: bool,
    pub stencil_test_enable: bool,
    pub front: RHIStencilOpState,
    pub back: RHIStencilOpState,
    pub min_depth_bounds: f32,
    pub max_depth_bounds: f32,
}

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    #[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPipelineDepthStencilStateCreateFlags.html>"]
    pub struct RHIPipelineDepthStencilStateCreateFlags: u32 {
    }
}

#[derive(Clone, Copy)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPipelineColorBlendStateCreateInfo.html>"]
pub struct RHIPipelineColorBlendStateCreateInfo<'a> {
    pub flags: RHIPipelineColorBlendStateCreateFlags,
    pub logic_op_enable: bool,
    pub logic_op: RHILogicOp,
    pub attachments: &'a [RHIPipelineColorBlendAttachmentState],
    pub blend_constants: [f32; 4],
}

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    #[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPipelineColorBlendStateCreateFlags.html>"]
    pub struct RHIPipelineColorBlendStateCreateFlags: u32 {
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Default)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPipelineDynamicStateCreateInfo.html>"]
pub struct RHIPipelineDynamicStateCreateInfo<'a> {
    pub flags: RHIPipelineDynamicStateCreateFlags,
    pub dynamic_states: &'a [RHIDynamicState],
}

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    #[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPipelineDynamicStateCreateFlags.html>"]
    pub struct RHIPipelineDynamicStateCreateFlags: u32 {
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Default)]
pub struct RHIShaderCreateInfo<'a> {
    pub spv: &'a [u32],
}
