#![allow(clippy::missing_safety_doc)]

extern crate alloc;
#[macro_use]
extern crate num_derive;

pub use ash;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
pub use winit;

use crate::types_v2::{
    RHICommandBufferLevel, RHICommandPoolCreateInfo, RHIExtent2D, RHIRenderPassCreateInfo,
};
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
    type RenderPass;

    unsafe fn initialize(init_info: InitInfo) -> Result<Self, RHIError>;
    unsafe fn prepare_context(&mut self);
    unsafe fn recreate_swapchain(&mut self, size: RHIExtent2D) -> Result<(), RHIError>;
    unsafe fn wait_for_fences(&mut self) -> Result<(), RHIError>;
    unsafe fn prepare_before_render_pass<F>(
        &mut self,
        pass_update_after_recreate_swapchain: F,
    ) -> Result<(), RHIError>
    where
        F: FnOnce();

    unsafe fn allocate_command_buffers(
        &self,
        allocate_info: CommandBufferAllocateInfo<Self>,
    ) -> Result<Vec<CommandBuffer>, RHIError>;

    unsafe fn create_command_pool(
        &self,
        create_info: &RHICommandPoolCreateInfo,
    ) -> Result<Self::CommandPool, RHIError>;

    unsafe fn create_render_pass(
        &self,
        create_info: &RHIRenderPassCreateInfo,
    ) -> Result<Self::RenderPass, RHIError>;

    unsafe fn clear(&mut self);
}

pub struct InitInfo<'a> {
    pub window_size: RHIExtent2D,
    pub window_handle: &'a dyn HasRawWindowHandle,
    pub display_handle: &'a dyn HasRawDisplayHandle,
}

pub struct CommandBufferAllocateInfo<R: RHI> {
    pub command_pool: R::CommandPool,
    pub level: RHICommandBufferLevel,
    pub count: u32,
}
