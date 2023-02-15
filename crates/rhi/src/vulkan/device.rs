use ash::extensions::khr;
use std::collections::HashSet;
use std::ffi::{c_char, CStr, CString};

use ash::vk;

use crate::types::{AdapterRequirements, DeviceRequirements, InstanceFlags, QueueFamilyIndices};
use crate::vulkan::adapter::Adapter;
use crate::vulkan::command_buffer::{CommandBuffer, CommandBufferState};
use crate::vulkan::instance::Instance;
use crate::DeviceError;

pub struct Device {
    /// Loads device local functions.
    raw: ash::Device,
    adapter: Adapter,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    command_pool: vk::CommandPool,
}

impl Device {
    pub fn raw(&self) -> &ash::Device {
        &self.raw
    }

    pub fn graphics_queue(&self) -> vk::Queue {
        self.graphics_queue
    }

    pub fn present_queue(&self) -> vk::Queue {
        self.present_queue
    }

    pub fn command_pool(&self) -> vk::CommandPool {
        self.command_pool
    }

    pub fn adapter(&self) -> &Adapter {
        &self.adapter
    }

    pub fn queue_family_indices(&self) -> QueueFamilyIndices {
        self.adapter.queue_family_indices()
    }

    pub unsafe fn create(
        instance: &Instance,
        adapter: Adapter,
        adapter_req: &AdapterRequirements,
        device_req: &DeviceRequirements,
    ) -> Result<Device, DeviceError> {
        let instance_raw = instance.shared_instance().raw();

        let indices = &adapter.queue_family_indices();
        indices.log_debug();

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

        let enable_validation = instance
            .shared_instance()
            .flags()
            .contains(InstanceFlags::VALIDATION);
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
            .sample_rate_shading(adapter_req.sample_rate_shading)
            .build();

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
            .features(physical_device_features)
            .push_next(&mut ray_tracing_feature)
            .push_next(&mut acceleration_struct_feature)
            .push_next(&mut vulkan_12_features)
            .push_next(&mut vulkan_13_features);

        let device_extensions_ptrs = device_extensions_ptrs
            .iter()
            // Safe because `enabled_extensions` entries have static lifetime.
            .map(|e| e.as_ptr())
            .collect::<Vec<_>>();

        let adapter_raw = adapter.raw();
        let support_extensions = Self::check_device_extension_support(
            instance,
            adapter_raw,
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
            .push_next(&mut physical_device_features2);

        let ash_device: ash::Device =
            unsafe { instance_raw.create_device(adapter_raw, &device_create_info, None)? };

        log::debug!("Vulkan logical device created.");

        let device = Device::new(ash_device, adapter)?;
        Ok(device)
    }

    fn check_device_extension_support(
        instance: &Instance,
        adapter: vk::PhysicalDevice,
        extensions: &[&str],
        use_swapchain: bool,
    ) -> bool {
        let extension_props = unsafe {
            instance
                .shared_instance()
                .raw()
                .enumerate_device_extension_properties(adapter)
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

    pub(crate) fn new(raw: ash::Device, adapter: Adapter) -> Result<Self, DeviceError> {
        let indices = adapter.queue_family_indices();
        // this queue should support graphics and present
        let graphics_queue = unsafe { raw.get_device_queue(indices.graphics_family.unwrap(), 0) };
        let present_queue = unsafe { raw.get_device_queue(indices.present_family.unwrap(), 0) };
        let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(indices.graphics_family.unwrap())
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .build();
        let command_pool = unsafe { raw.create_command_pool(&command_pool_create_info, None)? };
        Ok(Self {
            raw,
            adapter,
            graphics_queue,
            present_queue,
            command_pool,
        })
    }

    pub unsafe fn wait_idle(&self) -> Result<(), DeviceError> {
        unsafe { self.raw.device_wait_idle()? }
        Ok(())
    }

    pub unsafe fn set_object_name(
        &self,
        object_type: vk::ObjectType,
        object: impl vk::Handle,
        name: &str,
    ) {
        let debug_utils = match &self.adapter.shared_instance().debug_utils() {
            Some(utils) => utils,
            None => return,
        };

        let mut buffer: [u8; 64] = [0u8; 64];
        let buffer_vec: Vec<u8>;

        // Append a null terminator to the string
        let name_bytes = if name.len() < buffer.len() {
            // Common case, string is very small. Allocate a copy on the stack.
            buffer[..name.len()].copy_from_slice(name.as_bytes());
            // Add null terminator
            buffer[name.len()] = 0;
            &buffer[..name.len() + 1]
        } else {
            // Less common case, the string is large.
            // This requires a heap allocation.
            buffer_vec = name
                .as_bytes()
                .iter()
                .cloned()
                .chain(std::iter::once(0))
                .collect();
            &buffer_vec
        };
        let extension = &debug_utils.extension;
        let _result = extension.set_debug_utils_object_name(
            self.raw.handle(),
            &vk::DebugUtilsObjectNameInfoEXT::builder()
                .object_type(object_type)
                .object_handle(object.as_raw())
                .object_name(CStr::from_bytes_with_nul_unchecked(name_bytes)),
        );
    }

    ///--------command queue--------

    pub unsafe fn allocate_command_buffer(
        &self,
        is_primary: bool,
    ) -> Result<CommandBuffer, DeviceError> {
        let mut command_buffer = unsafe { self.allocate_command_buffers(is_primary, 1)? };
        Ok(command_buffer.pop().unwrap())
    }

    pub unsafe fn allocate_command_buffers(
        &self,
        is_primary: bool,
        count: u32,
    ) -> Result<Vec<CommandBuffer>, DeviceError> {
        let level = if is_primary {
            vk::CommandBufferLevel::PRIMARY
        } else {
            vk::CommandBufferLevel::SECONDARY
        };
        let create_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.command_pool)
            .level(level)
            .command_buffer_count(count)
            .build();

        let command_buffers = unsafe { self.raw.allocate_command_buffers(&create_info)? };
        Ok(command_buffers
            .iter()
            .map(|x| CommandBuffer::new(*x))
            .collect())
    }

    pub unsafe fn free_command_buffer(&self, command_buffer: &mut CommandBuffer) {
        let command_buffers = [command_buffer.raw()];
        unsafe {
            self.raw
                .free_command_buffers(self.command_pool, &command_buffers);
        }
        command_buffer.set_state(CommandBufferState::NotAllocated)
    }

    pub unsafe fn begin_command_buffer(
        &self,
        command_buffer: &mut CommandBuffer,
        is_single_use: bool,
        is_render_pass_continue: bool,
        is_simultaneous: bool,
    ) -> Result<(), DeviceError> {
        // The inheritance_info parameter is only relevant for secondary command buffers.
        // It specifies which state to inherit from the calling primary command buffers.
        let inheritance = vk::CommandBufferInheritanceInfo::builder();
        let mut flags = vk::CommandBufferUsageFlags::empty();
        if is_single_use {
            flags |= vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT;
        }
        if is_render_pass_continue {
            flags |= vk::CommandBufferUsageFlags::RENDER_PASS_CONTINUE;
        }
        if is_simultaneous {
            flags |= vk::CommandBufferUsageFlags::SIMULTANEOUS_USE;
        }
        let info = vk::CommandBufferBeginInfo::builder()
            .flags(flags) // Optional.
            .inheritance_info(&inheritance)
            .build(); // Optional.

        unsafe {
            self.raw.begin_command_buffer(command_buffer.raw(), &info)?;
        }
        command_buffer.set_state(CommandBufferState::Recording);
        Ok(())
    }

    pub unsafe fn end_command_buffer(
        &self,
        command_buffer: &mut CommandBuffer,
    ) -> Result<(), DeviceError> {
        unsafe {
            self.raw.end_command_buffer(command_buffer.raw())?;
        }
        command_buffer.set_state(CommandBufferState::RecordingEnded);
        Ok(())
    }

    pub fn update_submitted_command_buffer(&self, command_buffer: &mut CommandBuffer) {
        command_buffer.set_state(CommandBufferState::Submitted);
    }

    pub unsafe fn reset_command_buffer(
        &self,
        command_buffer: &mut CommandBuffer,
    ) -> Result<(), DeviceError> {
        unsafe {
            self.raw
                .reset_command_buffer(command_buffer.raw(), vk::CommandBufferResetFlags::empty())?;
        }
        command_buffer.set_state(CommandBufferState::Ready);
        Ok(())
    }

    pub unsafe fn create_single_use<F>(&self, action: F) -> Result<(), DeviceError>
    where
        F: FnOnce(&Device, &CommandBuffer),
    {
        let mut command_buffer = unsafe { self.allocate_and_begin_single_use()? };
        action(&self, &command_buffer);
        unsafe { self.end_single_use(&mut command_buffer) }
    }

    pub unsafe fn allocate_and_begin_single_use(&self) -> Result<CommandBuffer, DeviceError> {
        unsafe {
            let mut command_buffer = self.allocate_command_buffer(true)?;
            self.begin_command_buffer(&mut command_buffer, true, false, false)?;
            Ok(command_buffer)
        }
    }

    pub unsafe fn end_single_use(
        &self,
        command_buffer: &mut CommandBuffer,
    ) -> Result<(), DeviceError> {
        unsafe {
            self.end_command_buffer(command_buffer)?;
        }
        let command_buffers = [command_buffer.raw()];
        let submit_info = vk::SubmitInfo::builder()
            .command_buffers(&command_buffers)
            .build();
        unsafe {
            self.raw
                .queue_submit(self.graphics_queue, &[submit_info], vk::Fence::default())?;
        }

        // since we dont use fence here, we wait for it to finish
        unsafe {
            self.raw.queue_wait_idle(self.graphics_queue)?;
        }
        unsafe {
            self.free_command_buffer(command_buffer);
        }
        Ok(())
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.raw.destroy_command_pool(self.command_pool, None);
            self.wait_idle().unwrap();
            self.raw.destroy_device(None);
            log::debug!("Device destroyed.");
        }
    }
}
