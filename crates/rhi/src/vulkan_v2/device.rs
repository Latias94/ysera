use crate::vulkan_v2::adapter::{Adapter, AdapterShared};
use crate::vulkan_v2::command_buffer_allocator::CommandBufferAllocator;
use crate::vulkan_v2::instance::{Instance, InstanceShared};
use crate::{AdapterRequirements, InstanceFlags, QueueFamilyIndices};
use ash::extensions::khr;
use ash::vk;
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};
use std::collections::HashSet;
use std::ffi::{c_char, CStr, CString};
use std::sync::{Arc, Mutex};

pub struct Device {
    /// Loads device local functions.
    pub(crate) raw: ash::Device,
    pub(crate) adapter: Arc<AdapterShared>,
    pub(crate) instance: Arc<InstanceShared>,
    pub(crate) allocator: Arc<Mutex<Allocator>>,
    pub(crate) graphics_queue: vk::Queue,
    pub(crate) present_queue: vk::Queue,
    pub indices: QueueFamilyIndices,
    pub cb_allocator: CommandBufferAllocator,
}

impl Device {
    pub fn raw(&self) -> &ash::Device {
        &self.raw
    }

    pub unsafe fn create(
        instance: &Instance,
        adapter: &Adapter,
        indices: QueueFamilyIndices,
        requirement: &AdapterRequirements,
    ) -> Result<Self, crate::DeviceError> {
        let instance_raw = &instance.shared.raw;

        let queue_priorities = &[1_f32];

        let mut unique_indices = HashSet::new();
        unique_indices.insert(indices.graphics_family.unwrap());
        unique_indices.insert(indices.present_family.unwrap());

        let queue_create_infos = unique_indices
            .iter()
            .map(|i| {
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(*i)
                    .queue_priorities(queue_priorities)
                    .build()
            })
            .collect::<Vec<_>>();

        let physical_device_features = vk::PhysicalDeviceFeatures::builder()
            .sampler_anisotropy(requirement.sampler_anisotropy)
            .sample_rate_shading(requirement.sample_rate_shading);

        let enable_validation = instance.shared.flags.contains(InstanceFlags::VALIDATION);
        let mut required_layers = vec![];
        if enable_validation {
            required_layers.push("VK_LAYER_KHRONOS_validation");
        }
        let required_validation_layer_raw_names: Vec<CString> = required_layers
            .iter()
            .map(|layer_name| CString::new(*layer_name).unwrap())
            .collect();
        let enable_layer_names: Vec<*const c_char> = required_validation_layer_raw_names
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect();

        let enable_extensions = Self::get_required_device_extensions();

        let support_extensions =
            Self::check_device_extension_support(&instance.shared.raw, adapter.shared.raw);
        if !support_extensions {
            log::error!("device extensions not support");
        }

        let enable_extension_names = enable_extensions
            .iter()
            // Safe because `enabled_extensions` entries have static lifetime.
            .map(|&s| s.as_ptr())
            .collect::<Vec<_>>();
        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_infos)
            .enabled_layer_names(&enable_layer_names)
            .enabled_extension_names(&enable_extension_names)
            .enabled_features(&physical_device_features);

        let raw: ash::Device =
            unsafe { instance_raw.create_device(adapter.shared.raw, &device_create_info, None)? };

        log::debug!("Vulkan logical device created.");

        // this queue should support graphics and present
        let graphics_queue = unsafe { raw.get_device_queue(indices.graphics_family.unwrap(), 0) };
        let present_queue = raw.get_device_queue(indices.present_family.unwrap(), 0);

        let allocator = Allocator::new(&AllocatorCreateDesc {
            instance: instance.shared.raw.clone(),
            device: raw.clone(),
            physical_device: adapter.shared.raw,
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

        let cb_allocator =
            CommandBufferAllocator::new(&raw, indices.graphics_family.unwrap(), graphics_queue)?;

        Ok(Self {
            raw,
            adapter: adapter.shared.clone(),
            instance: instance.shared.clone(),
            allocator,
            graphics_queue,
            present_queue,
            indices,
            cb_allocator,
        })
    }

    fn get_required_device_extensions() -> [&'static CStr; 1] {
        [khr::Swapchain::name()]
    }

    fn check_device_extension_support(
        instance: &ash::Instance,
        device: vk::PhysicalDevice,
    ) -> bool {
        let required_extensions = Self::get_required_device_extensions();

        let extension_props = unsafe {
            instance
                .enumerate_device_extension_properties(device)
                .expect("Failed to enumerate device extension properties")
        };

        for required in required_extensions.iter() {
            let found = extension_props.iter().any(|ext| {
                let name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
                required == &name
            });

            if !found {
                return false;
            }
        }
        true
    }

    pub unsafe fn destroy(&self) {
        log::debug!("device_destroy");
    }
}

impl Device {}
