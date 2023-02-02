use crate::vulkan_v2::swapchain::SwapChainSupportDetail;
use crate::vulkan_v2::{conv, Api};
use crate::{DeviceError, DownlevelFlags, Extent3d, Features, OpenDevice, QueueFamilyIndices};
use ash::extensions::khr;
use ash::vk;
use naga::back::spv;
use std::collections::BTreeMap;
use std::ffi::CStr;
use std::sync::Arc;

/// vulkan PhysicalDeviceFeature 的 wrapper
#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkPhysicalDeviceFeatures.html>"]
pub struct PhysicalDeviceFeatures {
    core: vk::PhysicalDeviceFeatures,
}

#[derive(Default)]
pub struct PhysicalDeviceCapabilities {
    pub supported_extensions: Vec<vk::ExtensionProperties>,
    pub properties: vk::PhysicalDeviceProperties,
    pub driver: Option<vk::PhysicalDeviceDriverPropertiesKHR>,
}

// This is safe because the structs have `p_next: *mut c_void`, which we null out/never read.
unsafe impl Send for PhysicalDeviceCapabilities {}
unsafe impl Sync for PhysicalDeviceCapabilities {}

impl crate::PhysicalDevice<Api> for super::PhysicalDevice {
    unsafe fn open(&self, features: Features) -> Result<OpenDevice<Api>, DeviceError> {
        profiling::scope!("PhysicalDevice open");

        // 根据参数，检查该 PhysicalDevice 满不满足要求
        let enabled_extensions = self.required_device_extensions(features);
        let mut enabled_phd_features = self.physical_device_features(&enabled_extensions, features);
        let indices = if self.surface.is_some() {
            self.find_queue_family()
        } else {
            QueueFamilyIndices {
                graphics_family: Some(0),
                present_family: Some(0),
                compute_family: Some(0),
                transfer_family: Some(0),
            }
        };

        let queue_priorities = [1_f32];
        let queue_create_info = vk::DeviceQueueCreateInfo::builder()
            .queue_priorities(&queue_priorities)
            .queue_family_index(indices.graphics_family.unwrap())
            .build();
        let queue_create_infos = [queue_create_info];

        let enabled_extension_names = enabled_extensions
            .iter()
            .map(|&s| {
                // Safe because `enabled_extensions` entries have static lifetime.
                s.as_ptr()
            })
            .collect::<Vec<_>>();

        // 如果要顾及兼容性，需要在这里加上 Device 的 validation layer
        let device_create_pre_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&enabled_extension_names);

        let device_create_info = enabled_phd_features
            .add_to_device_create_builder(device_create_pre_info)
            .build();

        let device: ash::Device = unsafe {
            profiling::scope!("vkCreateDevice");
            self.instance
                .raw
                .create_device(self.raw, &device_create_info, None)
                .expect("Failed to create logical Device!")
        };

        log::info!("Vulkan logical device created.");

        self.from_raw(
            device,
            enabled_extensions,
            features,
            indices.graphics_family.unwrap(),
            0,
        )
    }

    unsafe fn surface_capabilities(
        &self,
        surface: &super::Surface,
    ) -> Option<crate::SurfaceCapabilities> {
        let detail = self.query_swapchain_support(surface);
        let formats = detail
            .formats
            .into_iter()
            .filter_map(conv::map_vk_surface_formats)
            .collect();
        let present_modes = detail
            .present_modes
            .into_iter()
            .filter_map(conv::map_vk_present_mode)
            .collect();
        let caps = detail.capabilities;

        // `0xFFFFFFFF` indicates that the extent depends on the created swapchain.
        let current_extent =
            if caps.current_extent.width != u32::MAX && caps.current_extent.height != u32::MAX {
                Some(Extent3d {
                    width: caps.current_extent.width,
                    height: caps.current_extent.height,
                    depth_or_array_layers: 1,
                })
            } else {
                None
            };

        let min_extent = Extent3d {
            width: caps.min_image_extent.width,
            height: caps.min_image_extent.height,
            depth_or_array_layers: 1,
        };

        let max_extent = Extent3d {
            width: caps.max_image_extent.width,
            height: caps.max_image_extent.height,
            depth_or_array_layers: caps.max_image_array_layers,
        };
        // If image count is 0, the support number of images is unlimited.
        let max_image_count = if caps.max_image_count == 0 {
            u32::MAX
        } else {
            caps.max_image_count
        };
        Some(crate::SurfaceCapabilities {
            formats,
            swap_chain_sizes: detail.capabilities.min_image_count..=max_image_count,
            current_extent,
            extents: min_extent..=max_extent,
            present_modes,
            usage: conv::map_vk_image_usage(caps.supported_usage_flags),
        })
    }
}

impl super::PhysicalDevice {
    pub fn raw_physical_device(&self) -> vk::PhysicalDevice {
        self.raw
    }

    pub fn physical_device_capabilities(&self) -> &PhysicalDeviceCapabilities {
        &self.phd_capabilities
    }

    pub fn shared_instance(&self) -> &super::InstanceShared {
        &self.instance
    }

    fn required_device_extensions(&self, features: Features) -> Vec<&'static CStr> {
        let (supported_extensions, unsupported_extensions) = self
            .phd_capabilities
            .get_required_extensions(features)
            .iter()
            .partition::<Vec<&CStr>, _>(|&&extension| {
                self.phd_capabilities.supported_extension(extension)
            });
        if !unsupported_extensions.is_empty() {
            log::warn!("Missing extensions: {:?}", unsupported_extensions);
        }

        log::debug!("Supported extensions: {:?}", supported_extensions);
        supported_extensions
    }

    pub fn physical_device_features(
        &self,
        enabled_extensions: &[&'static CStr],
        features: Features,
    ) -> PhysicalDeviceFeatures {
        PhysicalDeviceFeatures::from_extensions_and_requested_features(
            enabled_extensions,
            features,
            self.downlevel_flags,
        )
    }

    fn query_swapchain_support(&self, surface: &super::Surface) -> SwapChainSupportDetail {
        profiling::scope!("query_swapchain_support");
        let physical_device = self.raw;
        let surface_shared = surface.shared.clone();
        let surface_loader = &surface_shared.fp;
        let surface = surface_shared.raw;
        unsafe {
            let capabilities = surface_loader
                .get_physical_device_surface_capabilities(physical_device, surface)
                .expect("Failed to query for surface capabilities.");
            let formats = surface_loader
                .get_physical_device_surface_formats(physical_device, surface)
                .expect("Failed to query for surface formats.");
            let present_modes = surface_loader
                .get_physical_device_surface_present_modes(physical_device, surface)
                .expect("Failed to query for surface present mode.");

            SwapChainSupportDetail {
                capabilities,
                formats,
                present_modes,
            }
        }
    }

    pub unsafe fn from_raw(
        &self,
        raw_device: ash::Device,
        enabled_extensions: Vec<&'static CStr>,
        features: Features,
        family_index: u32,
        queue_index: u32,
    ) -> Result<OpenDevice<Api>, DeviceError> {
        // 返回的 vk::PhysicalDeviceMemoryProperties 结构有两个数组内存类型和内存堆。内存堆是不同的内存资源，
        // 如专用 VRAM 和当 VRAM 耗尽时 RAM 中的交换空间。不同类型的内存存在于这些堆中。现在我们只关心内存的类型，
        // 而不关心它来自的堆，但是您可以想象这可能会影响性能。
        let mem_properties = {
            profiling::scope!("vkGetPhysicalDeviceMemoryProperties");
            self.instance
                .raw
                .get_physical_device_memory_properties(self.raw)
        };
        // 我们首先找到适合缓冲区本身的内存类型
        // 来自 requirements 参数的内存类型位字段将用于指定合适的内存类型的位字段。
        // 这意味着我们可以通过简单地遍历它们并检查相应的位是否设置为 1 来找到合适的内存类型的索引。

        let memory_types =
            &mem_properties.memory_types[..mem_properties.memory_type_count as usize];
        let valid_memory_types: u32 = memory_types.iter().enumerate().fold(0, |u, (i, mem)| {
            if self.known_memory_flags.contains(mem.property_flags) {
                u | (1 << i)
            } else {
                u
            }
        });
        let swapchain_loader = khr::Swapchain::new(&self.instance.raw, &raw_device);
        let raw_queue = {
            profiling::scope!("vkGetDeviceQueue");
            raw_device.get_device_queue(family_index, queue_index)
        };
        let shared = Arc::new(super::DeviceShared {
            raw: raw_device,
            family_index,
            queue_index,
            raw_queue,
            instance: Arc::clone(&self.instance),
            physical_device: self.raw,
            enabled_extensions: enabled_extensions.into(),
            vendor_id: self.phd_capabilities.properties.vendor_id,
            timestamp_period: self.phd_capabilities.properties.limits.timestamp_period,
            render_passes: Default::default(),
            framebuffers: Default::default(),
        });

        let queue = super::Queue {
            raw: raw_queue,
            swapchain_loader,
            device: Arc::clone(&shared),
            family_index,
        };
        let naga_options = {
            let capabilities = vec![
                spv::Capability::Shader,
                spv::Capability::Matrix,
                spv::Capability::Sampled1D,
                spv::Capability::Image1D,
                spv::Capability::ImageQuery,
                spv::Capability::DerivativeControl,
                spv::Capability::SampledCubeArray,
                spv::Capability::SampleRateShading,
                spv::Capability::StorageImageExtendedFormats,
            ];
            let mut flags = spv::WriterFlags::empty();
            flags.set(
                spv::WriterFlags::DEBUG,
                self.instance.flags.contains(crate::InstanceFlags::DEBUG),
            );
            spv::Options {
                lang_version: (1, 0),
                flags,
                capabilities: Some(capabilities.iter().cloned().collect()),
                bounds_check_policies: naga::proc::BoundsCheckPolicies {
                    index: naga::proc::BoundsCheckPolicy::Restrict,
                    buffer: naga::proc::BoundsCheckPolicy::Restrict,
                    image: naga::proc::BoundsCheckPolicy::Restrict,
                    binding_array: naga::proc::BoundsCheckPolicy::Unchecked,
                },
                binding_map: BTreeMap::default(),
                zero_initialize_workgroup_memory: spv::ZeroInitializeWorkgroupMemoryMode::Polyfill,
            }
        };

        let device = super::Device {
            shared,
            valid_memory_types,
            naga_options,
        };
        Ok(OpenDevice { device, queue })
    }

    fn find_queue_family(&self) -> QueueFamilyIndices {
        let instance = &self.instance.raw;
        let physical_device = self.raw;
        let surface_shared = self.surface.clone().unwrap();
        let surface_loader = &surface_shared.fp;
        let surface = surface_shared.raw;
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let mut queue_family_indices = QueueFamilyIndices::default();

        for (index, queue_family) in queue_families.into_iter().enumerate() {
            let is_present_support = unsafe {
                surface_loader
                    .get_physical_device_surface_support(physical_device, index as u32, surface)
                    .expect("Get physical device surface support failed.")
            };

            if queue_family.queue_count > 0 {
                if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                    queue_family_indices.graphics_family = Some(index as u32);
                }
                if is_present_support {
                    queue_family_indices.present_family = Some(index as u32);
                }
            }

            if queue_family_indices.is_complete() {
                break;
            }
        }

        queue_family_indices
    }
}

impl PhysicalDeviceFeatures {
    pub fn add_to_device_create_builder<'a>(
        &'a mut self,
        mut info: vk::DeviceCreateInfoBuilder<'a>,
    ) -> vk::DeviceCreateInfoBuilder<'a> {
        info = info.enabled_features(&self.core);
        info
    }

    fn from_extensions_and_requested_features(
        enabled_extensions: &[&'static CStr],
        requested_features: Features,
        downlevel_flags: DownlevelFlags,
    ) -> Self {
        Self {
            core: vk::PhysicalDeviceFeatures::builder()
                .sampler_anisotropy(downlevel_flags.contains(DownlevelFlags::ANISOTROPIC_FILTERING))
                .sample_rate_shading(downlevel_flags.contains(DownlevelFlags::MULTISAMPLED_SHADING))
                .build(),
        }
    }

    pub fn get_features_and_downlevel_flags(
        &self,
        instance: &ash::Instance,
        phd: vk::PhysicalDevice,
        caps: &PhysicalDeviceCapabilities,
    ) -> (Features, DownlevelFlags) {
        let mut features = Features::empty();
        let mut downlevel_flag = DownlevelFlags::all();
        downlevel_flag.set(
            DownlevelFlags::ANISOTROPIC_FILTERING,
            self.core.sampler_anisotropy != 0,
        );
        downlevel_flag.set(
            DownlevelFlags::MULTISAMPLED_SHADING,
            self.core.sample_rate_shading != 0,
        );
        (features, downlevel_flag)
    }
}

impl PhysicalDeviceCapabilities {
    pub fn properties(&self) -> vk::PhysicalDeviceProperties {
        self.properties
    }

    pub fn supported_extension(&self, extension: &CStr) -> bool {
        self.supported_extensions
            .iter()
            .any(|ep| unsafe { CStr::from_ptr(ep.extension_name.as_ptr()) } == extension)
    }

    pub fn get_required_extensions(&self, requested_features: Features) -> Vec<&'static CStr> {
        let mut extensions = vec![];
        extensions.push(khr::Swapchain::name());
        extensions
    }
}

impl super::InstanceShared {
    pub fn get_physical_device_information(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> (PhysicalDeviceCapabilities, PhysicalDeviceFeatures) {
        let extension_properties = unsafe {
            self.raw
                .enumerate_device_extension_properties(physical_device)
                .expect("Failed to get device extension properties")
        };
        let device_properties = unsafe { self.raw.get_physical_device_properties(physical_device) };

        let capabilities = PhysicalDeviceCapabilities {
            supported_extensions: extension_properties,
            properties: device_properties,
            driver: Some(vk::PhysicalDeviceDriverPropertiesKHR::default()),
        };
        let core = vk::PhysicalDeviceFeatures::default();

        let features = PhysicalDeviceFeatures { core };

        (capabilities, features)
    }
}
