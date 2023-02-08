use alloc::ffi::CString;
use std::collections::HashSet;
use std::ffi::{c_char, CStr};
use std::sync::Arc;

use ash::extensions::khr;
use ash::vk;

use crate::vulkan::debug::DebugUtils;
use crate::vulkan::device::DeviceFeatures;
use crate::{
    AdapterRequirements, DeviceError, DeviceRequirements, InstanceFlags, QueueFamilyIndices,
};

use super::{device::Device, instance::Instance, surface::Surface, utils};

pub struct Adapter {
    pub shared: Arc<AdapterShared>,
}

pub struct AdapterShared {
    raw: vk::PhysicalDevice,
    max_msaa_samples: vk::SampleCountFlags,
    properties: vk::PhysicalDeviceProperties,
    features: vk::PhysicalDeviceFeatures,
    extra_features: DeviceFeatures,
    queue_family_properties: Vec<vk::QueueFamilyProperties>,
    queue_family_indices: QueueFamilyIndices,
}

impl AdapterShared {
    pub fn raw(&self) -> vk::PhysicalDevice {
        self.raw
    }

    pub fn max_msaa_samples(&self) -> vk::SampleCountFlags {
        self.max_msaa_samples
    }

    pub fn queue_family_indices(&self) -> QueueFamilyIndices {
        self.queue_family_indices
    }
}

impl Adapter {
    pub fn raw(&self) -> vk::PhysicalDevice {
        self.shared.raw
    }

    pub fn max_msaa_samples(&self) -> vk::SampleCountFlags {
        self.shared.max_msaa_samples
    }

    pub fn queue_family_indices(&self) -> QueueFamilyIndices {
        self.shared.queue_family_indices
    }
}

impl Adapter {
    pub fn new(
        raw: vk::PhysicalDevice,
        instance: &ash::Instance,
        surface: &Surface,
    ) -> Result<Self, DeviceError> {
        let max_msaa_samples = Self::get_max_msaa_samples(raw, instance);
        let properties = unsafe { instance.get_physical_device_properties(raw) };

        let features = unsafe { instance.get_physical_device_features(raw) };

        let queue_family_properties =
            unsafe { instance.get_physical_device_queue_family_properties(raw) };

        let mut ray_tracing_feature = vk::PhysicalDeviceRayTracingPipelineFeaturesKHR::default();
        let mut acceleration_struct_feature =
            vk::PhysicalDeviceAccelerationStructureFeaturesKHR::default();
        let mut vulkan_12_features = vk::PhysicalDeviceVulkan12Features::builder()
            .runtime_descriptor_array(true)
            .buffer_device_address(true);
        let mut vulkan_13_features = vk::PhysicalDeviceVulkan13Features::default();
        let mut features2 = vk::PhysicalDeviceFeatures2::builder()
            .push_next(&mut ray_tracing_feature)
            .push_next(&mut acceleration_struct_feature)
            .push_next(&mut vulkan_12_features)
            .push_next(&mut vulkan_13_features);

        unsafe { instance.get_physical_device_features2(raw, &mut features2) };
        let extra_features = DeviceFeatures {
            ray_tracing_pipeline: ray_tracing_feature.ray_tracing_pipeline == vk::TRUE,
            acceleration_structure: acceleration_struct_feature.acceleration_structure == vk::TRUE,
            runtime_descriptor_array: vulkan_12_features.runtime_descriptor_array == vk::TRUE,
            buffer_device_address: vulkan_12_features.buffer_device_address == vk::TRUE,
            dynamic_rendering: vulkan_13_features.dynamic_rendering == vk::TRUE,
            synchronization2: vulkan_13_features.synchronization2 == vk::TRUE,
        };
        let queue_family_indices =
            Self::get_queue_family_indices(raw, &queue_family_properties, surface)?;

        Ok(Self {
            shared: Arc::new(AdapterShared {
                raw,
                max_msaa_samples,
                properties,
                features,
                extra_features,
                queue_family_properties,
                queue_family_indices,
            }),
        })
    }

    pub unsafe fn meet_requirements(
        &self,
        requirements: &AdapterRequirements,
    ) -> Result<(), DeviceError> {
        if requirements.discrete_gpu
            && self.shared.properties.device_type != vk::PhysicalDeviceType::DISCRETE_GPU
        {
            log::error!("This Physical Device is not a discrete GPU, and one is required!");
            return Err(DeviceError::NotMeetRequirement);
        }

        if requirements.sampler_anisotropy && self.shared.features.sampler_anisotropy != vk::TRUE {
            log::error!("This Physical Device is not support sampler anisotropy!");
            return Err(DeviceError::NotMeetRequirement);
        }

        if !self
            .shared
            .queue_family_indices
            .has_meet_requirement(requirements)
        {
            log::error!("This Physical Device is not meet queue family indices' requirement! \nindices is {:#?},\nbut requirement is {:#?}", self.shared.queue_family_indices, requirements);
            return Err(DeviceError::NotMeetRequirement);
        }

        if !self
            .shared
            .extra_features
            .is_compatible_with(&requirements.extra_features)
        {
            log::error!("This Physical Device is not support extra features! \nsupport {:#?},\nbut requirement is {:#?}", self.shared.extra_features, requirements.extra_features);
            return Err(DeviceError::NotMeetRequirement);
        }

        // log::info!(
        //     "indices is {:#?},\nrequirement is {:#?}",
        //     queue_family_indices,
        //     requirements
        // );

        Ok(())
    }

    pub unsafe fn create_device(
        &self,
        instance: &Instance,
        indices: QueueFamilyIndices,
        adapter_req: &AdapterRequirements,
        device_req: &DeviceRequirements,
        debug_utils: Option<DebugUtils>,
    ) -> Result<Device, DeviceError> {
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

        let mut device_extensions_ptrs = device_req
            .required_extension
            .iter()
            .map(|e| CString::new(*e))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        if device_req.use_swapchain {
            device_extensions_ptrs.push(khr::Swapchain::name().into());
        }

        let physical_device_features = vk::PhysicalDeviceFeatures::builder()
            .sampler_anisotropy(adapter_req.sampler_anisotropy)
            .sample_rate_shading(adapter_req.sample_rate_shading);

        let mut ray_tracing_feature = vk::PhysicalDeviceRayTracingPipelineFeaturesKHR::builder()
            .ray_tracing_pipeline(adapter_req.extra_features.ray_tracing_pipeline);
        let mut acceleration_struct_feature =
            vk::PhysicalDeviceAccelerationStructureFeaturesKHR::builder()
                .acceleration_structure(adapter_req.extra_features.acceleration_structure);
        let mut vulkan_12_features = vk::PhysicalDeviceVulkan12Features::builder()
            .runtime_descriptor_array(adapter_req.extra_features.runtime_descriptor_array)
            .buffer_device_address(adapter_req.extra_features.buffer_device_address);
        let mut vulkan_13_features = vk::PhysicalDeviceVulkan13Features::builder()
            .dynamic_rendering(adapter_req.extra_features.dynamic_rendering)
            .synchronization2(adapter_req.extra_features.synchronization2);

        let mut physical_device_features2 = vk::PhysicalDeviceFeatures2::builder()
            .push_next(&mut ray_tracing_feature)
            .push_next(&mut acceleration_struct_feature)
            .push_next(&mut vulkan_12_features)
            .push_next(&mut vulkan_13_features);

        let device_extensions_ptrs = device_extensions_ptrs
            .iter()
            // Safe because `enabled_extensions` entries have static lifetime.
            .map(|e| e.as_ptr())
            .collect::<Vec<_>>();

        let support_extensions = Self::check_device_extension_support(
            instance,
            self.shared.raw,
            device_req.required_extension,
            device_req.use_swapchain,
        );

        if !support_extensions {
            log::error!("device extensions not support");
        }

        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_infos)
            .enabled_layer_names(&enable_layer_names)
            .enabled_extension_names(&device_extensions_ptrs)
            .enabled_features(&physical_device_features)
            .push_next(&mut physical_device_features2);

        let ash_device: ash::Device =
            unsafe { instance_raw.create_device(self.shared.raw, &device_create_info, None)? };

        log::debug!("Vulkan logical device created.");

        let device = Device::new(ash_device, debug_utils, self.shared.clone());
        Ok(device)
    }

    fn check_device_extension_support(
        instance: &Instance,
        device: vk::PhysicalDevice,
        extensions: &[&str],
        use_swapchain: bool,
    ) -> bool {
        let extension_props = unsafe {
            instance
                .raw()
                .enumerate_device_extension_properties(device)
                .expect("Failed to enumerate device extension properties")
        };

        for required in extensions.iter() {
            let found = extension_props.iter().any(|ext| {
                let name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
                *required == name.to_str().unwrap().to_owned()
            });

            if !found {
                return false;
            }
        }

        let swapchain_check =
            use_swapchain && extensions.contains(&khr::Swapchain::name().to_str().unwrap());
        swapchain_check
    }

    pub fn get_queue_family_indices(
        adapter: vk::PhysicalDevice,
        queue_family_properties: &[vk::QueueFamilyProperties],
        surface: &Surface,
    ) -> Result<QueueFamilyIndices, DeviceError> {
        let mut indices = QueueFamilyIndices::default();
        for (i, queue_family) in queue_family_properties.iter().enumerate() {
            if indices.is_complete() {
                break;
            }
            let index = i as u32;
            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                indices.graphics_family = Some(index);
            };
            if queue_family.queue_flags.contains(vk::QueueFlags::COMPUTE) {
                indices.compute_family = Some(index);
            };
            if queue_family.queue_flags.contains(vk::QueueFlags::TRANSFER) {
                indices.transfer_family = Some(index);
            };
            let support_present = unsafe {
                surface
                    .loader()
                    .get_physical_device_surface_support(adapter, index, surface.raw())
                    .map_err(DeviceError::Vulkan)?
            };

            if support_present {
                indices.present_family = Some(index);
            }
        }
        Ok(indices)
    }

    pub fn log_adapter_information(&self, instance: &ash::Instance) {
        let adapter = self.shared.raw;
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

    fn get_max_msaa_samples(
        adapter: vk::PhysicalDevice,
        instance: &ash::Instance,
    ) -> vk::SampleCountFlags {
        let properties = unsafe { instance.get_physical_device_properties(adapter) };
        let counts = properties.limits.framebuffer_color_sample_counts
            & properties.limits.framebuffer_depth_sample_counts;
        [
            vk::SampleCountFlags::TYPE_64,
            vk::SampleCountFlags::TYPE_32,
            vk::SampleCountFlags::TYPE_16,
            vk::SampleCountFlags::TYPE_8,
            vk::SampleCountFlags::TYPE_4,
            vk::SampleCountFlags::TYPE_2,
        ]
        .iter()
        .cloned()
        .find(|c| counts.contains(*c))
        .unwrap_or(vk::SampleCountFlags::TYPE_1)
    }
}
