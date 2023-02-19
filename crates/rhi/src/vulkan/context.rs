use std::sync::Arc;

use anyhow::Result;
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};
use parking_lot::Mutex;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use crate::types::{AdapterRequirements, DeviceFeatures, DeviceRequirements, InstanceDescriptor};
use crate::vulkan::device::Device;
use crate::vulkan::instance::{Instance, Surface};

pub struct ContextDescriptor<'a> {
    pub app_name: &'a str,
    pub window_handle: &'a dyn HasRawWindowHandle,
    pub display_handle: &'a dyn HasRawDisplayHandle,
    pub vulkan_version: u32,
    pub required_extensions: &'a [&'a str],
    pub device_feature: DeviceFeatures,
}

#[derive(Clone)]
pub struct Context {
    pub surface: Arc<Surface>,
    pub instance: Arc<Instance>,
    pub device: Arc<Device>,
    pub allocator: Arc<Mutex<Allocator>>,
}

impl Context {
    pub unsafe fn new(desc: ContextDescriptor) -> Result<Self> {
        let instance_desc = InstanceDescriptor::builder()
            // .flags(crate::vulkan::instance::InstanceFlags::empty())
            // .debug_level_filter(log::LevelFilter::Info)
            // .vulkan_version(desc.vulkan_version)
            .build();
        let instance = unsafe { Instance::init(&instance_desc)? };
        let surface = unsafe { instance.create_surface(desc.window_handle, desc.display_handle)? };
        let adapters = instance.enumerate_adapters(&surface)?;
        assert!(!adapters.is_empty());

        let adapter_requirements = AdapterRequirements::builder()
            .extra_features(DeviceFeatures::builder().synchronization2(true).build())
            // .compute(true)
            .build();
        let device_requirements = DeviceRequirements::builder()
            .required_extension(desc.required_extensions)
            .build();
        let mut selected_adapter = None;
        for adapter in adapters {
            if unsafe { adapter.meet_requirements(&adapter_requirements) }.is_ok() {
                selected_adapter = Some(adapter);
                break;
            }
        }
        let adapter = match selected_adapter {
            None => panic!("Cannot find the require device."),
            Some(adapter) => adapter,
        };

        let instance = Arc::new(instance);
        let surface = Arc::new(surface);

        log::debug!("Find the require device.");
        let debug_utils = instance.shared_instance().debug_utils().clone();

        let indices = adapter.queue_family_indices();

        let device = unsafe {
            Device::create(
                &instance,
                adapter,
                &adapter_requirements,
                &device_requirements,
            )?
        };

        let allocator = Allocator::new(&AllocatorCreateDesc {
            instance: instance.shared_instance().raw().clone(),
            device: device.raw().clone(),
            physical_device: device.adapter().raw(),
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

        let device = Arc::new(device);

        Ok(Self {
            instance,
            surface,
            device,
            allocator,
        })
    }
}
