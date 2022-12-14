use super::device::Device;
use super::instance::Instance;
use super::surface::Surface;
use super::swapchain::Swapchain;
use crate::vulkan::debug::DebugUtils;
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
    command_pool: vk::CommandPool,
    image_available_semaphore: vk::Semaphore,
    render_finished_semaphore: vk::Semaphore,
    // format: vk::SurfaceFormatKHR,
    // present_mode: vk::PresentModeKHR,
}

impl VulkanRenderer {
    pub fn new(window: &Window) -> Result<Self, DeviceError> {
        let instance_desc = InstanceDescriptor::builder()
            // .debug_level_filter(log::LevelFilter::Info)
            .build();
        let instance = unsafe { Instance::init(&instance_desc).unwrap() };
        let surface = unsafe {
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

        let allocator = match allocator {
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

        let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(indices.graphics_family.unwrap())
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .build();
        let command_pool = device.create_command_pool(&command_pool_create_info)?;

        let swapchain_desc = SwapchainDescriptor {
            adapter: &adapter,
            surface: &surface,
            instance: &instance,
            device: &device,
            queue_family: indices,
            dimensions: [inner_size.width, inner_size.height],
            command_pool,
            graphics_queue,
            present_queue,
            allocator: &allocator,
        };

        let swapchain = Swapchain::new(&swapchain_desc)?;

        let semaphore_create_info = vk::SemaphoreCreateInfo::builder().build();
        let image_available_semaphore = device.create_semaphore(&semaphore_create_info)?;
        let render_finished_semaphore = device.create_semaphore(&semaphore_create_info)?;

        Ok(Self {
            instance: Rc::new(instance),
            surface: Rc::new(surface),
            device,
            allocator: Some(Rc::new(allocator)),
            swapchain: Some(swapchain),
            debug_utils,
            present_queue,
            graphics_queue,
            command_pool,
            image_available_semaphore,
            render_finished_semaphore,
        })
    }

    pub fn render(&mut self) -> Result<(), DeviceError> {
        let swapchain = self.swapchain.as_mut().unwrap();
        let (image_index, _sub_optimal) =
            swapchain.acquire_next_image(u64::MAX, self.image_available_semaphore)?;
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

        let command_buffers = &[swapchain.command_buffers()[image_index as usize]];
        let signal_semaphores = &[self.render_finished_semaphore];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&[self.image_available_semaphore])
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores)
            .build();

        self.device
            .queue_submit(self.graphics_queue, &[submit_info], vk::Fence::null())?;

        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(&[swapchain.raw()])
            .image_indices(&[image_index as u32])
            .build();
        swapchain.queue_present(&present_info)?;
        Ok(())
    }
}

impl Drop for VulkanRenderer {
    fn drop(&mut self) {
        self.device.wait_idle();
        self.swapchain = None; // drop first
        self.device
            .destroy_semaphore(self.image_available_semaphore);
        self.device
            .destroy_semaphore(self.render_finished_semaphore);
        self.device.destroy_command_pool(self.command_pool);
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
