use super::device::Device;
use super::instance::Instance;
use super::surface::Surface;
use super::swapchain::Swapchain;
use crate::vulkan::adapter::Adapter;
use crate::vulkan::command_buffer_allocator::CommandBufferAllocator;
use crate::vulkan::debug::DebugUtils;
use crate::vulkan::swapchain::SwapchainDescriptor;
use crate::vulkan::utils;
use crate::{
    AdapterRequirements, DeviceError, InstanceDescriptor, QueueFamilyIndices, SurfaceError,
};
use ash::vk;
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::rc::Rc;
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub struct VulkanRenderer {
    adapter: Rc<Adapter>,
    instance: Rc<Instance>,
    surface: Rc<Surface>,
    device: Rc<Device>,
    allocator: Rc<Allocator>,
    swapchain: Option<Swapchain>,
    debug_utils: Option<DebugUtils>,
    present_queue: vk::Queue,
    graphics_queue: vk::Queue,
    command_pool: vk::CommandPool,
    extent: vk::Extent2D,
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    indices: QueueFamilyIndices,
    command_buffer_allocator: CommandBufferAllocator,
    frame: usize,
}

const MAX_FRAMES_IN_FLIGHT: usize = 2;

impl VulkanRenderer {
    pub fn new(window: &Window) -> Result<Self, DeviceError> {
        let instance_desc = InstanceDescriptor::builder()
            // .flags(crate::vulkan::instance::InstanceFlags::empty())
            // .debug_level_filter(log::LevelFilter::Info)
            .build();
        let instance = unsafe { Instance::init(&instance_desc).unwrap() };
        let surface = unsafe {
            instance
                .create_surface(window.raw_display_handle(), window.raw_window_handle())
                .unwrap()
        };
        let adapters = instance.enumerate_adapters().unwrap();
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
        log::debug!("Find the require device.");
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

        let command_buffer_allocator =
            CommandBufferAllocator::new(&device, command_pool, graphics_queue);

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
            command_buffer_allocator: &command_buffer_allocator,
        };

        let swapchain = Swapchain::new(&swapchain_desc)?;

        let semaphore_create_info = vk::SemaphoreCreateInfo::builder().build();
        let fence_create_info = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED)
            .build();
        let mut image_available_semaphores = vec![];
        let mut render_finished_semaphores = vec![];
        let mut in_flight_fences = vec![];
        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            image_available_semaphores.push(device.create_semaphore(&semaphore_create_info)?);
            render_finished_semaphores.push(device.create_semaphore(&semaphore_create_info)?);
            in_flight_fences.push(device.create_fence(&fence_create_info)?);
        }

        Ok(Self {
            adapter: Rc::new(adapter),
            instance: Rc::new(instance),
            surface: Rc::new(surface),
            device,
            allocator: Rc::new(allocator),
            extent: swapchain.extent(),
            swapchain: Some(swapchain),
            debug_utils,
            present_queue,
            graphics_queue,
            command_pool,
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            indices,
            command_buffer_allocator,
            frame: 0,
        })
    }

    pub fn render(&mut self) -> Result<(), DeviceError> {
        if self.swapchain.is_none() {
            self.recreate_swapchain(PhysicalSize {
                width: self.extent.width,
                height: self.extent.height,
            })?;
        }

        let in_flight_fence = self.in_flight_fences[self.frame];
        let in_flight_fences = [in_flight_fence];
        self.device
            .wait_for_fence(&in_flight_fences, true, u64::MAX)?;

        let swapchain = self.swapchain.as_mut().unwrap();
        let result =
            swapchain.acquire_next_image(u64::MAX, self.image_available_semaphores[self.frame]);
        let image_index = match result {
            Ok((image_index, _)) => image_index,
            Err(SurfaceError::OutOfDate) => {
                self.swapchain = None;
                return Ok(());
            }
            Err(e) => panic!("failed to acquire_next_image. Err: {}", e),
        };
        self.device.reset_fence(&in_flight_fences)?;

        let command_buffer = swapchain.render(self.frame, image_index as usize)?;

        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

        let wait_semaphores = &[self.image_available_semaphores[self.frame]];
        let signal_semaphores = &[self.render_finished_semaphores[self.frame]];

        let command_buffers = [command_buffer];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(signal_semaphores)
            .build();

        self.device
            .queue_submit(self.graphics_queue, &[submit_info], in_flight_fence)?;
        swapchain.update_submitted_command_buffer(self.frame);

        let swapchains = [swapchain.raw()];
        let image_indices = [image_index];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices)
            .build();

        match swapchain.queue_present(&present_info) {
            Ok(suboptimal) => suboptimal,
            Err(SurfaceError::OutOfDate) => {
                self.swapchain = None;
                return Ok(());
            }
            Err(e) => panic!("failed to acquire_next_image. Err: {}", e),
        };
        self.frame = (self.frame + 1) % MAX_FRAMES_IN_FLIGHT;
        Ok(())
    }

    pub fn recreate_swapchain(&mut self, inner_size: PhysicalSize<u32>) -> Result<(), DeviceError> {
        self.device.wait_idle();
        self.swapchain = None;

        let swapchain_desc = SwapchainDescriptor {
            adapter: &self.adapter,
            surface: &self.surface,
            instance: &self.instance,
            device: &self.device,
            queue_family: self.indices,
            dimensions: [inner_size.width, inner_size.height],
            command_pool: self.command_pool,
            graphics_queue: self.graphics_queue,
            present_queue: self.present_queue,
            allocator: &self.allocator,
            command_buffer_allocator: &self.command_buffer_allocator,
        };
        let swapchain = Swapchain::new(&swapchain_desc)?;
        self.swapchain = Some(swapchain);
        self.extent = vk::Extent2D {
            width: inner_size.width,
            height: inner_size.height,
        };
        log::debug!("======== Swapchain recreated.========");
        Ok(())
    }
}

impl Drop for VulkanRenderer {
    fn drop(&mut self) {
        self.device.wait_idle();
        self.swapchain = None; // drop first
        self.image_available_semaphores
            .iter()
            .for_each(|s| self.device.destroy_semaphore(*s));
        self.render_finished_semaphores
            .iter()
            .for_each(|s| self.device.destroy_semaphore(*s));
        self.in_flight_fences
            .iter()
            .for_each(|s| self.device.destroy_fence(*s));
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
