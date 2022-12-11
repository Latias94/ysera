use std::borrow::Cow;
use std::ffi::CStr;
use std::os::raw::c_void;

use ash::{extensions::ext, vk};

use crate::vulkan::utils;

#[derive(Clone)]
pub struct DebugUtils {
    pub extension: ext::DebugUtils,
    pub messenger: vk::DebugUtilsMessengerEXT,
}

unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> vk::Bool32 {
    let log_level = match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => log::Level::Debug,
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => log::Level::Info,
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => log::Level::Warn,
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => log::Level::Error,
        _ => log::Level::Warn,
    };

    let types = match message_type {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "[General]",
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[Performance]",
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[Validation]",
        _ => "[Unknown]",
    };
    let callback_data = *p_callback_data;
    let message_id_number: i32 = callback_data.message_id_number as i32;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    log::log!(
        log_level,
        "{} [{} ({})] : {}",
        types,
        message_id_name,
        &message_id_number.to_string(),
        message
    );

    vk::FALSE
}

pub struct ValidationInfo {
    pub is_enable: bool,
    pub required_validation_layers: [&'static str; 1],
}

pub fn check_validation_layer_support(
    entry: &ash::Entry,
    required_validation_layers: &[&str],
) -> bool {
    // if support validation layer, then return true

    let layer_properties = entry
        .enumerate_instance_layer_properties()
        .expect("Failed to enumerate Instance Layers Properties.");

    if layer_properties.is_empty() {
        log::error!("No available layers.");
        return false;
    }

    for required_layer_name in required_validation_layers.iter() {
        let mut is_layer_found = false;

        for layer_property in layer_properties.iter() {
            let test_layer_name = utils::vk_to_string(&layer_property.layer_name);
            if (*required_layer_name) == test_layer_name {
                is_layer_found = true;
                break;
            }
        }

        if !is_layer_found {
            return false;
        }
    }

    true
}

pub fn setup_debug_utils(
    entry: &ash::Entry,
    instance: &ash::Instance,
    min_level: vk::DebugUtilsMessageSeverityFlagsEXT,
) -> Result<(ash::extensions::ext::DebugUtils, vk::DebugUtilsMessengerEXT), crate::InstanceError> {
    let debug_utils_loader = ash::extensions::ext::DebugUtils::new(entry, instance);

    let messenger_ci = populate_debug_messenger_create_info(min_level);

    let utils_messenger = unsafe {
        debug_utils_loader
            .create_debug_utils_messenger(&messenger_ci, None)
            .map_err(crate::InstanceError::VulkanError)?
    };
    log::debug!(
        "Vulkan debug utils messenger created with log level: {:?}",
        min_level
    );

    Ok((debug_utils_loader, utils_messenger))
}

pub fn populate_debug_messenger_create_info(
    min_level: vk::DebugUtilsMessageSeverityFlagsEXT,
) -> vk::DebugUtilsMessengerCreateInfoEXT {
    let mut severity = vk::DebugUtilsMessageSeverityFlagsEXT::ERROR;
    if min_level <= vk::DebugUtilsMessageSeverityFlagsEXT::WARNING {
        severity |= vk::DebugUtilsMessageSeverityFlagsEXT::WARNING;
    }
    if min_level <= vk::DebugUtilsMessageSeverityFlagsEXT::INFO {
        severity |= vk::DebugUtilsMessageSeverityFlagsEXT::INFO;
    }
    if min_level <= vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE {
        severity |= vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE;
    }

    vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .flags(vk::DebugUtilsMessengerCreateFlagsEXT::empty())
        .message_severity(severity)
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        )
        .pfn_user_callback(Some(vulkan_debug_utils_callback))
        .build()
}
