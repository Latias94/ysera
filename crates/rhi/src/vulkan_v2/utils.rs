use crate::vulkan::surface::Surface;
use crate::QueueFamilyIndices;
use ash::vk;
use std::ffi::CStr;
use std::os::raw::c_char;

/// Helper function to convert [c_char; SIZE] to string
pub fn vk_to_string(raw_string_array: &[c_char]) -> String {
    // Implementation 1
    //    let end = '\0' as u8;
    //
    //    let mut content: Vec<u8> = vec![];
    //
    //    for ch in raw_string_array.iter() {
    //        let ch = (*ch) as u8;
    //
    //        if ch != end {
    //            content.push(ch);
    //        } else {
    //            break
    //        }
    //    }
    //
    //    String::from_utf8(content)
    //        .expect("Failed to convert vulkan raw string")

    // Implementation 2
    let raw_string = unsafe {
        let pointer = raw_string_array.as_ptr();
        CStr::from_ptr(pointer)
    };

    raw_string
        .to_str()
        .expect("Failed to convert vulkan raw string.")
        .to_owned()
}

pub fn get_queue_family_indices(
    instance: &ash::Instance,
    adapter: vk::PhysicalDevice,
    surface: &Surface,
) -> Result<QueueFamilyIndices, crate::DeviceError> {
    let queue_families = unsafe { instance.get_physical_device_queue_family_properties(adapter) };
    let mut indices = QueueFamilyIndices::default();
    for (i, queue_family) in queue_families.iter().enumerate() {
        if indices.is_complete() {
            break;
        }
        let index = i as u32;
        if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
            indices.graphics_family = Some(index);
        };
        if queue_family.queue_flags.contains(vk::QueueFlags::COMPUTE) {
            indices.compute_family = Some(index);
        };
        if queue_family.queue_flags.contains(vk::QueueFlags::TRANSFER) {
            indices.transfer_family = Some(index);
        };
        let support_present = unsafe {
            surface
                .loader()
                .get_physical_device_surface_support(adapter, index, surface.raw())
                .map_err(crate::DeviceError::VulkanError)?
        };

        if support_present {
            indices.present_family = Some(index);
        }
    }
    Ok(indices)
}
