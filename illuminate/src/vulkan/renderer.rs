use super::device::Device;
use super::instance::Instance;
use super::surface::Surface;
use super::swapchain::Swapchain;
use crate::vulkan::debug::DebugUtils;
use crate::vulkan::pipeline::Pipeline;
use crate::vulkan::render_pass::RenderPass;
use crate::vulkan::shader::{Shader, ShaderDescriptor};
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
    debug_utils: Option<DebugUtils>,
    present_queue: vk::Queue,
    graphics_queue: vk::Queue,
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
                .open(&instance, indices, &requirements, debug_utils.clone())
                .unwrap()
        };

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
        let graphics_queue = device.get_device_queue(indices.graphics_family.unwrap(), 0);
        let present_queue = device.get_device_queue(indices.present_family.unwrap(), 0);
        let device = Rc::new(device);
        let inner_size = window.inner_size();

        let swapchain_desc = SwapchainDescriptor {
            adapter: &adapter,
            surface: &surface,
            instance: &instance,
            device: &device,
            queue: present_queue,
            queue_family: indices,
            dimensions: [inner_size.width, inner_size.height],
        };

        let swapchain = Swapchain::new(&swapchain_desc)?;

        let shader_desc = ShaderDescriptor {
            label: Some("Triangle"),
            device: &device,
            vert_bytes: &Shader::load_pre_compiled_spv_bytes_from_name("triangle0.vert"),
            vert_entry_name: "main",
            frag_bytes: &Shader::load_pre_compiled_spv_bytes_from_name("triangle0.frag"),
            frag_entry_name: "main",
        };
        let shader = Shader::new(&shader_desc).map_err(|e| DeviceError::Other("Shader Error"))?;

        let pipeline = Pipeline::new(
            &device,
            swapchain.render_pass().raw(),
            swapchain.extent(),
            shader,
        )?;

        Ok(Self {
            instance: Rc::new(instance),
            surface: Rc::new(surface),
            device,
            allocator: Some(Rc::new(allocator)),
            swapchain: Some(swapchain),
            debug_utils,
            present_queue,
            graphics_queue,
        })
    }
}

impl Drop for VulkanRenderer {
    fn drop(&mut self) {
        self.device.wait_idle();
        self.swapchain = None; // drop first

        if let Some(DebugUtils {
            extension,
            messenger,
        }) = self.debug_utils.take()
        {
            unsafe {
                extension.destroy_debug_utils_messenger(messenger, None);
            }
        }

        unsafe {
            self.surface
                .loader()
                .destroy_surface(self.surface.raw(), None);
        }
        log::debug!("surface destroyed.");
    }
}
