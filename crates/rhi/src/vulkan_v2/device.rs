use crate::vulkan_v2::debug::DebugUtils;

pub struct Device {
    /// Loads device local functions.
    raw: ash::Device,
    debug_utils: Option<DebugUtils>,
}

impl Device {
    pub fn raw(&self) -> &ash::Device {
        &self.raw
    }

    pub fn new(raw: ash::Device, debug_utils: Option<DebugUtils>) -> Self {
        Self { raw, debug_utils }
    }
}
