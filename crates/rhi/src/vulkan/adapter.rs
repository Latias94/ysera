use std::sync::Arc;

use ash::vk;

use crate::types::{AdapterRequirements, DeviceFeatures, QueueFamilyIndices};
use crate::utils::c_char_to_string;
use crate::vulkan::instance::InstanceShared;
use crate::DeviceError;

use super::{instance::Instance, instance::Surface};

pub struct Adapter {
    raw: vk::PhysicalDevice,
    instance: Arc<InstanceShared>,
    max_msaa_samples: vk::SampleCountFlags,
    properties: vk::PhysicalDeviceProperties,
    features: vk::PhysicalDeviceFeatures,
    extra_features: DeviceFeatures,
    queue_family_properties: Vec<vk::QueueFamilyProperties>,
    queue_family_indices: QueueFamilyIndices,
}

impl Adapter {
    pub fn raw(&self) -> vk::PhysicalDevice {
        self.raw
    }

    pub fn max_msaa_samples(&self) -> vk::SampleCountFlags {
        // todo
        // self.max_msaa_samples
        vk::SampleCountFlags::TYPE_1
    }

    pub fn queue_family_indices(&self) -> QueueFamilyIndices {
        self.queue_family_indices
    }

    pub fn shared_instance(&self) -> &InstanceShared {
        &self.instance
    }
}

impl Adapter {
    pub fn new(
        raw: vk::PhysicalDevice,
        instance: &Instance,
        surface: &Surface,
    ) -> Result<Self, DeviceError> {
        let instance_raw = instance.shared_instance().raw();
        let max_msaa_samples = Self::get_max_msaa_samples(raw, instance_raw);
        let properties = unsafe { instance_raw.get_physical_device_properties(raw) };

        let features = unsafe { instance_raw.get_physical_device_features(raw) };

        let queue_family_properties =
            unsafe { instance_raw.get_physical_device_queue_family_properties(raw) };

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

        unsafe { instance_raw.get_physical_device_features2(raw, &mut features2) };
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
            raw,
            instance: instance.shared.clone(),
            max_msaa_samples,
            properties,
            features,
            extra_features,
            queue_family_properties,
            queue_family_indices,
        })
    }

    pub unsafe fn meet_requirements(
        &self,
        requirements: &AdapterRequirements,
    ) -> Result<(), DeviceError> {
        if requirements.discrete_gpu
            && self.properties.device_type != vk::PhysicalDeviceType::DISCRETE_GPU
        {
            log::error!("This Physical Device is not a discrete GPU, and one is required!");
            return Err(DeviceError::NotMeetRequirement);
        }

        if requirements.sampler_anisotropy && self.features.sampler_anisotropy != vk::TRUE {
            log::error!("This Physical Device is not support sampler anisotropy!");
            return Err(DeviceError::NotMeetRequirement);
        }

        if !self.queue_family_indices.has_meet_requirement(requirements) {
            log::error!("This Physical Device is not meet queue family indices' requirement! \nindices is {:#?},\nbut requirement is {:#?}", self.queue_family_indices, requirements);
            return Err(DeviceError::NotMeetRequirement);
        }

        if !self
            .extra_features
            .is_compatible_with(&requirements.extra_features)
        {
            log::error!("This Physical Device is not support extra features! \nsupport {:#?},\nbut requirement is {:#?}", self.extra_features, requirements.extra_features);
            return Err(DeviceError::NotMeetRequirement);
        }

        // log::info!(
        //     "indices is {:#?},\nrequirement is {:#?}",
        //     queue_family_indices,
        //     requirements
        // );

        Ok(())
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
        let device_name = c_char_to_string(&device_properties.device_name);
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
