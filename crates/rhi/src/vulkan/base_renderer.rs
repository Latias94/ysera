use crate::vulkan::command_buffer::CommandBuffer;
use crate::vulkan::context::{Context, ContextDescriptor};
use crate::vulkan::device::DeviceFeatures;
use crate::vulkan::swapchain::{Swapchain, SwapchainDescriptor};
use anyhow::Result;
use ash::vk;
use imgui::Ui;
use std::marker::PhantomData;
use std::time::Duration;
use typed_builder::TypedBuilder;
use winit::window::Window;

pub struct BaseRenderer<R: RendererBase> {
    _phantom: PhantomData<R>,
    pub context: Context,
    pub swapchain: Swapchain,
    command_buffers: Vec<CommandBuffer>,
}

pub trait RendererBase: Sized {
    type Gui: Gui;

    fn new(base: &mut BaseRenderer<Self>) -> Result<Self>;

    fn update(
        &mut self,
        base: &BaseRenderer<Self>,
        gui: &mut Self::Gui,
        image_index: usize,
        delta_time: Duration,
    ) -> Result<()>;

    fn record_commands(&self, base: &BaseRenderer<Self>, image_index: usize) -> Result<()> {
        Ok(())
    }

    fn on_recreate_swapchain(&mut self) -> Result<()>;
}

pub trait Gui: Sized {
    fn new() -> Result<Self>;

    fn build(&mut self, ui: &Ui);
}

impl Gui for () {
    fn new() -> Result<Self> {
        Ok(())
    }

    fn build(&mut self, _ui: &Ui) {}
}

#[derive(TypedBuilder)]
pub struct RHIConfig<'a> {
    #[builder(default = "Ysera App")]
    pub app_name: &'a str,
    #[builder(default = 2)]
    pub max_frame_in_flight: u32,
}

impl<R: RendererBase> BaseRenderer<R> {
    fn new(window: &Window, config: RHIConfig) -> Result<Self> {
        let required_extensions = vec!["VK_KHR_swapchain"];

        let context_desc = ContextDescriptor {
            app_name: config.app_name,
            window_handle: window,
            display_handle: window,
            vulkan_version: vk::API_VERSION_1_3,
            required_extensions: &required_extensions,
            device_feature: DeviceFeatures::default(),
        };

        let context = unsafe { Context::new(context_desc)? };
        let inner_size = window.inner_size();

        let swapchain_desc = SwapchainDescriptor {
            context: &context,
            dimensions: [inner_size.width, inner_size.height],
            old_swapchain: None,
            max_frame_in_flight: config.max_frame_in_flight,
            queue_family: context.indices,
        };

        let swapchain = unsafe { Swapchain::new(&swapchain_desc)? };
        let command_buffers = unsafe {
            context
                .command_buffer_allocator
                .allocate_command_buffers(true, swapchain_desc.max_frame_in_flight)?
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
