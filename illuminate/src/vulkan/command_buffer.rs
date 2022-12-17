use crate::vulkan::device::Device;
use crate::DeviceError;
use ash::vk;
use std::rc::Rc;

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

    pub fn create_single_use<F>(&self, action: F) -> Result<(), DeviceError>
    where
        F: FnOnce(&Rc<Device>, &vk::CommandBuffer),
    {
        let command_buffer = self.allocate_and_begin_single_use()?;
        action(&self.device, &command_buffer);
        self.end_single_use(command_buffer)
    }

    pub fn allocate_and_begin_single_use(&self) -> Result<vk::CommandBuffer, DeviceError> {
        let create_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1)
            .build();

        let command_buffers = self.device.allocate_command_buffers(&create_info)?;

        // The inheritance_info parameter is only relevant for secondary command buffers.
        // It specifies which state to inherit from the calling primary command buffers.
        let inheritance = vk::CommandBufferInheritanceInfo::builder();

        let info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::empty()) // Optional.
            .inheritance_info(&inheritance)
            .build(); // Optional.

        let command_buffer = command_buffers[0];
        self.device.begin_command_buffer(command_buffer, &info)?;
        Ok(command_buffer)
    }

    pub fn end_single_use(&self, command_buffer: vk::CommandBuffer) -> Result<(), DeviceError> {
        self.end_command_buffer(command_buffer)?;
        let submit_info = vk::SubmitInfo::builder()
            .command_buffers(&[command_buffer])
            .build();
        self.device
            .queue_submit(self.queue, &[submit_info], vk::Fence::default())?;

        self.device.queue_wait_idle(self.queue)?;
        self.device
            .free_command_buffers(self.command_pool, &[command_buffer]);
        Ok(())
    }

    fn end_command_buffer(&self, command_buffer: vk::CommandBuffer) -> Result<(), DeviceError> {
        self.device.end_command_buffer(command_buffer)
    }

    fn reset_command_buffer(&self, command_buffer: vk::CommandBuffer) -> Result<(), DeviceError> {
        self.device.end_command_buffer(command_buffer)
    }
}
