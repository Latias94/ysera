use crate::vulkan_v2::device::Device;
use crate::DeviceError;
use ash::vk;
use ash::vk::CommandBufferResetFlags;
use std::ops::Deref;

#[derive(Clone)]
pub struct CommandBufferAllocator {
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

    pub fn new(
        device: &ash::Device,
        queue_family_index: u32,
        queue: vk::Queue,
    ) -> Result<Self, DeviceError> {
        let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .build();
        let command_pool = unsafe { device.create_command_pool(&command_pool_create_info, None)? };

        Ok(Self {
            command_pool,
            queue,
        })
    }

    pub unsafe fn allocate_command_buffer(
        &self,
        device: &ash::Device,
        is_primary: bool,
    ) -> Result<CommandBuffer, DeviceError> {
        let mut command_buffer = unsafe { self.allocate_command_buffers(device, is_primary, 1)? };
        Ok(command_buffer.pop().unwrap())
    }

    pub unsafe fn allocate_command_buffers(
        &self,
        device: &ash::Device,
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

        let command_buffers = unsafe { device.allocate_command_buffers(&create_info)? };
        Ok(command_buffers
            .iter()
            .map(|x| CommandBuffer::new(*x))
            .collect())
    }

    pub unsafe fn free_command_buffer(
        &self,
        device: &ash::Device,
        command_buffer: &mut CommandBuffer,
    ) {
        let command_buffers = [command_buffer.raw()];
        unsafe {
            device.free_command_buffers(self.command_pool, &command_buffers);
        }
        command_buffer.set_state(CommandBufferState::NotAllocated)
    }

    pub unsafe fn begin_command_buffer(
        &self,
        device: &ash::Device,
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
            device.begin_command_buffer(command_buffer.raw(), &info)?;
        }
        command_buffer.set_state(CommandBufferState::Recording);
        Ok(())
    }

    pub unsafe fn end_command_buffer(
        &self,
        device: &ash::Device,
        command_buffer: &mut CommandBuffer,
    ) -> Result<(), DeviceError> {
        unsafe {
            device.end_command_buffer(command_buffer.raw())?;
        }
        command_buffer.set_state(CommandBufferState::RecordingEnded);
        Ok(())
    }

    pub fn update_submitted_command_buffer(&self, command_buffer: &mut CommandBuffer) {
        command_buffer.set_state(CommandBufferState::Submitted);
    }

    pub unsafe fn reset_command_buffer(
        &self,
        device: &ash::Device,
        command_buffer: &mut CommandBuffer,
    ) -> Result<(), DeviceError> {
        unsafe {
            device.reset_command_buffer(command_buffer.raw(), CommandBufferResetFlags::empty())?;
        }
        command_buffer.set_state(CommandBufferState::Ready);
        Ok(())
    }

    pub unsafe fn create_single_use<F>(&self, device: &Device, action: F) -> Result<(), DeviceError>
    where
        F: FnOnce(&Device, &CommandBuffer),
    {
        let mut command_buffer = unsafe { self.allocate_and_begin_single_use(device.raw())? };
        action(device, &command_buffer);
        unsafe { self.end_single_use(device.raw(), &mut command_buffer) }
    }

    pub unsafe fn allocate_and_begin_single_use(
        &self,
        device: &ash::Device,
    ) -> Result<CommandBuffer, DeviceError> {
        let mut command_buffer = self.allocate_command_buffer(device, true)?;
        unsafe {
            self.begin_command_buffer(device, &mut command_buffer, true, false, false)?;
        }
        Ok(command_buffer)
    }

    pub unsafe fn end_single_use(
        &self,
        device: &ash::Device,
        command_buffer: &mut CommandBuffer,
    ) -> Result<(), DeviceError> {
        unsafe {
            self.end_command_buffer(device, command_buffer)?;
        }

        let command_buffers = [command_buffer.raw()];
        let submit_info = vk::SubmitInfo::builder()
            .command_buffers(&command_buffers)
            .build();

        unsafe {
            device.queue_submit(self.queue, &[submit_info], vk::Fence::default())?;
            // since we dont use fence here, we wait for it to finish
            device.queue_wait_idle(self.queue)?;
            self.free_command_buffer(device, command_buffer);
        }

        Ok(())
    }

    pub unsafe fn destroy(&self, device: &ash::Device) {
        unsafe {
            device.destroy_command_pool(self.command_pool, None);
        }
    }
}

pub enum CommandBufferState {
    /// ready to begin
    Ready,
    Recording,
    InRenderPass,
    RecordingEnded,
    Submitted,
    NotAllocated,
}

pub struct CommandBuffer {
    raw: vk::CommandBuffer,
    state: CommandBufferState,
}

impl CommandBuffer {
    pub fn raw(&self) -> vk::CommandBuffer {
        self.raw
    }

    pub fn set_state(&mut self, state: CommandBufferState) {
        self.state = state;
    }

    pub fn new(raw: vk::CommandBuffer) -> Self {
        Self {
            raw,
            state: CommandBufferState::Ready,
        }
    }
}

impl Deref for CommandBuffer {
    type Target = vk::CommandBuffer;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}
