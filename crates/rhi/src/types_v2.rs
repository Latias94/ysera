use bitflags::bitflags;
use typed_builder::TypedBuilder;

pub enum RHICommandBufferLevel {
    PRIMARY,
    SECONDARY,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RHIFormat {
    UNDEFINED,
    B8G8R8A8_UNORM,
    B8G8R8A8_SRGB,
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
