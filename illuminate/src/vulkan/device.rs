use crate::vulkan::debug::DebugUtils;
use crate::DeviceError;
use ash::vk;
use std::ffi::CStr;

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

    pub fn wait_idle(&self) {
        unsafe { self.raw.device_wait_idle().unwrap() }
    }

    pub fn create_texture_view(
        &self,
        create_info: &vk::ImageViewCreateInfo,
    ) -> Result<vk::ImageView, DeviceError> {
        Ok(unsafe { self.raw.create_image_view(create_info, None)? })
    }

    pub fn destroy_texture_view(&self, image_view: vk::ImageView) {
        unsafe {
            self.raw.destroy_image_view(image_view, None);
        }
    }
    pub fn create_shader_module(
        &self,
        create_info: &vk::ShaderModuleCreateInfo,
    ) -> Result<vk::ShaderModule, DeviceError> {
        Ok(unsafe { self.raw.create_shader_module(create_info, None)? })
    }

    pub fn destroy_shader_module(&self, shader_module: vk::ShaderModule) {
        unsafe {
            self.raw.destroy_shader_module(shader_module, None);
        }
    }

    pub fn get_device_queue(&self, queue_family_index: u32, queue_index: u32) -> vk::Queue {
        unsafe { self.raw.get_device_queue(queue_family_index, queue_index) }
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