use super::device::Device;
use super::instance::Instance;
use super::surface::Surface;
use super::swapchain::Swapchain;
use crate::vulkan::swapchain::SwapchainDescriptor;
use crate::vulkan::utils;
use crate::{AdapterRequirements, DeviceError, InstanceDescriptor};
use ash::vk;
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::rc::Rc;
use winit::window::Window;

pub struct VulkanRenderer {
    instance: Rc<Instance>,
    surface: Rc<Surface>,
    device: Rc<Device>,
    allocator: Option<Rc<Allocator>>,
    swapchain: Option<Swapchain>,
    // format: vk::SurfaceFormatKHR,
    // present_mode: vk::PresentModeKHR,
}

impl VulkanRenderer {
    pub fn new(window: &Window) -> Result<Self, DeviceError> {
        let instance_desc = InstanceDescriptor::builder()
            // .debug_level_filter(log::LevelFilter::Info)
            .build();
        let instance = unsafe { Instance::init(&instance_desc).unwrap() };
        let mut surface = unsafe {
            instance
                .create_surface(window.raw_display_handle(), window.raw_window_handle())
                .unwrap()
        };
        let adapters = unsafe { instance.enumerate_adapters().unwrap() };
        assert!(!adapters.is_empty());

        let requirements = AdapterRequirements::builder()
            .compute(true)
            .adapter_extension_names(vec![])
            .build();
        let mut selected_adapter = None;
        for adapter in adapters {
            if unsafe { adapter.meet_requirements(instance.raw(), &surface, &requirements) }.is_ok()
            {
                selected_adapter = Some(adapter);
                break;
            }
        }
        let adapter = match selected_adapter {
            None => panic!("Cannot find the require device."),
            Some(adapter) => adapter,
        };
        log::info!("Find the require device.");
        let debug_utils = instance.debug_utils().clone();

        let indices = utils::get_queue_family_indices(instance.raw(), adapter.raw(), &surface)?;
        indices.log_debug();

        let device = unsafe {
            adapter
                .open(&instance, indices, &requirements, debug_utils)
                .unwrap()
        };
        log::info!("Device opened.");

        let allocator = Allocator::new(&AllocatorCreateDesc {
            instance: instance.raw().clone(),
            device: device.raw().clone(),
            physical_device: adapter.raw(),
            debug_settings: Default::default(),
            buffer_device_address: true, // Ideally, check the BufferDeviceAddressFeatures struct.
        });

        let mut allocator = match allocator {
            Ok(x) => x,
            Err(e) => {
                log::error!("gpu-allocator allocator create failed!");
                panic!("{e}");
            }
        };

        // this queue should support graphics and present
        let queue = device.get_device_queue(indices.graphics_family.unwrap(), 0);
        let device = Rc::new(device);
        let inner_size = window.inner_size();
        let swapchain_desc = SwapchainDescriptor {
            adapter: &adapter,
            surface: &surface,
            instance: &instance,
            device: &device,
            queue,
            queue_family: indices,
            dimensions: [inner_size.width, inner_size.height],
        };
        let swapchain = Swapchain::new(&swapchain_desc)?;

        Ok(Self {
            instance: Rc::new(instance),
            surface: Rc::new(surface),
            device,
            allocator: Some(Rc::new(allocator)),
            swapchain: Some(swapchain),
        })
    }
}
