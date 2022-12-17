use super::{device::Device, instance::Instance, surface::Surface, utils};
use crate::vulkan::debug::DebugUtils;
use crate::vulkan::instance::InstanceFlags;
use crate::{AdapterRequirements, QueueFamilyIndices};
use ash::extensions::khr;
use ash::vk;
use std::collections::HashSet;
use std::ffi::{c_char, CString};

pub struct Adapter {
    raw: vk::PhysicalDevice,
}

impl Adapter {
    pub fn raw(&self) -> vk::PhysicalDevice {
        self.raw
    }

    pub fn new(raw: vk::PhysicalDevice) -> Self {
        Self { raw }
    }

    pub unsafe fn meet_requirements(
        &self,
        instance: &ash::Instance,
        surface: &Surface,
        requirements: &AdapterRequirements,
    ) -> Result<(), crate::DeviceError> {
        let properties = unsafe { instance.get_physical_device_properties(self.raw) };
        if requirements.discrete_gpu
            && properties.device_type != vk::PhysicalDeviceType::DISCRETE_GPU
        {
            log::error!("Device is not a discrete GPU, and one is required!");
            return Err(crate::DeviceError::NotMeetRequirement);
        }

        // let features = unsafe { instance.get_physical_device_features(self.raw) };
        let _queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(self.raw) };

        let queue_family_indices = utils::get_queue_family_indices(instance, self.raw, surface)?;
        if !queue_family_indices.has_meet_requirement(requirements) {
            log::error!("Device is not meet queue family indices' requirement! \nindices is {:#?},\nbut requirement is {:#?}", queue_family_indices, requirements);
            return Err(crate::DeviceError::NotMeetRequirement);
        }
        // log::info!(
        //     "indices is {:#?},\nrequirement is {:#?}",
        //     queue_family_indices,
        //     requirements
        // );

        Ok(())
    }

    pub unsafe fn open(
        &self,
        instance: &Instance,
        indices: QueueFamilyIndices,
        requirement: &AdapterRequirements,
        debug_utils: Option<DebugUtils>,
    ) -> Result<Device, crate::DeviceError> {
        let instance_raw = instance.raw();

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
            .sample_rate_shading(true);

        let enable_validation = instance.flags().contains(InstanceFlags::VALIDATION);
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

        let enable_extensions = [khr::Swapchain::name()];
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

        let ash_device: ash::Device =
            unsafe { instance_raw.create_device(self.raw, &device_create_info, None)? };

        log::debug!("Vulkan logical device created.");

        let device = Device::new(ash_device, debug_utils);
        Ok(device)
    }

    pub fn log_adapter_information(&self, instance: &ash::Instance) {
        let adapter = self.raw;
        let device_properties = unsafe { instance.get_physical_device_properties(adapter) };
        let device_features = unsafe { instance.get_physical_device_features(adapter) };
        let device_queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(adapter) };

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
