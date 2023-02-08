use std::ffi::CStr;
use std::sync::Arc;

use ash::vk;

use crate::vulkan::adapter::{Adapter, AdapterShared};
use crate::vulkan::debug::DebugUtils;
use crate::DeviceError;

pub struct Device {
    /// Loads device local functions.
    raw: ash::Device,
    debug_utils: Option<DebugUtils>,
    adapter: Arc<AdapterShared>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DeviceFeatures {
    pub ray_tracing_pipeline: bool,
    pub acceleration_structure: bool,
    pub runtime_descriptor_array: bool,
    pub buffer_device_address: bool,
    pub dynamic_rendering: bool,
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

impl Device {
    pub(crate) fn raw(&self) -> &ash::Device {
        &self.raw
    }

    pub(crate) fn new(
        raw: ash::Device,
        debug_utils: Option<DebugUtils>,
        adapter: Arc<AdapterShared>,
    ) -> Self {
        Self {
            raw,
            debug_utils,
            adapter,
        }
    }

    pub unsafe fn wait_idle(&self) -> Result<(), DeviceError> {
        unsafe { self.raw.device_wait_idle().unwrap() }
        Ok(())
    }

    pub unsafe fn set_object_name(
        &self,
        object_type: vk::ObjectType,
        object: impl vk::Handle,
        name: &str,
    ) {
        let debug_utils = match &self.debug_utils {
            Some(utils) => utils,
            None => return,
        };

        let mut buffer: [u8; 64] = [0u8; 64];
        let buffer_vec: Vec<u8>;

        // Append a null terminator to the string
        let name_bytes = if name.len() < buffer.len() {
            // Common case, string is very small. Allocate a copy on the stack.
            buffer[..name.len()].copy_from_slice(name.as_bytes());
            // Add null terminator
            buffer[name.len()] = 0;
            &buffer[..name.len() + 1]
        } else {
            // Less common case, the string is large.
            // This requires a heap allocation.
            buffer_vec = name
                .as_bytes()
                .iter()
                .cloned()
                .chain(std::iter::once(0))
                .collect();
            &buffer_vec
        };
        let extension = &debug_utils.extension;
        let _result = extension.set_debug_utils_object_name(
            self.raw.handle(),
            &vk::DebugUtilsObjectNameInfoEXT::builder()
                .object_type(object_type)
                .object_handle(object.as_raw())
                .object_name(CStr::from_bytes_with_nul_unchecked(name_bytes)),
        );
    }
}
