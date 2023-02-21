#![allow(non_camel_case_types)]

use bitflags::bitflags;
use typed_builder::TypedBuilder;

pub enum RHICommandBufferLevel {
    PRIMARY,
    SECONDARY,
}

#[derive(FromPrimitive, ToPrimitive, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkFormat.html>"]
pub enum RHIFormat {
    UNDEFINED = 0,
    R4G4_UNORM_PACK8 = 1,
    R4G4B4A4_UNORM_PACK16 = 2,
    B4G4R4A4_UNORM_PACK16 = 3,
    R5G6B5_UNORM_PACK16 = 4,
    B5G6R5_UNORM_PACK16 = 5,
    R5G5B5A1_UNORM_PACK16 = 6,
    B5G5R5A1_UNORM_PACK16 = 7,
    A1R5G5B5_UNORM_PACK16 = 8,
    R8_UNORM = 9,
    R8_SNORM = 10,
    R8_USCALED = 11,
    R8_SSCALED = 12,
    R8_UINT = 13,
    R8_SINT = 14,
    R8_SRGB = 15,
    R8G8_UNORM = 16,
    R8G8_SNORM = 17,
    R8G8_USCALED = 18,
    R8G8_SSCALED = 19,
    R8G8_UINT = 20,
    R8G8_SINT = 21,
    R8G8_SRGB = 22,
    R8G8B8_UNORM = 23,
    R8G8B8_SNORM = 24,
    R8G8B8_USCALED = 25,
    R8G8B8_SSCALED = 26,
    R8G8B8_UINT = 27,
    R8G8B8_SINT = 28,
    R8G8B8_SRGB = 29,
    B8G8R8_UNORM = 30,
    B8G8R8_SNORM = 31,
    B8G8R8_USCALED = 32,
    B8G8R8_SSCALED = 33,
    B8G8R8_UINT = 34,
    B8G8R8_SINT = 35,
    B8G8R8_SRGB = 36,
    R8G8B8A8_UNORM = 37,
    R8G8B8A8_SNORM = 38,
    R8G8B8A8_USCALED = 39,
    R8G8B8A8_SSCALED = 40,
    R8G8B8A8_UINT = 41,
    R8G8B8A8_SINT = 42,
    R8G8B8A8_SRGB = 43,
    B8G8R8A8_UNORM = 44,
    B8G8R8A8_SNORM = 45,
    B8G8R8A8_USCALED = 46,
    B8G8R8A8_SSCALED = 47,
    B8G8R8A8_UINT = 48,
    B8G8R8A8_SINT = 49,
    B8G8R8A8_SRGB = 50,
    A8B8G8R8_UNORM_PACK32 = 51,
    A8B8G8R8_SNORM_PACK32 = 52,
    A8B8G8R8_USCALED_PACK32 = 53,
    A8B8G8R8_SSCALED_PACK32 = 54,
    A8B8G8R8_UINT_PACK32 = 55,
    A8B8G8R8_SINT_PACK32 = 56,
    A8B8G8R8_SRGB_PACK32 = 57,
    A2R10G10B10_UNORM_PACK32 = 58,
    A2R10G10B10_SNORM_PACK32 = 59,
    A2R10G10B10_USCALED_PACK32 = 60,
    A2R10G10B10_SSCALED_PACK32 = 61,
    A2R10G10B10_UINT_PACK32 = 62,
    A2R10G10B10_SINT_PACK32 = 63,
    A2B10G10R10_UNORM_PACK32 = 64,
    A2B10G10R10_SNORM_PACK32 = 65,
    A2B10G10R10_USCALED_PACK32 = 66,
    A2B10G10R10_SSCALED_PACK32 = 67,
    A2B10G10R10_UINT_PACK32 = 68,
    A2B10G10R10_SINT_PACK32 = 69,
    R16_UNORM = 70,
    R16_SNORM = 71,
    R16_USCALED = 72,
    R16_SSCALED = 73,
    R16_UINT = 74,
    R16_SINT = 75,
    R16_SFLOAT = 76,
    R16G16_UNORM = 77,
    R16G16_SNORM = 78,
    R16G16_USCALED = 79,
    R16G16_SSCALED = 80,
    R16G16_UINT = 81,
    R16G16_SINT = 82,
    R16G16_SFLOAT = 83,
    R16G16B16_UNORM = 84,
    R16G16B16_SNORM = 85,
    R16G16B16_USCALED = 86,
    R16G16B16_SSCALED = 87,
    R16G16B16_UINT = 88,
    R16G16B16_SINT = 89,
    R16G16B16_SFLOAT = 90,
    R16G16B16A16_UNORM = 91,
    R16G16B16A16_SNORM = 92,
    R16G16B16A16_USCALED = 93,
    R16G16B16A16_SSCALED = 94,
    R16G16B16A16_UINT = 95,
    R16G16B16A16_SINT = 96,
    R16G16B16A16_SFLOAT = 97,
    R32_UINT = 98,
    R32_SINT = 99,
    R32_SFLOAT = 100,
    R32G32_UINT = 101,
    R32G32_SINT = 102,
    R32G32_SFLOAT = 103,
    R32G32B32_UINT = 104,
    R32G32B32_SINT = 105,
    R32G32B32_SFLOAT = 106,
    R32G32B32A32_UINT = 107,
    R32G32B32A32_SINT = 108,
    R32G32B32A32_SFLOAT = 109,
    R64_UINT = 110,
    R64_SINT = 111,
    R64_SFLOAT = 112,
    R64G64_UINT = 113,
    R64G64_SINT = 114,
    R64G64_SFLOAT = 115,
    R64G64B64_UINT = 116,
    R64G64B64_SINT = 117,
    R64G64B64_SFLOAT = 118,
    R64G64B64A64_UINT = 119,
    R64G64B64A64_SINT = 120,
    R64G64B64A64_SFLOAT = 121,
    B10G11R11_UFLOAT_PACK32 = 122,
    E5B9G9R9_UFLOAT_PACK32 = 123,
    D16_UNORM = 124,
    X8_D24_UNORM_PACK32 = 125,
    D32_SFLOAT = 126,
    S8_UINT = 127,
    D16_UNORM_S8_UINT = 128,
    D24_UNORM_S8_UINT = 129,
    D32_SFLOAT_S8_UINT = 130,
    BC1_RGB_UNORM_BLOCK = 131,
    BC1_RGB_SRGB_BLOCK = 132,
    BC1_RGBA_UNORM_BLOCK = 133,
    BC1_RGBA_SRGB_BLOCK = 134,
    BC2_UNORM_BLOCK = 135,
    BC2_SRGB_BLOCK = 136,
    BC3_UNORM_BLOCK = 137,
    BC3_SRGB_BLOCK = 138,
    BC4_UNORM_BLOCK = 139,
    BC4_SNORM_BLOCK = 140,
    BC5_UNORM_BLOCK = 141,
    BC5_SNORM_BLOCK = 142,
    BC6H_UFLOAT_BLOCK = 143,
    BC6H_SFLOAT_BLOCK = 144,
    BC7_UNORM_BLOCK = 145,
    BC7_SRGB_BLOCK = 146,
    ETC2_R8G8B8_UNORM_BLOCK = 147,
    ETC2_R8G8B8_SRGB_BLOCK = 148,
    ETC2_R8G8B8A1_UNORM_BLOCK = 149,
    ETC2_R8G8B8A1_SRGB_BLOCK = 150,
    ETC2_R8G8B8A8_UNORM_BLOCK = 151,
    ETC2_R8G8B8A8_SRGB_BLOCK = 152,
    EAC_R11_UNORM_BLOCK = 153,
    EAC_R11_SNORM_BLOCK = 154,
    EAC_R11G11_UNORM_BLOCK = 155,
    EAC_R11G11_SNORM_BLOCK = 156,
    ASTC_4x4_UNORM_BLOCK = 157,
    ASTC_4x4_SRGB_BLOCK = 158,
    ASTC_5x4_UNORM_BLOCK = 159,
    ASTC_5x4_SRGB_BLOCK = 160,
    ASTC_5x5_UNORM_BLOCK = 161,
    ASTC_5x5_SRGB_BLOCK = 162,
    ASTC_6x5_UNORM_BLOCK = 163,
    ASTC_6x5_SRGB_BLOCK = 164,
    ASTC_6x6_UNORM_BLOCK = 165,
    ASTC_6x6_SRGB_BLOCK = 166,
    ASTC_8x5_UNORM_BLOCK = 167,
    ASTC_8x5_SRGB_BLOCK = 168,
    ASTC_8x6_UNORM_BLOCK = 169,
    ASTC_8x6_SRGB_BLOCK = 170,
    ASTC_8x8_UNORM_BLOCK = 171,
    ASTC_8x8_SRGB_BLOCK = 172,
    ASTC_10x5_UNORM_BLOCK = 173,
    ASTC_10x5_SRGB_BLOCK = 174,
    ASTC_10x6_UNORM_BLOCK = 175,
    ASTC_10x6_SRGB_BLOCK = 176,
    ASTC_10x8_UNORM_BLOCK = 177,
    ASTC_10x8_SRGB_BLOCK = 178,
    ASTC_10x10_UNORM_BLOCK = 179,
    ASTC_10x10_SRGB_BLOCK = 180,
    ASTC_12x10_UNORM_BLOCK = 181,
    ASTC_12x10_SRGB_BLOCK = 182,
    ASTC_12x12_UNORM_BLOCK = 183,
    ASTC_12x12_SRGB_BLOCK = 184,
}

pub enum RHIImageType {
    D1,
    D2,
    D3,
}

#[derive(Copy, Clone)]
pub struct RHIExtent2D {
    pub width: u32,
    pub height: u32,
}

#[derive(Copy, Clone)]
pub struct RHIExtent3D {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

#[derive(Copy, Clone)]
pub struct RHIViewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub min_depth: f32,
    pub max_depth: f32,
}

#[derive(Copy, Clone)]
pub struct RHIOffset2D {
    pub x: u32,
    pub y: u32,
}

#[derive(Copy, Clone)]
pub struct RHIRect2D {
    pub offset: RHIOffset2D,
    pub extent: RHIExtent2D,
}

#[derive(Copy, Clone)]
pub struct RHICommandPoolCreateInfo {
    pub flags: RHICommandPoolCreateFlags,
    pub queue_family_index: u32,
}

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[derive(FromPrimitive, ToPrimitive, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkSampleCountFlagBits.html>"]
pub enum RHISampleCountFlagBits {
    TYPE_1 = 1 << 0,
    TYPE_2 = 1 << 1,
    TYPE_4 = 1 << 2,
    TYPE_8 = 1 << 3,
    TYPE_16 = 1 << 4,
    TYPE_32 = 1 << 5,
    TYPE_64 = 1 << 6,
}

#[derive(FromPrimitive, ToPrimitive, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkImageTiling.html>"]
pub enum RHIImageTiling {
    OPTIMAL = 0,
    LINEAR = 1,
}

#[derive(FromPrimitive, ToPrimitive, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkImageType.html>"]
pub enum RHIAttachmentLoadOp {
    LOAD = 0,
    CLEAR = 1,
    DONT_CARE = 2,
}

#[derive(FromPrimitive, ToPrimitive, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkAttachmentStoreOp.html>"]
pub enum RHIAttachmentStoreOp {
    STORE = 0,
    DONT_CARE = 1,
}

#[derive(
    FromPrimitive, ToPrimitive, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default,
)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkImageLayout.html>"]
pub enum RHIImageLayout {
    #[default]
    #[doc = "Implicit layout an image is when its contents are undefined due to various reasons (e.g. right after creation)"]
    UNDEFINED = 0,
    #[doc = "General layout when image can be used for any kind of access"]
    GENERAL = 1,
    #[doc = "Optimal layout when image is only used for color attachment read/write"]
    COLOR_ATTACHMENT_OPTIMAL = 2,
    #[doc = "Optimal layout when image is only used for depth/stencil attachment read/write"]
    DEPTH_STENCIL_ATTACHMENT_OPTIMAL = 3,
    #[doc = "Optimal layout when image is used for read only depth/stencil attachment and shader access"]
    DEPTH_STENCIL_READ_ONLY_OPTIMAL = 4,
    #[doc = "Optimal layout when image is used for read only shader access"]
    SHADER_READ_ONLY_OPTIMAL = 5,
    #[doc = "Optimal layout when image is used only as source of transfer operations"]
    TRANSFER_SRC_OPTIMAL = 6,
    #[doc = "Optimal layout when image is used only as destination of transfer operations"]
    TRANSFER_DST_OPTIMAL = 7,
    #[doc = "Initial layout used when the data is populated by the CPU"]
    PREINITIALIZED = 8,
}

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkRenderPassCreateFlagBits.html>"]
    pub struct RHIRenderPassCreateFlags: u16 {
        #[doc = "Provided by VK_QCOM_render_pass_transform"]
        const CREATE_TRANSFORM_QCOM = 1 << 0;
    }
}

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPipelineStageFlagBits.html>"]
    pub struct RHIPipelineStageFlags: u32 {
        #[doc = "Before subsequent commands are processed"]
        const TOP_OF_PIPE = 0x00000001;
        #[doc = "Draw/DispatchIndirect command fetch"]
        const DRAW_INDIRECT = 0x00000002;
        #[doc = "Vertex/index fetch"]
        const VERTEX_INPUT = 0x00000004;
        #[doc = "Vertex shading"]
        const VERTEX_SHADER = 0x00000008;
        #[doc = "Tessellation control shading"]
        const TESSELLATION_CONTROL_SHADER = 0x00000010;
        #[doc = "Tessellation evaluation shading"]
        const TESSELLATION_EVALUATION_SHADER = 0x00000020;
        #[doc = "Geometry shading"]
        const GEOMETRY_SHADER = 0x00000040;
        #[doc = "Fragment shading"]
        const FRAGMENT_SHADER = 0x00000080;
        #[doc = "Early fragment (depth and stencil) tests"]
        const EARLY_FRAGMENT_TESTS = 0x00000100;
        #[doc = "Late fragment (depth and stencil) tests"]
        const LATE_FRAGMENT_TESTS = 0x00000200;
        #[doc = "Color attachment writes"]
        const COLOR_ATTACHMENT_OUTPUT = 0x00000400;
        #[doc = "Compute shading"]
        const COMPUTE_SHADER = 0x00000800;
        #[doc = "Transfer/copy operations"]
        const TRANSFER = 0x00001000;
        #[doc = "After previous commands have completed"]
        const BOTTOM_OF_PIPE = 0x00002000;
        #[doc = "Indicates host (CPU) is a source/sink of the dependency"]
        const HOST = 0x00004000;
        #[doc = "All stages of the graphics pipeline"]
        const ALL_GRAPHICS = 0x00008000;
        #[doc = "All stages supported on the queue"]
        const ALL_COMMANDS = 0x00010000;
    }
}

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkAccessFlagBits.html>"]
    pub struct RHIAccessFlags: u32 {
        #[doc = "Controls coherency of indirect command reads"]
        const INDIRECT_COMMAND_READ = 1 << 0;
        #[doc = "Controls coherency of index reads"]
        const INDEX_READ = 1 << 1;
        #[doc = "Controls coherency of vertex attribute reads"]
        const VERTEX_ATTRIBUTE_READ = 1 << 2;
        #[doc = "Controls coherency of uniform buffer reads"]
        const UNIFORM_READ = 1 << 3;
        #[doc = "Controls coherency of input attachment reads"]
        const INPUT_ATTACHMENT_READ = 1 << 4;
        #[doc = "Controls coherency of shader reads"]
        const SHADER_READ = 1 << 5;
        #[doc = "Controls coherency of shader writes"]
        const SHADER_WRITE = 1 << 6;
        #[doc = "Controls coherency of color attachment reads"]
        const COLOR_ATTACHMENT_READ = 1 << 7;
        #[doc = "Controls coherency of color attachment writes"]
        const COLOR_ATTACHMENT_WRITE = 1 << 8;
        #[doc = "Controls coherency of depth/stencil attachment reads"]
        const DEPTH_STENCIL_ATTACHMENT_READ = 1 << 9;
        #[doc = "Controls coherency of depth/stencil attachment writes"]
        const DEPTH_STENCIL_ATTACHMENT_WRITE = 1 << 10;
        #[doc = "Controls coherency of transfer reads"]
        const TRANSFER_READ = 1 << 11;
        #[doc = "Controls coherency of transfer writes"]
        const TRANSFER_WRITE = 1 << 12;
        #[doc = "Controls coherency of host reads"]
        const HOST_READ = 1 << 13;
        #[doc = "Controls coherency of host writes"]
        const HOST_WRITE = 1 << 14;
        #[doc = "Controls coherency of memory reads"]
        const MEMORY_READ = 1 << 15;
        #[doc = "Controls coherency of memory writes"]
        const MEMORY_WRITE = 1 << 16;
    }
}

bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkDependencyFlagBits.html>"]
    pub struct RHIDependencyFlags: u16 {
        #[doc = "Dependency is per pixel region "]
        const BY_REGION = 1<<0;
    }
}

#[derive(Clone, Debug, TypedBuilder)]
pub struct InstanceDescriptor<'a> {
    #[builder(default)]
    pub name: &'a str,
    #[builder(default = InstanceFlags::all())]
    pub flags: InstanceFlags,
    #[builder(default = log::LevelFilter::Warn)]
    pub debug_level_filter: log::LevelFilter,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct InstanceFlags: u16 {
        const DEBUG = 1 << 0;
        const VALIDATION = 1 << 1;
    }
}

#[derive(Debug, TypedBuilder)]
pub struct PhysicalDeviceRequirements {
    // queue requirement
    #[builder(default = true)]
    pub graphics: bool,
    #[builder(default = true)]
    pub present: bool,
    #[builder(default = true)]
    pub compute: bool,
    #[builder(default = true)]
    pub transfer: bool,
    #[builder(default)]
    pub extra_features: DeviceFeatures,
    #[builder(default = true)]
    pub discrete_gpu: bool,
}

#[derive(Debug, Clone, Copy, Default, TypedBuilder)]
pub struct DeviceFeatures {
    // vk::PhysicalDeviceFeatures
    #[builder(default = true)]
    pub sampler_anisotropy: bool,
    #[builder(default = true)]
    pub sample_rate_shading: bool,
    #[builder(default = true)]
    pub fragment_stores_and_atomics: bool,
    #[builder(default = true)]
    pub independent_blend: bool,
    #[builder(default = false)]
    pub geometry_shader: bool,
    // vk::PhysicalDeviceFeatures2 12 13
    #[builder(default = false)]
    pub ray_tracing_pipeline: bool,
    #[builder(default = false)]
    pub acceleration_structure: bool,
    #[builder(default = false)]
    pub runtime_descriptor_array: bool,
    #[builder(default = false)]
    pub buffer_device_address: bool,
    #[builder(default = false)]
    pub dynamic_rendering: bool,
    #[builder(default = true)]
    pub synchronization2: bool,
}

impl DeviceFeatures {
    pub fn is_compatible_with(&self, requirements: &Self) -> bool {
        (!requirements.sampler_anisotropy || self.sampler_anisotropy)
            && (!requirements.sample_rate_shading || self.sample_rate_shading)
            && (!requirements.fragment_stores_and_atomics || self.fragment_stores_and_atomics)
            && (!requirements.independent_blend || self.independent_blend)
            && (!requirements.geometry_shader || self.geometry_shader)
            && (!requirements.ray_tracing_pipeline || self.ray_tracing_pipeline)
            && (!requirements.acceleration_structure || self.acceleration_structure)
            && (!requirements.runtime_descriptor_array || self.runtime_descriptor_array)
            && (!requirements.buffer_device_address || self.buffer_device_address)
            && (!requirements.dynamic_rendering || self.dynamic_rendering)
            && (!requirements.synchronization2 || self.synchronization2)
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
    pub compute_family: Option<u32>,
    pub transfer_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn has_meet_requirement(&self, requirements: &PhysicalDeviceRequirements) -> bool {
        if requirements.graphics && self.graphics_family.is_none() {
            return false;
        }
        if requirements.present && self.present_family.is_none() {
            return false;
        }
        if requirements.compute && self.compute_family.is_none() {
            return false;
        }
        if requirements.transfer && self.transfer_family.is_none() {
            return false;
        }
        true
    }

    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some()
            && self.transfer_family.is_some()
            && self.present_family.is_some()
            && self.compute_family.is_some()
    }

    pub fn log_debug(&self) {
        if self.graphics_family.is_some() {
            log::debug!(
                "graphics family indices is {}, ",
                self.graphics_family.unwrap()
            );
        }
        if self.present_family.is_some() {
            log::debug!("present family indices is {}", self.present_family.unwrap());
        }
        if self.compute_family.is_some() {
            log::debug!("compute family indices is {}", self.compute_family.unwrap());
        }
        if self.transfer_family.is_some() {
            log::debug!(
                "transfer family indices is {}",
                self.transfer_family.unwrap()
            );
        }
    }
}

#[derive(Debug, TypedBuilder)]
pub struct DeviceRequirement {
    /// extension except swapchain ext
    pub required_extension: Vec<String>,
    /// Set to false for headless rendering to omit the swapchain device extensions
    #[builder(default = true)]
    pub use_swapchain: bool,
}
