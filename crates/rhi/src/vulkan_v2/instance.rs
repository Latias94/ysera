use crate::vulkan_v2::debug::DebugUtils;
use crate::vulkan_v2::{debug, platforms, utils};
use crate::{Backend, DeviceType, ExposedPhysicalDevice, InstanceDescriptor, InstanceError};
use ash::extensions::khr;
use ash::vk;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};
use std::ffi::{c_void, CStr, CString};
use std::sync::Arc;

impl crate::Instance<super::Api> for super::Instance {
    unsafe fn init(desc: &InstanceDescriptor) -> Result<Self, InstanceError> {
        let entry = ash::Entry::linked();

        let app_name = CString::new(desc.name).unwrap();
        let engine_name = CString::new("Eureka Engine").unwrap();
        let app_info = vk::ApplicationInfo::builder()
            .application_version(vk::make_api_version(0, 1, 0, 0))
            .engine_version(vk::make_api_version(0, 1, 0, 0))
            .application_name(app_name.as_c_str())
            .engine_name(engine_name.as_c_str())
            .api_version(vk::API_VERSION_1_3)
            .build();

        let enable_validation = desc.flags.contains(crate::InstanceFlags::VALIDATION);
        let mut required_layers = vec![];
        if enable_validation {
            required_layers.push("VK_LAYER_KHRONOS_validation");
        }
        let enable_debug = desc.flags.contains(crate::InstanceFlags::DEBUG);
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
            .enabled_extension_names(extension_names.as_slice())
            .build();

        log::debug!("Creating Vulkan instance...");
        let instance: ash::Instance = entry
            .create_instance(&create_info, None)
            .expect("Failed to create instance!");

        Self::from_raw(
            entry,
            instance,
            extension_cstr_names,
            desc.flags,
            desc.debug_level_filter,
        )
    }

    unsafe fn create_surface(
        &self,
        display_handle: RawDisplayHandle,
        window_handle: RawWindowHandle,
    ) -> Result<super::Surface, InstanceError> {
        match (window_handle, display_handle) {
            #[cfg(windows)]
            (RawWindowHandle::Win32(handle), _) => {
                self.create_surface_from_hwnd(handle.hinstance, handle.hwnd)
            }
            (RawWindowHandle::Wayland(handle), RawDisplayHandle::Wayland(display)) => {
                self.create_surface_from_wayland(display.display, handle.surface)
            }
            (RawWindowHandle::Xlib(handle), RawDisplayHandle::Xlib(display)) => {
                self.create_surface_from_xlib(display.display as *mut _, handle.window)
            }
            (RawWindowHandle::Xcb(handle), RawDisplayHandle::Xcb(display)) => {
                self.create_surface_from_xcb(display.connection, handle.window)
            }
            (_, _) => Err(InstanceError::NotSupport()),
        }
    }

    unsafe fn destroy_surface(&self, surface: super::Surface) {
        surface.shared.fp.destroy_surface(surface.shared.raw, None);
    }

    unsafe fn enumerate_physical_devices(
        &self,
        surface: &super::Surface,
    ) -> Vec<ExposedPhysicalDevice<super::Api>> {
        let instance = &self.shared.raw;
        let physical_devices = instance
            .enumerate_physical_devices()
            .expect("Failed to enumerate Physical Devices!");
        log::info!(
            "{} devices (GPU) found with vulkan support.",
            physical_devices.len()
        );

        let mut result = vec![];
        for &each_device in physical_devices.iter() {
            let (phd_capabilities, phd_features) =
                self.shared.get_physical_device_information(each_device);

            let (features, downlevel_flags) = phd_features.get_features_and_downlevel_flags(
                instance,
                each_device,
                &phd_capabilities,
            );

            let physical_device = super::PhysicalDevice {
                raw: each_device,
                instance: self.shared.clone(),
                surface: Some(surface.shared.clone()),
                known_memory_flags: vk::MemoryPropertyFlags::HOST_COHERENT
                    | vk::MemoryPropertyFlags::HOST_VISIBLE
                    | vk::MemoryPropertyFlags::DEVICE_LOCAL,
                phd_capabilities,
                downlevel_flags,
            };

            let vk_device_type = physical_device.phd_capabilities.properties.device_type;
            let device_type = match vk_device_type {
                vk::PhysicalDeviceType::OTHER => DeviceType::Other,
                vk::PhysicalDeviceType::INTEGRATED_GPU => DeviceType::IntegratedGpu,
                vk::PhysicalDeviceType::DISCRETE_GPU => DeviceType::DiscreteGpu,
                vk::PhysicalDeviceType::VIRTUAL_GPU => DeviceType::VirtualGpu,
                vk::PhysicalDeviceType::CPU => DeviceType::Cpu,
                _ => DeviceType::Other,
            };

            let properties = physical_device.phd_capabilities.properties;
            let device_name = utils::vk_to_string(&properties.device_name);

            let info = crate::PhysicalDeviceInfo {
                name: device_name,
                vendor: properties.vendor_id as usize,
                device_type,
                device: properties.device_id as usize,
                backend: Backend::Vulkan,
                driver: properties.driver_version.to_string(),
                driver_info: "".to_string(),
            };

            self.log_physical_device_information(each_device);

            let expose_physical_device = ExposedPhysicalDevice {
                physical_device,
                info,
            };
            result.push(expose_physical_device);
        }

        result
    }
}

impl super::Instance {
    pub unsafe fn from_raw(
        entry: ash::Entry,
        raw_instance: ash::Instance,
        extensions: Vec<&'static CStr>,
        flags: crate::InstanceFlags,
        debug_level_filter: log::LevelFilter,
    ) -> Result<Self, InstanceError> {
        let debug_utils: Option<DebugUtils> =
            if extensions.contains(&ash::extensions::ext::DebugUtils::name()) {
                log::info!("Enabling debug utils");
                let vk_msg_max_level = match debug_level_filter {
                    log::LevelFilter::Error => vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
                    log::LevelFilter::Warn => vk::DebugUtilsMessageSeverityFlagsEXT::WARNING,
                    log::LevelFilter::Info => vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
                    log::LevelFilter::Trace | log::LevelFilter::Debug => {
                        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                    }
                    _ => vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
                };
                let (extension, messenger) =
                    debug::setup_debug_utils(&entry, &raw_instance, vk_msg_max_level)?;
                Some(DebugUtils {
                    extension,
                    messenger,
                })
            } else {
                None
            };

        log::info!("Vulkan instance created.");
        Ok(Self {
            shared: Arc::new(super::InstanceShared {
                raw: raw_instance,
                entry,
                extensions,
                debug_utils,
                flags,
            }),
        })
    }

    #[allow(dead_code)]
    fn create_surface_from_hwnd(
        &self,
        hinstance: *mut c_void,
        hwnd: *mut c_void,
    ) -> Result<super::Surface, InstanceError> {
        if !self.shared.extensions.contains(&khr::Win32Surface::name()) {
            log::warn!("Vulkan driver does not support VK_KHR_WIN32_SURFACE");
            return Err(InstanceError::NotSupport());
        }

        let surface = {
            let info = vk::Win32SurfaceCreateInfoKHR::builder()
                .flags(vk::Win32SurfaceCreateFlagsKHR::empty())
                .hinstance(hinstance)
                .hwnd(hwnd);
            let win32_loader = khr::Win32Surface::new(&self.shared.entry, &self.shared.raw);
            unsafe {
                win32_loader
                    .create_win32_surface(&info, None)
                    .expect("Unable to create Win32 surface")
            }
        };

        Ok(self.create_surface_from_vk_surface_khr(surface))
    }

    #[allow(dead_code)]
    fn create_surface_from_xlib(
        &self,
        dpy: *mut vk::Display,
        window: vk::Window,
    ) -> Result<super::Surface, InstanceError> {
        if !self.shared.extensions.contains(&khr::XlibSurface::name()) {
            log::warn!("Vulkan driver does not support VK_KHR_xlib_surface");
            return Err(InstanceError::NotSupport());
        }

        let surface = {
            let xlib_loader = khr::XlibSurface::new(&self.shared.entry, &self.shared.raw);
            let info = vk::XlibSurfaceCreateInfoKHR::builder()
                .flags(vk::XlibSurfaceCreateFlagsKHR::empty())
                .window(window)
                .dpy(dpy);

            unsafe { xlib_loader.create_xlib_surface(&info, None) }
                .expect("XlibSurface::create_xlib_surface() failed")
        };

        Ok(self.create_surface_from_vk_surface_khr(surface))
    }

    #[allow(dead_code)]
    fn create_surface_from_xcb(
        &self,
        connection: *mut vk::xcb_connection_t,
        window: vk::xcb_window_t,
    ) -> Result<super::Surface, InstanceError> {
        if !self.shared.extensions.contains(&khr::XcbSurface::name()) {
            log::warn!("Vulkan driver does not support VK_KHR_xcb_surface");
            return Err(InstanceError::NotSupport());
        }

        let surface = {
            let xcb_loader = khr::XcbSurface::new(&self.shared.entry, &self.shared.raw);
            let info = vk::XcbSurfaceCreateInfoKHR::builder()
                .flags(vk::XcbSurfaceCreateFlagsKHR::empty())
                .window(window)
                .connection(connection);

            unsafe { xcb_loader.create_xcb_surface(&info, None) }
                .expect("XcbSurface::create_xcb_surface() failed")
        };

        Ok(self.create_surface_from_vk_surface_khr(surface))
    }

    #[allow(dead_code)]
    fn create_surface_from_wayland(
        &self,
        display: *mut c_void,
        surface: *mut c_void,
    ) -> Result<super::Surface, InstanceError> {
        if !self
            .shared
            .extensions
            .contains(&khr::WaylandSurface::name())
        {
            log::debug!("Vulkan driver does not support VK_KHR_wayland_surface");
            return Err(InstanceError::NotSupport());
        }

        let surface = {
            let w_loader = khr::WaylandSurface::new(&self.shared.entry, &self.shared.raw);
            let info = vk::WaylandSurfaceCreateInfoKHR::builder()
                .flags(vk::WaylandSurfaceCreateFlagsKHR::empty())
                .display(display)
                .surface(surface);

            unsafe { w_loader.create_wayland_surface(&info, None) }.expect("WaylandSurface failed")
        };

        Ok(self.create_surface_from_vk_surface_khr(surface))
    }

    fn create_surface_from_vk_surface_khr(&self, surface: vk::SurfaceKHR) -> super::Surface {
        let fp = khr::Surface::new(&self.shared.entry, &self.shared.raw);
        super::Surface {
            shared: Arc::new(super::SurfaceShared { raw: surface, fp }),
            instance: Arc::clone(&self.shared),
            swapchain: None,
        }
    }

    fn log_physical_device_information(&self, physical_device: vk::PhysicalDevice) {
        let instance = &self.shared.raw;

        let device_properties = unsafe { instance.get_physical_device_properties(physical_device) };
        let device_features = unsafe { instance.get_physical_device_features(physical_device) };
        let device_queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let device_type = match device_properties.device_type {
            vk::PhysicalDeviceType::CPU => "Cpu",
            vk::PhysicalDeviceType::INTEGRATED_GPU => "Integrated GPU",
            vk::PhysicalDeviceType::DISCRETE_GPU => "Discrete GPU",
            vk::PhysicalDeviceType::VIRTUAL_GPU => "Virtual GPU",
            vk::PhysicalDeviceType::OTHER => "Unknown",
            _ => panic!(),
        };
        let device_name = utils::vk_to_string(&device_properties.device_name);
        log::debug!(
            "\tDevice Name: {}, id: {}, type: {}",
            device_name,
            device_properties.device_id,
            device_type
        );
        let major_version = vk::api_version_major(device_properties.api_version);
        let minor_version = vk::api_version_minor(device_properties.api_version);
        let patch_version = vk::api_version_patch(device_properties.api_version);
        log::debug!(
            "\tAPI Version: {}.{}.{}",
            major_version,
            minor_version,
            patch_version
        );
        log::debug!("\tSupport Queue Family: {}", device_queue_families.len());
        log::debug!(
            "\t\tQueue Count | {: ^10} | {: ^10} | {: ^10} | {: ^15}",
            "Graphics",
            "Compute",
            "Transfer",
            "Sparse Binding"
        );

        for queue_family in device_queue_families.iter() {
            let is_graphics_support = if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            {
                "support"
            } else {
                "unsupport"
            };
            let is_compute_support = if queue_family.queue_flags.contains(vk::QueueFlags::COMPUTE) {
                "support"
            } else {
                "unsupport"
            };
            let is_transfer_support = if queue_family.queue_flags.contains(vk::QueueFlags::TRANSFER)
            {
                "support"
            } else {
                "unsupport"
            };
            let is_sparse_support = if queue_family
                .queue_flags
                .contains(vk::QueueFlags::SPARSE_BINDING)
            {
                "support"
            } else {
                "unsupport"
            };
            log::debug!(
                "\t\t{}\t    | {: ^10} | {: ^10} | {: ^10} | {: ^15}",
                queue_family.queue_count,
                is_graphics_support,
                is_compute_support,
                is_transfer_support,
                is_sparse_support
            );
        }
        // there are plenty of features
        log::debug!(
            "\tGeometry Shader support: {}",
            if device_features.geometry_shader == 1 {
                "Support"
            } else {
                "Unsupport"
            }
        );
        log::debug!(
            "\tTessellation Shader support: {}",
            if device_features.tessellation_shader == 1 {
                "Support"
            } else {
                "Unsupport"
            }
        );
    }
}
