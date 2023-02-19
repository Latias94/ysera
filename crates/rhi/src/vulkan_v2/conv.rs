use crate::types_v2::{RHICommandPoolCreateFlags, RHIFormat};
use ash::vk;

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
        if value == vk::Format::B8G8R8A8_UNORM {
            RHIFormat::B8G8R8A8_UNORM
        } else if value == vk::Format::B8G8R8A8_SRGB {
            RHIFormat::B8G8R8A8_SRGB
        } else {
            log::error!("Cannot find related RHIFormat from {:?}", value);
            RHIFormat::UNDEFINED
        }
    }
}

impl RHIFormat {
    pub fn to_vk(self) -> vk::Format {
        if self == RHIFormat::B8G8R8A8_UNORM {
            vk::Format::B8G8R8A8_UNORM
        } else if self == RHIFormat::B8G8R8A8_SRGB {
            vk::Format::B8G8R8A8_SRGB
        } else {
            log::error!("Cannot find related RHIFormat from {:?}", self);
            vk::Format::B8G8R8A8_SRGB
        }
    }
}
