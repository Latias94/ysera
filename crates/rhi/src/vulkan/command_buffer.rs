use std::ops::Deref;

use ash::vk;

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
