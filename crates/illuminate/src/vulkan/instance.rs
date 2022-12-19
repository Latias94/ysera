use super::debug::DebugUtils;
use super::{adapter::Adapter, surface::Surface};
use crate::vulkan::{debug, platforms};
use crate::{InstanceDescriptor, InstanceError};
use ash::{extensions::*, vk};
use log::LevelFilter;
use raw_window_handle::RawWindowHandle;
use std::ffi::{c_void, CStr, CString};

bitflags::bitflags! {
    pub struct InstanceFlags: u16 {
        const DEBUG = 1 << 0;
        const VALIDATION = 1 << 1;
    }
}

pub struct Instance {
    /// Loads instance level functions. Needs to outlive the Devices it has created.
    raw: ash::Instance,
    /// Loads the Vulkan library. Needs to outlive Instance and Device.
    entry: ash::Entry,
    debug_utils: Option<DebugUtils>,
    extensions: Vec<&'static CStr>,
    flags: InstanceFlags,
}

impl Instance {
    pub fn new(
        raw: ash::Instance,
        entry: ash::Entry,
        debug_utils: Option<DebugUtils>,
        extensions: Vec<&'static CStr>,
        flags: InstanceFlags,
    ) -> Self {
        Self {
            raw,
            entry,
            debug_utils,
            extensions,
            flags,
        }
    }

    pub fn raw(&self) -> &ash::Instance {
        &self.raw
    }

    pub fn flags(&self) -> InstanceFlags {
        self.flags
    }

    pub fn debug_utils(&self) -> &Option<DebugUtils> {
        &self.debug_utils
    }

    pub unsafe fn init(desc: &InstanceDescriptor) -> Result<Self, InstanceError> {
        let entry = ash::Entry::linked();

        let app_name = CString::new(desc.name).unwrap();
        let engine_name = CString::new("Eureka Engine").unwrap();
        let app_info = vk::ApplicationInfo::builder()
            .application_version(vk::make_api_version(0, 1, 0, 0))
            .engine_version(vk::make_api_version(0, 1, 0, 0))
            .application_name(app_name.as_c_str())
            .engine_name(engine_name.as_c_str())
            .api_version(vk::API_VERSION_1_3);
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
            raw: instance,
            entry,
            debug_utils,
            extensions: extension_cstr_names,
            flags,
        })
    }

    pub fn enumerate_adapters(&self) -> Result<Vec<Adapter>, InstanceError> {
        let instance = &self.raw;
        let adapters = unsafe {
            instance
                .enumerate_physical_devices()
                .map_err(InstanceError::VulkanError)?
        };
        log::info!(
            "{} devices (GPU) found with vulkan support.",
            adapters.len()
        );

        let mut result = vec![];

        for &each_adapter in adapters.iter() {
            let adapter = Adapter::new(each_adapter);
            adapter.log_adapter_information(&self.raw);
            result.push(adapter);
        }
        Ok(result)
    }
    pub unsafe fn create_surface(
        &self,
        display_handle: raw_window_handle::RawDisplayHandle,
        window_handle: RawWindowHandle,
    ) -> Result<Surface, InstanceError> {
        let surface = match (window_handle, display_handle) {
            #[cfg(windows)]
            (RawWindowHandle::Win32(handle), _) => {
                self.create_surface_from_hwnd(handle.hinstance, handle.hwnd)
            }
            _ => todo!(),
        }?;
        Ok(surface)
    }
}

impl Instance {
    fn create_surface_from_hwnd(
        &self,
        hinstance: *mut c_void,
        hwnd: *mut c_void,
    ) -> Result<Surface, InstanceError> {
        if !self.extensions.contains(&khr::Win32Surface::name()) {
            panic!("Vulkan driver does not support `VK_KHR_WIN32_SURFACE`");
        }

        let surface = {
            let info = vk::Win32SurfaceCreateInfoKHR::builder()
                .flags(vk::Win32SurfaceCreateFlagsKHR::empty())
                .hinstance(hinstance)
                .hwnd(hwnd);
            let win32_loader = khr::Win32Surface::new(&self.entry, &self.raw);
            unsafe { win32_loader.create_win32_surface(&info, None)? }
        };

        let surface_loader = khr::Surface::new(&self.entry, &self.raw);
        Ok(Surface::new(surface, surface_loader))
    }
}
