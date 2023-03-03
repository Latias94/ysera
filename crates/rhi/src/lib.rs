#![allow(clippy::missing_safety_doc)]

extern crate alloc;
#[macro_use]
extern crate num_derive;

use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
pub use winit;

pub use error::*;
use rhi_types::{RHIExtent2D, RHIRect2D, RHISubpassContents, RHIViewport};

use crate::types_v2::{
    RHICommandBufferLevel, RHICommandPoolCreateInfo, RHIDescriptorSetLayoutCreateInfo,
    RHIFramebufferCreateInfo, RHIGraphicsPipelineCreateInfo, RHIPipelineBindPoint,
    RHIPipelineLayoutCreateInfo, RHIRenderPassBeginInfo, RHIRenderPassCreateInfo,
    RHIShaderCreateInfo, RHISwapChainDesc,
};

mod error;
mod gui;
pub mod types;
pub mod types_v2;
pub mod utils;
pub mod vulkan_v2;

const MAX_FRAMES_IN_FLIGHT: u8 = 3;

pub trait RHI: Sized + Send + Sync + Clone {
    type CommandPool;
    type CommandBuffer;
    type RenderPass: Copy + Clone;
    type Image: Copy + Clone;
    type ImageView: Copy + Clone;
    type Allocation;
    type Format: Copy + Clone;
    type Framebuffer: Copy + Clone;
    type DescriptorSet: Copy + Clone;
    type DescriptorSetLayout: Copy + Clone;
    type PipelineLayout: Copy + Clone;
    type Pipeline: Default + Copy + Clone;
    type Sampler: Copy + Clone;
    type Shader: Copy + Clone;
    type Buffer;

    unsafe fn initialize(init_info: InitInfo) -> Result<Self, RHIError>;
    unsafe fn prepare_context(&mut self);
    unsafe fn recreate_swapchain(&mut self, size: RHIExtent2D) -> Result<(), RHIError>;
    unsafe fn wait_for_fences(&mut self) -> Result<(), RHIError>;
    unsafe fn reset_command_pool(&mut self) -> Result<(), RHIError>;
    unsafe fn get_current_frame_index(&self) -> usize;
    unsafe fn prepare_before_render_pass<F>(
        &mut self,
        pass_update_after_recreate_swapchain: F,
    ) -> Result<bool, RHIError>
    where
        F: FnOnce();
    unsafe fn submit_rendering<F>(
        &mut self,
        pass_update_after_recreate_swapchain: F,
    ) -> Result<(), RHIError>
    where
        F: FnOnce();

    unsafe fn allocate_command_buffers(
        &self,
        allocate_info: CommandBufferAllocateInfo<Self>,
    ) -> Result<Vec<Self::CommandBuffer>, RHIError>;

    unsafe fn create_command_pool(
        &self,
        create_info: &RHICommandPoolCreateInfo,
    ) -> Result<Self::CommandPool, RHIError>;

    unsafe fn create_render_pass(
        &self,
        create_info: &RHIRenderPassCreateInfo,
    ) -> Result<Self::RenderPass, RHIError>;

    unsafe fn create_framebuffer(
        &self,
        create_info: &RHIFramebufferCreateInfo<Self>,
    ) -> Result<Self::Framebuffer, RHIError>;

    unsafe fn create_shader_module(
        &self,
        create_info: &RHIShaderCreateInfo,
    ) -> Result<Self::Shader, RHIError>;

    unsafe fn create_descriptor_set_layout(
        &self,
        create_info: &RHIDescriptorSetLayoutCreateInfo,
    ) -> Result<Self::DescriptorSetLayout, RHIError>;

    unsafe fn create_pipeline_layout(
        &self,
        create_info: &RHIPipelineLayoutCreateInfo<Self>,
    ) -> Result<Self::PipelineLayout, RHIError>;

    unsafe fn create_graphics_pipeline(
        &self,
        create_info: &RHIGraphicsPipelineCreateInfo<Self>,
    ) -> Result<Self::Pipeline, RHIError>;

    fn get_swapchain_info(&self) -> RHISwapChainDesc<Self>;

    fn get_current_command_buffer(&self) -> Self::CommandBuffer;

    unsafe fn cmd_begin_render_pass(
        &self,
        command_buffer: Self::CommandBuffer,
        begin_info: &RHIRenderPassBeginInfo<Self>,
        subpass_contents: RHISubpassContents,
    );

    unsafe fn cmd_end_render_pass(&self, command_buffer: Self::CommandBuffer);

    unsafe fn cmd_set_viewport(
        &self,
        command_buffer: Self::CommandBuffer,
        first_viewport: u32,
        viewports: &[RHIViewport],
    );

    unsafe fn cmd_set_scissor(
        &self,
        command_buffer: Self::CommandBuffer,
        first_scissor: u32,
        scissors: &[RHIRect2D],
    );

    // unsafe fn cmd_bind_index_buffer(
    //     &self,
    //     command_buffer: Self::CommandBuffer,
    //     buffer: Self::Buffer,
    //     offset: RHIDeviceSize,
    //     index_type: RHIIndexType,
    // );

    unsafe fn cmd_bind_pipeline(
        &self,
        command_buffer: Self::CommandBuffer,
        pipeline_bind_point: RHIPipelineBindPoint,
        pipeline: Self::Pipeline,
    );

    unsafe fn cmd_draw(
        &self,
        command_buffer: Self::CommandBuffer,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    );

    unsafe fn destroy_shader_module(&self, shader: Self::Shader);

    unsafe fn destroy_sampler(&self, sampler: Self::Sampler);

    unsafe fn destroy_image(&self, image: Self::Image);

    unsafe fn destroy_image_view(&self, image_view: Self::ImageView);

    unsafe fn destroy_framebuffer(&self, framebuffer: Self::Framebuffer);

    unsafe fn destroy_buffer(&self, buffer: Self::Buffer);

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
