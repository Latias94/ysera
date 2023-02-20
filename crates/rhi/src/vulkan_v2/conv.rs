use crate::types_v2::{RHICommandPoolCreateFlags, RHIFormat};
use ash::vk;
use num_traits::{FromPrimitive, ToPrimitive};

impl RHICommandPoolCreateFlags {
    pub fn to_vk(&self) -> vk::CommandPoolCreateFlags {
        let mut flags = vk::CommandPoolCreateFlags::empty();
        if self.contains(RHICommandPoolCreateFlags::TRANSIENT) {
            flags |= vk::CommandPoolCreateFlags::TRANSIENT;
        }
        if self.contains(RHICommandPoolCreateFlags::RESET_COMMAND_BUFFER) {
            flags |= vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER;
        }
        flags
    }
}

impl From<vk::Format> for RHIFormat {
    fn from(value: vk::Format) -> Self {
        match RHIFormat::from_i32(value.as_raw()) {
            None => RHIFormat::UNDEFINED,
            Some(x) => x,
        }
    }
}

impl RHIFormat {
    pub fn to_vk(self) -> vk::Format {
        match self.to_i32() {
            None => vk::Format::UNDEFINED,
            Some(x) => vk::Format::from_raw(x),
        }
    }
}
