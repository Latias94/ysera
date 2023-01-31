use crate::vulkan_v2::Api;
use crate::{DeviceError, OpenDevice};
use ash::vk;

#[derive(Default)]
pub struct PhysicalDeviceCapabilities {
    pub supported_extensions: Vec<vk::ExtensionProperties>,
    pub properties: vk::PhysicalDeviceProperties,
    pub driver: Option<vk::PhysicalDeviceDriverPropertiesKHR>,
}

// This is safe because the structs have `p_next: *mut c_void`, which we null out/never read.
unsafe impl Send for PhysicalDeviceCapabilities {}
unsafe impl Sync for PhysicalDeviceCapabilities {}

impl crate::PhysicalDevice<super::Api> for super::PhysicalDevice {
    unsafe fn open(&self) -> Result<OpenDevice<Api>, DeviceError> {
        todo!()
    }
}
