use crate::vulkan::command_buffer::{CommandBuffer, CommandBufferState};
use crate::vulkan::device::Device;
use crate::DeviceError;
use ash::vk;
use ash::vk::CommandBufferResetFlags;
use std::rc::Rc;

#[derive(Clone)]
pub struct CommandBufferAllocator {
    device: Rc<Device>,
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

    pub fn new(device: &Rc<Device>, command_pool: vk::CommandPool, queue: vk::Queue) -> Self {
        Self {
            device: device.clone(),
            command_pool,
            queue,
        }
    }

    pub fn allocate_command_buffer(&self, is_primary: bool) -> Result<CommandBuffer, DeviceError> {
        let mut command_buffer = self.allocate_command_buffers(is_primary, 1)?;
        Ok(command_buffer.pop().unwrap())
    }

    pub fn allocate_command_buffers(
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

        let command_buffers = self.device.allocate_command_buffers(&create_info)?;
        Ok(command_buffers
            .iter()
            .map(|x| CommandBuffer::new(*x))
            .collect())
    }

    pub fn free_command_buffer(&self, command_buffer: &mut CommandBuffer) {
        let command_buffers = [command_buffer.raw()];
        self.device
            .free_command_buffers(self.command_pool, &command_buffers);
        command_buffer.set_state(CommandBufferState::NotAllocated)
    }

    pub fn begin_command_buffer(
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

        self.device
            .begin_command_buffer(command_buffer.raw(), &info)?;
        command_buffer.set_state(CommandBufferState::Recording);
        Ok(())
    }

    pub fn end_command_buffer(
        &self,
        command_buffer: &mut CommandBuffer,
    ) -> Result<(), DeviceError> {
        self.device.end_command_buffer(command_buffer.raw())?;
        command_buffer.set_state(CommandBufferState::RecordingEnded);
        Ok(())
    }

    pub fn update_submitted_command_buffer(&self, command_buffer: &mut CommandBuffer) {
        command_buffer.set_state(CommandBufferState::Submitted);
    }

    pub fn reset_command_buffer(
        &self,
        command_buffer: &mut CommandBuffer,
    ) -> Result<(), DeviceError> {
        self.device
            .reset_command_buffer(command_buffer.raw(), CommandBufferResetFlags::empty())?;
        command_buffer.set_state(CommandBufferState::Ready);
        Ok(())
    }

    pub fn create_single_use<F>(&self, action: F) -> Result<(), DeviceError>
    where
        F: FnOnce(&Rc<Device>, &CommandBuffer),
    {
        let mut command_buffer = self.allocate_and_begin_single_use()?;
        action(&self.device, &command_buffer);
        self.end_single_use(&mut command_buffer)
    }

    pub fn allocate_and_begin_single_use(&self) -> Result<CommandBuffer, DeviceError> {
        let mut command_buffer = self.allocate_command_buffer(true)?;
        self.begin_command_buffer(&mut command_buffer, true, false, false)?;
        Ok(command_buffer)
    }

    pub fn end_single_use(&self, command_buffer: &mut CommandBuffer) -> Result<(), DeviceError> {
        self.end_command_buffer(command_buffer)?;

        let command_buffers = [command_buffer.raw()];
        let submit_info = vk::SubmitInfo::builder()
            .command_buffers(&command_buffers)
            .build();
        self.device
            .queue_submit(self.queue, &[submit_info], vk::Fence::default())?;

        // since we dont use fence here, we wait for it to finish
        self.device.queue_wait_idle(self.queue)?;
        self.free_command_buffer(command_buffer);
        Ok(())
    }
}
