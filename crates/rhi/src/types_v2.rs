#![allow(non_camel_case_types)]

use bitflags::bitflags;
use rhi_types::{
    RHIAccessFlags, RHIAttachmentLoadOp, RHIAttachmentStoreOp, RHIDescriptorType, RHIFormat,
    RHIImageLayout, RHIPipelineStageFlags, RHIPrimitiveTopology, RHIRect2D, RHISampleCountFlagBits,
    RHIShaderStageFlags,
};
use typed_builder::TypedBuilder;

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
    pub stages: &'a [RHIPipelineShaderStageCreateInfo<R>],
    pub vertex_input_stage: &'a RHIPipelineVertexInputStateCreateInfo<'a>,
    pub input_assembly_stage: &'a RHIPipelineInputAssemblyStateCreateInfo,
    pub tessellation_stage: &'a RHIPipelineTessellationStateCreateInfo,
    pub viewport_stage: &'a RHIPipelineViewportStateCreateInfo<'a, R>,
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
pub struct RHIPipelineShaderStageCreateInfo<R: RHI> {
    pub flags: RHIPipelineShaderStageCreateFlags,
    pub stage: RHIShaderStageFlags,
    pub shader: R::Shader,
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
