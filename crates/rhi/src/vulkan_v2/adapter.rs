use std::sync::Arc;

use ash::extensions::khr;
use ash::vk;

use crate::{AdapterRequirements, DeviceError};

use super::{instance::Instance, utils};

pub struct Adapter {
    pub shared: Arc<AdapterShared>,
}

pub struct AdapterShared {
    pub raw: vk::PhysicalDevice,
    pub max_msaa_samples: vk::SampleCountFlags,
}

impl Adapter {
    pub fn new(raw: vk::PhysicalDevice, instance: &Instance) -> Self {
        let max_msaa_samples = Self::get_max_msaa_samples(raw, &instance.shared.raw);
        let shared = Arc::new(AdapterShared {
            raw,
            max_msaa_samples,
        });

        Self { shared }
    }

    pub unsafe fn meet_requirements(
        &self,
        instance: &ash::Instance,
        surface: &Surface,
        requirements: &AdapterRequirements,
    ) -> Result<(), DeviceError> {
        let properties = unsafe { instance.get_physical_device_properties(self.shared.raw) };
        if requirements.discrete_gpu
            && properties.device_type != vk::PhysicalDeviceType::DISCRETE_GPU
        {
            log::error!("Device is not a discrete GPU, and one is required!");
            return Err(DeviceError::NotMeetRequirement);
        }

        let features = unsafe { instance.get_physical_device_features(self.shared.raw) };
        if requirements.sampler_anisotropy && features.sampler_anisotropy != vk::TRUE {
            log::error!("Device is not support sampler anisotropy!");
            return Err(DeviceError::NotMeetRequirement);
        }

        let _queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(self.shared.raw) };

        let queue_family_indices =
            utils::get_queue_family_indices(instance, self.shared.raw, surface)?;
        if !queue_family_indices.has_meet_requirement(requirements) {
            log::error!("Device is not meet queue family indices' requirement! \nindices is {:#?},\nbut requirement is {:#?}", queue_family_indices, requirements);
            return Err(DeviceError::NotMeetRequirement);
        }
        // log::info!(
        //     "indices is {:#?},\nrequirement is {:#?}",
        //     queue_family_indices,
        //     requirements
        // );

        Ok(())
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
        instance_raw: &ash::Instance,
    ) -> vk::SampleCountFlags {
        let properties = unsafe { instance_raw.get_physical_device_properties(adapter) };
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

pub struct Surface {
    raw: vk::SurfaceKHR,
    loader: khr::Surface,
}

impl Surface {
    pub fn raw(&self) -> vk::SurfaceKHR {
        self.raw
    }

    pub fn loader(&self) -> &khr::Surface {
        &self.loader
    }

    pub fn new(raw: vk::SurfaceKHR, loader: khr::Surface) -> Self {
        Self { raw, loader }
    }

    pub unsafe fn get_physical_device_surface_support(
        &self,
        adapter: vk::PhysicalDevice,
        index: u32,
    ) -> Result<bool, DeviceError> {
        let support = unsafe {
            self.loader
                .get_physical_device_surface_support(adapter, index, self.raw)
                .map_err(DeviceError::Vulkan)?
        };
        Ok(support)
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.loader.destroy_surface(self.raw, None);
        }
    }
}
