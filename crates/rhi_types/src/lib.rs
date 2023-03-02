#![allow(non_camel_case_types)]

#[macro_use]
extern crate num_derive;

use bitflags::bitflags;
use typed_builder::TypedBuilder;

#[derive(Debug, Clone, Copy, TypedBuilder)]
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

#[derive(Debug, Clone, TypedBuilder)]
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

#[derive(Debug, Clone, TypedBuilder)]
pub struct DeviceRequirement {
    /// extension except swapchain ext
    pub required_extension: Vec<String>,
    /// Set to false for headless rendering to omit the swapchain device extensions
    #[builder(default = true)]
    pub use_swapchain: bool,
}

#[derive(Debug, Copy, Clone, Default)]
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

#[derive(Copy, Clone, Default)]
pub struct RHIOffset2D {
    pub x: i32,
    pub y: i32,
}

#[derive(TypedBuilder, Copy, Clone)]
pub struct RHIRect2D {
    #[builder(default)]
    pub offset: RHIOffset2D,
    pub extent: RHIExtent2D,
}

#[derive(
    FromPrimitive, ToPrimitive, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default,
)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkSampleCountFlagBits.html>"]
pub enum RHISampleCountFlagBits {
    #[default]
    TYPE_1 = 1 << 0,
    TYPE_2 = 1 << 1,
    TYPE_4 = 1 << 2,
    TYPE_8 = 1 << 3,
    TYPE_16 = 1 << 4,
    TYPE_32 = 1 << 5,
    TYPE_64 = 1 << 6,
}

#[derive(FromPrimitive, ToPrimitive, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkImageTiling.html>"]
pub enum RHIImageTiling {
    OPTIMAL = 0,
    LINEAR = 1,
}

#[derive(FromPrimitive, ToPrimitive, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkImageType.html>"]
pub enum RHIAttachmentLoadOp {
    LOAD = 0,
    CLEAR = 1,
    DONT_CARE = 2,
}

#[derive(FromPrimitive, ToPrimitive, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkAttachmentStoreOp.html>"]
pub enum RHIAttachmentStoreOp {
    STORE = 0,
    DONT_CARE = 1,
}

#[derive(
    FromPrimitive, ToPrimitive, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default,
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
    #[doc = "Generated from 'VK_KHR_swapchain'"]
    PRESENT_SRC_KHR = 1_000_001_002,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
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
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
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
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    #[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkShaderStageFlagBits.html>"]
    pub struct RHIShaderStageFlags: u32 {
        const VERTEX = 1 << 0;
        const TESSELLATION_CONTROL = 1 << 1;
        const TESSELLATION_EVALUATION = 1 << 2;
        const GEOMETRY = 1 << 3;
        const FRAGMENT = 1 << 4;
        const COMPUTE = 1 << 5;
    }
}

#[derive(
    FromPrimitive, ToPrimitive, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default,
)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPrimitiveTopology.html>"]
pub enum RHIPrimitiveTopology {
    POINT_LIST = 0,
    LINE_LIST = 1,
    LINE_STRIP = 2,
    #[default]
    TRIANGLE_LIST = 3,
    TRIANGLE_STRIP = 4,
    TRIANGLE_FAN = 5,
    LINE_LIST_WITH_ADJACENCY = 6,
    LINE_STRIP_WITH_ADJACENCY = 7,
    TRIANGLE_LIST_WITH_ADJACENCY = 8,
    TRIANGLE_STRIP_WITH_ADJACENCY = 9,
    PATCH_LIST = 10,
}

#[derive(
    FromPrimitive, ToPrimitive, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default,
)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkDescriptorType.html>"]
pub enum RHIDescriptorType {
    #[default]
    SAMPLER = 0,
    COMBINED_IMAGE_SAMPLER = 1,
    SAMPLED_IMAGE = 2,
    STORAGE_IMAGE = 3,
    UNIFORM_TEXEL_BUFFER = 4,
    STORAGE_TEXEL_BUFFER = 5,
    UNIFORM_BUFFER = 6,
    STORAGE_BUFFER = 7,
    UNIFORM_BUFFER_DYNAMIC = 8,
    STORAGE_BUFFER_DYNAMIC = 9,
    INPUT_ATTACHMENT = 10,
}

#[derive(
    FromPrimitive, ToPrimitive, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default,
)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPolygonMode.html>"]
pub enum RHIPolygonMode {
    #[default]
    FILL = 0,
    LINE = 1,
    POINT = 2,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    #[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkCullModeFlagBits.html>"]
    pub struct RHICullModeFlags: u32 {
        const NONE = 1 << 0;
        const FRONT = 1 << 1;
        const BACK = 1 << 2;
    }
}

#[derive(
    FromPrimitive, ToPrimitive, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default,
)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkFrontFace.html>"]
pub enum RHIFrontFace {
    #[default]
    COUNTER_CLOCKWISE = 0,
    CLOCKWISE = 1,
}

#[derive(
    FromPrimitive, ToPrimitive, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default,
)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkCompareOp.html>"]
pub enum RHICompareOp {
    #[default]
    NEVER = 0,
    LESS = 1,
    EQUAL = 2,
    LESS_OR_EQUAL = 3,
    GREATER = 4,
    NOT_EQUAL = 5,
    GREATER_OR_EQUAL = 6,
    ALWAYS = 7,
}

#[derive(TypedBuilder, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkStencilOpState.html>"]
pub struct RHIStencilOpState {
    #[builder(default = RHIStencilOp::KEEP)]
    pub fail_op: RHIStencilOp,
    #[builder(default = RHIStencilOp::KEEP)]
    pub pass_op: RHIStencilOp,
    #[builder(default = RHIStencilOp::KEEP)]
    pub depth_fail_op: RHIStencilOp,
    #[builder(default = RHICompareOp::NEVER)]
    pub compare_op: RHICompareOp,
    pub compare_mask: u32,
    pub write_mask: u32,
    pub reference: u32,
}

#[derive(
    FromPrimitive, ToPrimitive, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default,
)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkStencilOp.html>"]
pub enum RHIStencilOp {
    #[default]
    KEEP = 0,
    ZERO = 1,
    REPLACE = 2,
    INCREMENT_AND_CLAMP = 3,
    DECREMENT_AND_CLAMP = 4,
    INVERT = 5,
    INCREMENT_AND_WRAP = 6,
    DECREMENT_AND_WRAP = 7,
}

#[derive(
    FromPrimitive, ToPrimitive, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default,
)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkLogicOp.html>"]
pub enum RHILogicOp {
    #[default]
    CLEAR = 0,
    AND = 1,
    AND_REVERSE = 2,
    COPY = 3,
    AND_INVERTED = 4,
    NO_OP = 5,
    XOR = 6,
    OR = 7,
    NOR = 8,
    EQUIVALENT = 9,
    INVERT = 10,
    OR_REVERSE = 11,
    COPY_INVERTED = 12,
    OR_INVERTED = 13,
    NAND = 14,
    SET = 15,
}

#[derive(TypedBuilder, Debug, Clone, Copy)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPipelineColorBlendAttachmentState.html>"]
pub struct RHIPipelineColorBlendAttachmentState {
    #[builder(default)]
    pub blend_enable: bool,
    #[builder(default = RHIBlendFactor::ONE)]
    pub src_color_blend_factor: RHIBlendFactor,
    #[builder(default = RHIBlendFactor::ZERO)]
    pub dst_color_blend_factor: RHIBlendFactor,
    #[builder(default = RHIBlendOp::ADD)]
    pub color_blend_op: RHIBlendOp,
    #[builder(default = RHIBlendFactor::ONE)]
    pub src_alpha_blend_factor: RHIBlendFactor,
    #[builder(default = RHIBlendFactor::ZERO)]
    pub dst_alpha_blend_factor: RHIBlendFactor,
    #[builder(default = RHIBlendOp::ADD)]
    pub alpha_blend_op: RHIBlendOp,
    #[builder(default = RHIColorComponentFlags::RGBA)]
    pub color_write_mask: RHIColorComponentFlags,
}

#[derive(FromPrimitive, ToPrimitive, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkBlendFactor.html>"]
pub enum RHIBlendFactor {
    ZERO = 0,
    ONE = 1,
    SRC_COLOR = 2,
    ONE_MINUS_SRC_COLOR = 3,
    DST_COLOR = 4,
    ONE_MINUS_DST_COLOR = 5,
    SRC_ALPHA = 6,
    ONE_MINUS_SRC_ALPHA = 7,
    DST_ALPHA = 8,
    ONE_MINUS_DST_ALPHA = 9,
    CONSTANT_COLOR = 10,
    ONE_MINUS_CONSTANT_COLOR = 11,
    CONSTANT_ALPHA = 12,
    ONE_MINUS_CONSTANT_ALPHA = 13,
    SRC_ALPHA_SATURATE = 14,
    SRC1_COLOR = 15,
    ONE_MINUS_SRC1_COLOR = 16,
    SRC1_ALPHA = 17,
    ONE_MINUS_SRC1_ALPHA = 18,
}

#[derive(
    FromPrimitive, ToPrimitive, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default,
)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkBlendOp.html>"]
pub enum RHIBlendOp {
    #[default]
    ADD = 0,
    SUBTRACT = 1,
    REVERSE_SUBTRACT = 2,
    MIN = 3,
    MAX = 4,
}

bitflags! {
    #[derive(Debug, Clone,  Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
    #[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkColorComponentFlagBits.html>"]
    pub struct RHIColorComponentFlags: u32 {
        const R = 1 << 0;
        const G = 1 << 1;
        const B = 1 << 2;
        const A = 1 << 3;
        const RGBA = Self::R.bits() | Self::G.bits() | Self::B.bits() | Self::A.bits();
    }
}

#[derive(FromPrimitive, ToPrimitive, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkDynamicState.html>"]
pub enum RHIDynamicState {
    VIEWPORT = 0,
    SCISSOR = 1,
    LINE_WIDTH = 2,
    DEPTH_BIAS = 3,
    BLEND_CONSTANTS = 4,
    DEPTH_BOUNDS = 5,
    STENCIL_COMPARE_MASK = 6,
    STENCIL_WRITE_MASK = 7,
    STENCIL_REFERENCE = 8,
}

#[repr(C)]
#[derive(Clone, Copy)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkClearValue.html>"]
pub union RHIClearValue {
    pub color: RHIClearColorValue,
    pub depth_stencil: RHIClearDepthStencilValue,
}

#[repr(C)]
#[derive(Copy, Clone)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkClearColorValue.html>"]
pub union RHIClearColorValue {
    pub float32: [f32; 4],
    pub int32: [i32; 4],
    pub uint32: [u32; 4],
}

#[repr(C)]
#[derive(Copy, Clone)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkClearDepthStencilValue.html>"]
pub struct RHIClearDepthStencilValue {
    pub depth: f32,
    pub stencil: u32,
}

#[derive(
    FromPrimitive, ToPrimitive, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default,
)]
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkSubpassContents.html>"]
pub enum RHISubpassContents {
    #[default]
    INLINE = 0,
    SECONDARY_COMMAND_BUFFERS = 1,
}
