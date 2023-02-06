use crate::vulkan::adapter::Adapter;
use crate::vulkan::command_buffer_allocator::CommandBufferAllocator;
use crate::vulkan::device::{Device, DeviceFeatures};
use crate::vulkan::instance::Instance;
use crate::vulkan::surface::Surface;
use crate::vulkan::utils;
use crate::{AdapterRequirements, InstanceDescriptor, QueueFamilyIndices};
use anyhow::Result;
use ash::vk;
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};
use parking_lot::Mutex;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::sync::Arc;

pub struct ContextDescriptor<'a> {
    pub app_name: &'a str,
    pub window_handle: &'a dyn HasRawWindowHandle,
    pub display_handle: &'a dyn HasRawDisplayHandle,
    pub vulkan_version: u32,
    pub required_extensions: &'a [&'a str],
    pub device_feature: DeviceFeatures,
}

pub struct Context {
    pub adapter: Arc<Adapter>,
    pub instance: Arc<Instance>,
    pub surface: Arc<Surface>,
    pub device: Arc<Device>,
    pub allocator: Arc<Mutex<Allocator>>,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
    pub command_buffer_allocator: CommandBufferAllocator,
    pub indices: QueueFamilyIndices,
}

impl Context {
    pub unsafe fn new(desc: ContextDescriptor) -> Result<Self> {
        let instance_desc = InstanceDescriptor::builder()
            // .flags(crate::vulkan::instance::InstanceFlags::empty())
            // .debug_level_filter(log::LevelFilter::Info)
            .vulkan_version(desc.vulkan_version)
            .build();
        let instance = unsafe { Instance::init(&instance_desc)? };
        let surface = unsafe { instance.create_surface(desc.window_handle, desc.display_handle)? };
        let adapters = instance.enumerate_adapters()?;
        assert!(!adapters.is_empty());

        let requirements = AdapterRequirements::builder()
            // .compute(true)
            .required_extension(desc.required_extensions)
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

        let adapter = Arc::new(adapter);
        let instance = Arc::new(instance);
        let surface = Arc::new(surface);

        log::debug!("Find the require device.");
        let debug_utils = instance.debug_utils().clone();

        let indices = utils::get_queue_family_indices(instance.raw(), adapter.raw(), &surface)?;
        indices.log_debug();

        let device =
            unsafe { adapter.open(&instance, indices, &requirements, debug_utils.clone())? };

        let allocator = Allocator::new(&AllocatorCreateDesc {
            instance: instance.raw().clone(),
            device: device.raw().clone(),
            physical_device: adapter.raw(),
            debug_settings: Default::default(),
            // check https://stackoverflow.com/questions/73341075/rust-gpu-allocator-bufferdeviceaddress-must-be-enabbled
            buffer_device_address: false,
        });

        let allocator = match allocator {
            Ok(x) => x,
            Err(e) => {
                log::error!("gpu-allocator allocator create failed!");
                panic!("{e}");
            }
        };
        let allocator = Arc::new(Mutex::new(allocator));

        // this queue should support graphics and present
        let graphics_queue = device.get_device_queue(indices.graphics_family.unwrap(), 0);
        let present_queue = device.get_device_queue(indices.present_family.unwrap(), 0);
        let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(indices.graphics_family.unwrap())
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .build();
        let command_pool = device.create_command_pool(&command_pool_create_info)?;

        let device = Arc::new(device);
        let command_buffer_allocator =
            CommandBufferAllocator::new(&device, command_pool, graphics_queue);

        Ok(Self {
            adapter,
            instance,
            surface,
            device,
            allocator,
            graphics_queue,
            present_queue,
            command_buffer_allocator,
            indices,
        })
    }
}
