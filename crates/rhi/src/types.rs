use ash::vk;
use bitflags::bitflags;
use std::hash::{Hash, Hasher};
use typed_builder::TypedBuilder;

pub type Label<'a> = Option<&'a str>;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct InstanceFlags: u16 {
        const DEBUG = 1 << 0;
        const VALIDATION = 1 << 1;
    }
}

#[derive(Debug, TypedBuilder)]
pub struct AdapterRequirements {
    // queue requirement
    #[builder(default = true)]
    pub graphics: bool,
    #[builder(default = true)]
    pub present: bool,
    #[builder(default = false)]
    pub compute: bool,
    #[builder(default = true)]
    pub transfer: bool,

    // vk::PhysicalDeviceFeatures
    #[builder(default = true)]
    pub sampler_anisotropy: bool,
    #[builder(default = true)]
    pub sample_rate_shading: bool,
    // vk::PhysicalDeviceFeatures2 12 13
    #[builder(default)]
    pub extra_features: DeviceFeatures,
    #[builder(default = true)]
    pub discrete_gpu: bool,
}

#[derive(Debug, Clone, Copy, Default, TypedBuilder)]
pub struct DeviceFeatures {
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
        (!requirements.ray_tracing_pipeline || self.ray_tracing_pipeline)
            && (!requirements.acceleration_structure || self.acceleration_structure)
            && (!requirements.runtime_descriptor_array || self.runtime_descriptor_array)
            && (!requirements.buffer_device_address || self.buffer_device_address)
            && (!requirements.dynamic_rendering || self.dynamic_rendering)
            && (!requirements.synchronization2 || self.synchronization2)
    }
}

#[derive(Debug, TypedBuilder)]
pub struct DeviceRequirements<'a> {
    /// extension except swapchain ext
    pub required_extension: &'a [&'a str],
    /// Set to false for headless rendering to omit the swapchain device extensions
    #[builder(default = true)]
    pub use_swapchain: bool,
}

#[derive(Clone, TypedBuilder)]
pub struct RenderPassDescriptor<'a> {
    pub label: Label<'a>,
    pub depth: f32,   // 1.0
    pub stencil: u32, // 0.0
    pub render_area: math::Rect2D,
    pub clear_color: Color,
    pub clear_flags: RenderPassClearFlags,
    pub attachments: &'a [RenderTargetAttachment],
}

#[derive(Clone, Debug, TypedBuilder)]
pub struct RenderTargetAttachment {
    pub format: ImageFormat,
    pub ty: RenderTargetAttachmentType,
    pub load_op: AttachmentLoadOp,
    pub store_op: AttachmentStoreOp,
}

impl Default for RenderTargetAttachment {
    fn default() -> Self {
        Self {
            format: ImageFormat::Bgra8UnormSrgb,
            ty: RenderTargetAttachmentType::Color,
            load_op: AttachmentLoadOp::Load,
            store_op: AttachmentStoreOp::Store,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RenderTargetAttachmentType {
    Color,
    Depth,
    Stencil,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct RenderPassClearFlags: u16 {
        const None = 1 << 0;
        const COLOR_BUFFER = 1 << 1;
        const DEPTH_BUFFER = 1 << 2;
        const STENCIL_BUFFER = 1 << 3;
    }
}

#[derive(Debug, Clone)]
pub enum AttachmentLoadOp {
    Clear,
    Discard,
    Load,
}

#[derive(Debug, Clone)]
pub enum AttachmentStoreOp {
    Discard,
    Store,
}

#[derive(Debug, Clone)]
pub enum ImageLayout {
    ColorInput,
    ColorOutput,
    DepthStencilReadOnly,
    DepthStencilReadWrite,
    General,
    Present,
    TransferSource,
    TransferDestination,
    Undefined,
}

bitflags! {
    #[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPipelineStageFlagBits.html>"]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct PipelineState: u16 {
        #[doc = "Before subsequent commands are processed"]
        const TOP_OF_PIPE = 1 << 0;
        #[doc = "Color attachment writes"]
        const COLOR_ATTACHMENT_OUTPUT = 1 << 1;
        #[doc = "Compute shading"]
        const COMPUTE_SHADER = 1 << 2;
        #[doc = "Fragment shading"]
        const FRAGMENT_SHADER = 1 << 3;
        #[doc = "Early fragment (depth and stencil) tests"]
        const FRAGMENT_TESTS_EARLY = 1 << 4;
        #[doc = "Late fragment (depth and stencil) tests"]
        const FRAGMENT_TESTS_LATE = 1 << 5;
        #[doc = "Transfer/copy operations"]
        const TRANSFER = 1 << 6;
    }
}

bitflags! {
    #[repr(transparent)]
    #[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkAccessFlagBits.html>"]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct AccessFlags: u16 {
        #[doc = "Controls coherency of input attachment reads"]
        const COLOR_ATTACHMENT_READ = 1 << 0;
        #[doc = "Controls coherency of color attachment writes"]
        const COLOR_ATTACHMENT_WRITE = 1 << 1;
    }
}

#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkFormat.html>"]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum ImageFormat {
    Bgra8UnormSrgb,
    Depth32Float,
    Depth24Stencil8,
}

impl ImageFormat {
    pub fn is_depth_format(&self) -> bool {
        *self == ImageFormat::Depth24Stencil8 || *self == ImageFormat::Depth32Float
    }
}

pub enum ImageUsage {
    Texture,
    Attachment,
    Storage,
}

pub enum ImageWrap {
    Clamp,
    Repeat,
}

pub enum ImageFilter {
    Linear,
    Nearest,
}

pub enum ImageType {
    D2,
    Cube,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Hash for Color {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.r.to_bits().hash(state);
        self.g.to_bits().hash(state);
        self.b.to_bits().hash(state);
        self.a.to_bits().hash(state);
    }
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::new(0f32, 0f32, 0f32, 1f32)
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterInfo {
    /// Adapter name
    pub name: String,
    /// Vendor PCI id of the adapter
    ///
    /// If the vendor has no PCI id, then this value will be the backend's vendor id equivalent. On Vulkan,
    /// Mesa would have a vendor id equivalent to it's `VkVendorId` value.
    pub vendor: usize,
    /// PCI id of the adapter
    pub device: usize,
    /// Type of device
    pub device_type: DeviceType,
    /// Driver name
    pub driver: String,
    /// Driver info
    pub driver_info: String,
    /// Backend used for device
    pub backend: Backend,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Backend {
    Empty = 0,
    Vulkan = 1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeviceType {
    /// Other or Unknown.
    Other,
    /// Integrated GPU with shared CPU/GPU memory.
    IntegratedGpu,
    /// Discrete GPU with separate CPU/GPU memory.
    DiscreteGpu,
    /// Virtual / Hosted.
    VirtualGpu,
    /// Cpu / Software Rendering.
    Cpu,
}

bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    pub struct Features: u64 {
    }
}

#[derive(Debug, Clone)]
pub struct SurfaceConfiguration {
    /// Number of textures in the swap chain. Must be in
    /// `SurfaceCapabilities::swap_chain_size` range.
    pub swap_chain_size: u32,
    /// Vertical synchronization mode.
    pub present_mode: PresentMode,
    /// Format of the surface textures.
    pub format: ImageFormat,
    pub extent: Extent3d,
}

/// Behavior of the presentation engine based on frame rate.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum PresentMode {
    AutoNoVsync = 1,
    #[default]
    Fifo = 2,
    FifoRelaxed = 3,
    Immediate = 4,
    Mailbox = 5,
}

#[derive(Debug, Clone)]
pub struct Extent3d {
    pub width: u32,
    pub height: u32,
    pub depth_or_array_layers: u32,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
    pub compute_family: Option<u32>,
    pub transfer_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn has_meet_requirement(&self, requirements: &AdapterRequirements) -> bool {
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
