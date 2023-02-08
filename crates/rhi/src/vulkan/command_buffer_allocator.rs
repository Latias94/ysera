use std::sync::Arc;

use ash::vk;
use ash::vk::CommandBufferResetFlags;

use crate::vulkan::command_buffer::{CommandBuffer, CommandBufferState};
use crate::vulkan::device::Device;
use crate::DeviceError;

#[derive(Clone)]
pub struct CommandBufferAllocator {
    device: Arc<Device>,
    queue: vk::Queue,
    command_pool: vk::CommandPool,
}

impl CommandBufferAllocator {
    pub fn queue(&self) -> vk::Queue {
        self.queue
    }
    pub fn command_pool(&self) -> vk::CommandPool {
        self.command_pool
    }

    pub fn new(device: &Arc<Device>, command_pool: vk::CommandPool, queue: vk::Queue) -> Self {
        Self {
            device: device.clone(),
            command_pool,
            queue,
        }
    }

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

        let command_buffers = unsafe { self.device.raw().allocate_command_buffers(&create_info)? };
        Ok(command_buffers
            .iter()
            .map(|x| CommandBuffer::new(*x))
            .collect())
    }

    pub unsafe fn free_command_buffer(&self, command_buffer: &mut CommandBuffer) {
        let command_buffers = [command_buffer.raw()];
        unsafe {
            self.device
                .raw()
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
            self.device
                .raw()
                .begin_command_buffer(command_buffer.raw(), &info)?;
        }
        command_buffer.set_state(CommandBufferState::Recording);
        Ok(())
    }

    pub unsafe fn end_command_buffer(
        &self,
        command_buffer: &mut CommandBuffer,
    ) -> Result<(), DeviceError> {
        unsafe {
            self.device.raw().end_command_buffer(command_buffer.raw())?;
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
            self.device
                .raw()
                .reset_command_buffer(command_buffer.raw(), CommandBufferResetFlags::empty())?;
        }
        command_buffer.set_state(CommandBufferState::Ready);
        Ok(())
    }

    pub unsafe fn create_single_use<F>(&self, action: F) -> Result<(), DeviceError>
    where
        F: FnOnce(&Arc<Device>, &CommandBuffer),
    {
        let mut command_buffer = unsafe { self.allocate_and_begin_single_use()? };
        action(&self.device, &command_buffer);
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
            self.device
                .raw()
                .queue_submit(self.queue, &[submit_info], vk::Fence::default())?;
        }

        // since we dont use fence here, we wait for it to finish
        unsafe {
            self.device.raw().queue_wait_idle(self.queue)?;
        }
        unsafe {
            self.free_command_buffer(command_buffer);
        }
        Ok(())
    }
}
