#![allow(clippy::missing_safety_doc)]

use log::LevelFilter;
use std::ffi::CStr;
use typed_builder::TypedBuilder;

mod error;
pub mod vulkan;
use crate::vulkan::instance::InstanceFlags;
pub use error::*;

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

#[derive(Debug, Default)]
pub struct QueueFamilyIndices {
    graphics_family: Option<u32>,
    present_family: Option<u32>,
    compute_family: Option<u32>,
    transfer_family: Option<u32>,
}
impl QueueFamilyIndices {
    pub fn is_complete(&self, requirements: &AdapterRequirements) -> bool {
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
}
