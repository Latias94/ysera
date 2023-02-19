#![allow(clippy::missing_safety_doc)]

extern crate alloc;

pub use ash;
pub use winit;

use crate::types_v2::{RHICommandBufferLevel, RHICommandPoolCreateInfo};
use crate::vulkan::command_buffer::CommandBuffer;
pub use error::*;
use winit::window::Window as WinitWindow;

mod error;
mod gui;
pub mod types;
pub mod types_v2;
pub mod utils;
pub mod vulkan;
pub mod vulkan_v2;

const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub trait RHI: Sized {
    type CommandPool;
    type CommandBuffer;

    fn initialize(init_info: InitInfo) -> Result<Self, RHIError>;
    fn prepare_context();

    fn allocate_command_buffers(
        &self,
        allocate_info: CommandBufferAllocateInfo<Self>,
    ) -> Result<CommandBuffer, RHIError>;

    fn create_command_pool(
        &self,
        create_info: &RHICommandPoolCreateInfo,
    ) -> Result<Self::CommandPool, RHIError>;
}

pub struct InitInfo<'a> {
    pub window: &'a WinitWindow,
}

pub struct CommandBufferAllocateInfo<R: RHI> {
    pub command_pool: R::CommandPool,
    pub level: RHICommandBufferLevel,
    pub count: u32,
}
