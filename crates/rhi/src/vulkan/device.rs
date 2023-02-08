use std::ffi::CStr;
use std::sync::Arc;

use ash::vk;

use crate::vulkan::adapter::AdapterShared;
use crate::vulkan::command_buffer::{CommandBuffer, CommandBufferState};
use crate::vulkan::debug::DebugUtils;
use crate::{DeviceError, QueueFamilyIndices};

pub struct Device {
    /// Loads device local functions.
    raw: ash::Device,
    debug_utils: Option<DebugUtils>,
    adapter: Arc<AdapterShared>,

    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    command_pool: vk::CommandPool,
    indices: QueueFamilyIndices,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DeviceFeatures {
    pub ray_tracing_pipeline: bool,
    pub acceleration_structure: bool,
    pub runtime_descriptor_array: bool,
    pub buffer_device_address: bool,
    pub dynamic_rendering: bool,
    pub synchronization2: bool,
}

impl DeviceFeatures {
    pub fn is_compatible_with(&self, requirements: &Self) -> bool {
        (!requirements.ray_tracing_pipeline || self.ray_tracing_pipeline)
            && (!requirements.acceleration_structure || self.acceleration_structure)
            && (!requirements.runtime_descriptor_array || self.runtime_descriptor_array)
            && (!requirements.buffer_device_address || self.buffer_device_address)
            && (!requirements.dynamic_rendering || self.dynamic_rendering)
            && (!requirements.synchronization2 || self.synchronization2)
    }
}

impl Device {
    pub(crate) fn raw(&self) -> &ash::Device {
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

    pub fn queue_family_indices(&self) -> QueueFamilyIndices {
        self.indices
    }

    pub(crate) fn new(
        raw: ash::Device,
        debug_utils: Option<DebugUtils>,
        adapter: Arc<AdapterShared>,
        indices: QueueFamilyIndices,
    ) -> Result<Self, DeviceError> {
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
            debug_utils,
            adapter,
            graphics_queue,
            present_queue,
            command_pool,
            indices,
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
        let debug_utils = match &self.debug_utils {
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
        }
    }
}
