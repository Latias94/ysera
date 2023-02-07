#![allow(clippy::missing_safety_doc)]

extern crate alloc;
extern crate core;

use core::fmt::Debug;

pub use ash;
use typed_builder::TypedBuilder;
pub use winit;

use crate::vulkan::device::DeviceFeatures;
pub use error::*;

mod error;
mod gui;
pub mod vulkan;

const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub type Label<'a> = Option<&'a str>;

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

#[derive(Debug, TypedBuilder)]
pub struct DeviceRequirements<'a> {
    /// extension except swapchain ext
    pub required_extension: &'a [&'a str],
    /// Set to false for headless rendering to omit the swapchain device extensions
    #[builder(default = true)]
    pub use_swapchain: bool,
}

bitflags::bitflags! {
    pub struct InstanceFlags: u16 {
        const DEBUG = 1 << 0;
        const VALIDATION = 1 << 1;
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
    #[builder(default = ash::vk::API_VERSION_1_3)]
    pub vulkan_version: u32,
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

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Extent3d {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}
impl Extent3d {
    pub fn new(width: u32, height: u32, depth: u32) -> Self {
        Self {
            width,
            height,
            depth,
        }
    }
}
