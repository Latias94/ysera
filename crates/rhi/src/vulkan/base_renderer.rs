use crate::vulkan::adapter::Adapter;
use crate::vulkan::command_buffer::CommandBuffer;
use crate::vulkan::command_buffer_allocator::CommandBufferAllocator;
use crate::vulkan::context::{Context, ContextDescriptor};
use crate::vulkan::device::{Device, DeviceFeatures};
use crate::vulkan::instance::Instance;
use crate::vulkan::surface::Surface;
use crate::vulkan::swapchain::{Swapchain, SwapchainDescriptor};
use crate::QueueFamilyIndices;
use anyhow::Result;
use ash::vk;
use gpu_allocator::vulkan::Allocator;
use imgui::Ui;
use parking_lot::Mutex;
use raw_window_handle::HasRawWindowHandle;
use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Duration;
use winit::window::Window;

const MAX_FRAMES_IN_FLIGHT: u32 = 2;

pub struct BaseRenderer<R: Renderer> {
    _phantom: PhantomData<R>,
    pub context: Context,
    pub swapchain: Swapchain,
    command_buffers: Vec<CommandBuffer>,
}

pub trait Renderer: Sized {
    type Gui: Gui;

    fn new(base: &mut BaseRenderer<Self>) -> Result<Self>;

    fn update(
        &mut self,
        base: &BaseRenderer<Self>,
        gui: &mut Self::Gui,
        image_index: usize,
        delta_time: Duration,
    ) -> Result<()>;

    fn record_commands(
        &self,
        base: &BaseRenderer<Self>,
        buffer: &CommandBuffer,
        image_index: usize,
    ) -> Result<()> {
        Ok(())
    }

    fn on_recreate_swapchain(&mut self, base: &BaseRenderer<Self>) -> Result<()>;
}

pub trait Gui: Sized {
    fn new() -> Result<Self>;

    fn build(&mut self, ui: &Ui);
}

impl<R: Renderer> BaseRenderer<R> {
    fn new(window: &Window, app_name: &str) -> Result<Self> {
        let required_extensions = vec!["VK_KHR_swapchain"];

        let context_desc = ContextDescriptor {
            app_name,
            window_handle: window,
            display_handle: window,
            vulkan_version: vk::API_VERSION_1_3,
            required_extensions: &required_extensions,
            device_feature: DeviceFeatures {},
        };

        let context = unsafe { Context::new(context_desc)? };
        let inner_size = window.inner_size();

        let swapchain_desc = SwapchainDescriptor {
            context: &context,
            dimensions: [inner_size.width, inner_size.height],
            old_swapchain: None,
            max_frame_in_flight: MAX_FRAMES_IN_FLIGHT,
            queue_family: context.indices,
        };

        let swapchain = unsafe { Swapchain::new(&swapchain_desc)? };
        let command_buffers = unsafe {
            context
                .command_buffer_allocator
                .allocate_command_buffers(true, MAX_FRAMES_IN_FLIGHT)?
        };

        Ok(Self {
            _phantom: Default::default(),
            context,
            swapchain,
            command_buffers,
        })
    }

    fn recreate_swapchain(&mut self, width: u32, height: u32) -> Result<()> {
        unsafe {
            self.context.device.raw().device_wait_idle()?;
            self.swapchain.resize(&self.context, width, height)?;
        }

        Ok(())
    }
}
