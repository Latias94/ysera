use alloc::ffi::CString;
use std::ffi::{c_void, CStr};
use std::sync::Arc;

use ash::{extensions::*, vk};
use log::LevelFilter;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use crate::types::{InstanceDescriptor, InstanceFlags};
use crate::vulkan::debug;
use crate::vulkan::debug::DebugUtils;
use crate::vulkan::platforms;
use crate::{DeviceError, InstanceError};

use super::adapter::Adapter;

pub struct Instance {
    pub(crate) shared: Arc<InstanceShared>,
}

#[derive(Clone)]
pub struct InstanceShared {
    /// Loads instance level functions. Needs to outlive the Devices it has created.
    raw: ash::Instance,
    /// Loads the Vulkan library. Needs to outlive Instance and Device.
    entry: ash::Entry,
    debug_utils: Option<DebugUtils>,
    extensions: Vec<&'static CStr>,
    flags: InstanceFlags,
}

impl InstanceShared {
    pub fn raw(&self) -> &ash::Instance {
        &self.raw
    }

    pub fn flags(&self) -> InstanceFlags {
        self.flags
    }

    pub fn debug_utils(&self) -> &Option<DebugUtils> {
        &self.debug_utils
    }
}

impl Instance {
    pub fn shared_instance(&self) -> &InstanceShared {
        &self.shared
    }

    pub unsafe fn init(desc: &InstanceDescriptor) -> Result<Self, InstanceError> {
        #[cfg(not(target_os = "macos"))]
        let vulkan_api_version = desc.vulkan_version;

        #[cfg(target_os = "macos")]
        // https://github.com/KhronosGroup/MoltenVK/issues/1567
        let vulkan_api_version = vk::API_VERSION_1_1;

        #[cfg(not(target_os = "macos"))]
        let entry = ash::Entry::linked();

        // #[cfg(target_os = "macos")]
        // let entry = ash_molten::linked();

        let app_name = CString::new(desc.name).unwrap();
        let engine_name = CString::new("Eureka Engine").unwrap();
        let app_info = vk::ApplicationInfo::builder()
            .application_version(vk::make_api_version(0, 1, 0, 0))
            .engine_version(vk::make_api_version(0, 1, 0, 0))
            .application_name(app_name.as_c_str())
            .engine_name(engine_name.as_c_str())
            .api_version(vulkan_api_version);
        let enable_validation = desc.flags.contains(InstanceFlags::VALIDATION);
        let mut required_layers = vec![];
        if enable_validation {
            required_layers.push("VK_LAYER_KHRONOS_validation");
        }
        let enable_debug = desc.flags.contains(InstanceFlags::DEBUG);
        if enable_validation
            && !debug::check_validation_layer_support(&entry, required_layers.as_slice())
        {
            log::error!("Validation layers requested, but not available!");
        }

        let required_layer_raw_names: Vec<CString> = required_layers
            .iter()
            .map(|layer_name| CString::new(*layer_name).unwrap())
            .collect();

        let enable_layer_names: Vec<*const i8> = required_layer_raw_names
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect();

        let extension_cstr_names = platforms::required_extension_names(enable_debug);
        log::debug!("Required extension:");
        let extension_names: Vec<*const i8> = extension_cstr_names
            .iter()
            .map(|x| {
                log::debug!("  * {}", x.to_str().unwrap());
                x.as_ptr()
            })
            .collect();

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_layer_names(enable_layer_names.as_slice())
            .enabled_extension_names(extension_names.as_slice());

        log::debug!("Creating Vulkan instance...");
        let instance: ash::Instance = entry
            .create_instance(&create_info, None)
            .map_err(InstanceError::VulkanError)?;

        let debug_utils: Option<DebugUtils> =
            if extension_cstr_names.contains(&ext::DebugUtils::name()) {
                log::info!("Enabling debug utils");
                let vk_msg_max_level = match desc.debug_level_filter {
                    LevelFilter::Error => vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
                    LevelFilter::Warn => vk::DebugUtilsMessageSeverityFlagsEXT::WARNING,
                    LevelFilter::Info => vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
                    LevelFilter::Trace | LevelFilter::Debug => {
                        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                    }
                    _ => vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
                };
                let (extension, messenger) =
                    debug::setup_debug_utils(&entry, &instance, vk_msg_max_level)?;
                Some(DebugUtils {
                    extension,
                    messenger,
                })
            } else {
                None
            };
        log::debug!("Vulkan instance created.");

        let flags = desc.flags;

        Ok(Self {
            shared: Arc::new(InstanceShared {
                raw: instance,
                entry,
                debug_utils,
                extensions: extension_cstr_names,
                flags,
            }),
        })
    }

    pub fn enumerate_adapters(&self, surface: &Surface) -> Result<Vec<Adapter>, DeviceError> {
        let instance = &self.shared.raw;
        let adapters = unsafe { instance.enumerate_physical_devices()? };
        log::info!(
            "{} devices (GPU) found with vulkan support.",
            adapters.len()
        );

        let mut result = vec![];

        for &each_adapter in adapters.iter() {
            let adapter = Adapter::new(each_adapter, &self, surface)?;
            adapter.log_adapter_information(&self.shared.raw);
            result.push(adapter);
        }
        Ok(result)
    }

    pub unsafe fn create_surface(
        &self,
        window_handle: &dyn HasRawWindowHandle,
        display_handle: &dyn HasRawDisplayHandle,
    ) -> Result<Surface, InstanceError> {
        let surface = ash_window::create_surface(
            &self.shared.entry,
            &self.shared.raw,
            display_handle.raw_display_handle(),
            window_handle.raw_window_handle(),
            None,
        )?;
        let surface_loader = khr::Surface::new(&self.shared.entry, &self.shared.raw);
        Ok(Surface::new(surface, surface_loader))
    }
}

impl Instance {
    #[allow(dead_code)]
    #[cfg(target_os = "windows")]
    fn create_surface_from_hwnd(
        &self,
        hinstance: *mut c_void,
        hwnd: *mut c_void,
    ) -> Result<Surface, InstanceError> {
        if !self.shared.extensions.contains(&khr::Win32Surface::name()) {
            panic!("Vulkan driver does not support `VK_KHR_WIN32_SURFACE`");
        }

        let surface = {
            let info = vk::Win32SurfaceCreateInfoKHR::builder()
                .flags(vk::Win32SurfaceCreateFlagsKHR::empty())
                .hinstance(hinstance)
                .hwnd(hwnd);
            let win32_loader = khr::Win32Surface::new(&self.shared.entry, &self.shared.raw);
            unsafe { win32_loader.create_win32_surface(&info, None)? }
        };

        let surface_loader = khr::Surface::new(&self.shared.entry, &self.shared.raw);
        Ok(Surface::new(surface, surface_loader))
    }
}

pub struct Surface {
    raw: vk::SurfaceKHR,
    loader: khr::Surface,
}

impl Surface {
    pub fn raw(&self) -> vk::SurfaceKHR {
        self.raw
    }

    pub fn loader(&self) -> &khr::Surface {
        &self.loader
    }

    pub fn new(raw: vk::SurfaceKHR, loader: khr::Surface) -> Self {
        Self { raw, loader }
    }
}
