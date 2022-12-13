#![allow(clippy::missing_safety_doc)]

extern crate core;

use log::LevelFilter;
use std::ffi::CStr;
use std::fmt::{Debug, Formatter};
use typed_builder::TypedBuilder;

mod error;
pub mod vulkan;
use crate::vulkan::instance::InstanceFlags;
pub use error::*;

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

#[derive(Debug, Default, Copy, Clone)]
pub struct QueueFamilyIndices {
    graphics_family: Option<u32>,
    present_family: Option<u32>,
    compute_family: Option<u32>,
    transfer_family: Option<u32>,
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
