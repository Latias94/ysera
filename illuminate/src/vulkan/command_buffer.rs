use crate::vulkan::command_buffer::CommandBufferState::Ready;
use ash::vk;
use std::ops::Deref;

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

    pub fn new(raw: vk::CommandBuffer) -> Self {
        Self { raw, state: Ready }
    }
}

impl Deref for CommandBuffer {
    type Target = vk::CommandBuffer;

    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}
