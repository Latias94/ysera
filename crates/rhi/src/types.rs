use crate::vulkan::instance::InstanceFlags;
use crate::GraphicsApi;
use alloc::borrow::Cow;
use log::LevelFilter;
use std::ffi::CStr;
use std::ops::RangeInclusive;
use typed_builder::TypedBuilder;

#[derive(Debug)]
pub struct ExposedAdapter<Api: GraphicsApi> {
    pub physical_device: Api::PhysicalDevice,
    pub info: AdapterInfo,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterInfo {
    pub name: String,
    /// Vendor PCI id of the physical device
    ///
    /// If the vendor has no PCI id, then this value will be the backend's vendor id equivalent. On Vulkan,
    /// Mesa would have a vendor id equivalent to it's `VkVendorId` value.
    pub vendor: usize,
    /// PCI id of the physical device
    pub device: usize,
    /// Type of device
    pub device_type: DeviceType,
    /// Driver name
    pub driver: String,
    /// Driver info
    pub driver_info: String,
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

pub type Label<'a> = Option<&'a str>;

#[derive(Debug, TypedBuilder)]
pub struct AdapterRequirements {
    #[builder(default = true)]
    pub graphics: bool,
    #[builder(default = true)]
    pub present: bool,
    #[builder(default = false)]
    pub compute: bool,
    #[builder(default = true)]
    pub transfer: bool,
    #[builder(default = true)]
    pub sampler_anisotropy: bool,
    #[builder(default = true)]
    pub sample_rate_shading: bool,
    #[builder(default = true)]
    pub discrete_gpu: bool,
    pub adapter_extension_names: Vec<&'static CStr>,
}

#[derive(Clone, Debug, TypedBuilder)]
pub struct InstanceDescriptor<'a> {
    #[builder(default)]
    pub name: &'a str,
    #[builder(default = InstanceFlags::all())]
    pub flags: InstanceFlags,
    #[builder(default = log::LevelFilter::Warn)]
    pub debug_level_filter: LevelFilter,
}

#[derive(Debug, Clone)]
pub struct SurfaceCapabilities {
    /// List of supported texture formats.
    ///
    /// Must be at least one.
    pub formats: Vec<ImageFormat>,
    /// Range for the swap chain sizes.
    ///
    /// - `swap_chain_sizes.start` must be at least 1.
    /// - `swap_chain_sizes.end` must be larger or equal to `swap_chain_sizes.start`.
    pub swap_chain_sizes: RangeInclusive<u32>,
    /// Current extent of the surface, if known.
    pub current_extent: Option<Extent3d>,
    /// Range of supported extents.
    ///
    /// `current_extent` must be inside this range.
    pub extents: RangeInclusive<Extent3d>,

    /// List of supported V-sync modes.
    ///
    /// Must be at least one.
    pub present_modes: Vec<PresentMode>,
    /// Supported texture usage flags.
    ///
    /// Must have at least `TextureUses::COLOR_TARGET`
    pub usage: TextureUses,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Extent3d {
    pub width: u32,
    pub height: u32,
    pub depth_or_array_layers: u32,
}

#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkFormat.html>"]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum ImageFormat {
    Bgra8UnormSrgb,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PresentMode {
    AutoVsync = 0,
    AutoNoVsync = 1,
    Fifo = 2,
    FifoRelaxed = 3,
    Immediate = 4,
    Mailbox = 5,
}

bitflags::bitflags! {
    pub struct TextureUses: u16 {
        /// The texture is in unknown state.
        const UNINITIALIZED = 1 << 0;
        /// Ready to present image to the surface.
        const PRESENT = 1 << 1;
        /// The source of a hardware copy.
        const COPY_SRC = 1 << 2;
        /// The destination of a hardware copy.
        const COPY_DST = 1 << 3;
        /// Read-only sampled or fetched resource.
        const RESOURCE = 1 << 4;
        /// The color target of a renderpass.
        const COLOR_TARGET = 1 << 5;
        /// Read-only depth stencil usage.
        const DEPTH_STENCIL_READ = 1 << 6;
        /// Read-write depth stencil usage
        const DEPTH_STENCIL_WRITE = 1 << 7;
    }
}

#[derive(Clone, Debug)]
pub struct BufferDescriptor<'a> {
    pub label: Option<&'a str>,
    pub size: u64,
    pub usage: BufferUses,
    pub memory_flags: MemoryPropertyFlags,
}

bitflags::bitflags! {
    #[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkBufferUsageFlagBits.html>"]
    /// For internal use.
    pub struct BufferUses: u16 {
        /// The argument to a read-only mapping.
        const MAP_READ = 1 << 0;
        /// The argument to a write-only mapping.
        const MAP_WRITE = 1 << 1;
        /// The source of a hardware copy.
        const COPY_SRC = 1 << 2;
        /// The destination of a hardware copy.
        const COPY_DST = 1 << 3;
        /// The index buffer used for drawing.
        const INDEX = 1 << 4;
        /// A vertex buffer used for drawing.
        const VERTEX = 1 << 5;
        /// A uniform buffer bound in a bind group.
        const UNIFORM = 1 << 6;
    }
}

bitflags::bitflags! {
    #[doc = "<https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkMemoryPropertyFlagBits.html>"]
    pub struct MemoryPropertyFlags: u16 {
        /// If otherwise stated, then allocate memory on device
        const DEVICE_LOCAL = 1 << 0;
        /// Memory is mappable by host
        const HOST_VISIBLE = 1 << 1;
        /// Memory will have i/o coherency. If not set, application may need to use
        /// vkFlushMappedMemoryRanges and vkInvalidateMappedMemoryRanges to flush/invalidate host cache
        const HOST_COHERENT = 1 << 2;
        /// Memory will be cached by the host
        const HOST_CACHED = 1 << 3;
        /// Memory may be allocated by the driver when it is required
        const LAZILY_ALLOCATED = 1 << 4;
    }
}

pub struct ShaderModuleDescriptor<'a> {
    pub label: Label<'a>,
    pub runtime_checks: bool,
}

#[allow(clippy::large_enum_variant)]
pub enum ShaderInput<'a> {
    Naga(NagaShader),
    SpirV(&'a [u32]),
}

/// Naga shader module.
pub struct NagaShader {
    /// Shader module IR.
    pub module: Cow<'static, naga::Module>,
    /// Analysis information of the module.
    pub info: naga::valid::ModuleInfo,
}

#[derive(Debug, Clone)]
pub struct SurfaceConfiguration {
    /// Number of textures in the swap chain.
    pub swap_chain_size: u32,
    /// Vertical synchronization mode.
    pub present_mode: PresentMode,
    /// Format of the surface textures.
    pub format: ImageFormat,
    /// Requested texture extent.
    pub extent: Extent3d,
    /// Allowed usage of surface textures,
    pub usage: TextureUses,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct QueueFamilyIndices {
    pub(crate) graphics_family: Option<u32>,
    pub(crate) present_family: Option<u32>,
    pub(crate) compute_family: Option<u32>,
    pub(crate) transfer_family: Option<u32>,
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

#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}
