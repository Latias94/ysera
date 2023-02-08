use crate::vulkan::command_buffer::CommandBuffer;
use crate::vulkan::context::{Context, ContextDescriptor};
use crate::vulkan::device::DeviceFeatures;
use crate::vulkan::swapchain::{AcquiredImage, Swapchain, SwapchainDescriptor};
use crate::vulkan::sync::{Fence, Semaphore};
use crate::DeviceError;
use anyhow::Result;
use ash::vk;
use imgui::Ui;
use std::marker::PhantomData;
use std::time::Duration;
use typed_builder::TypedBuilder;
use winit::window::Window;
use ysera_imgui::gui::GuiContext;

pub trait RendererBase: Sized {
    type Gui: Gui;

    fn new(base: &mut BaseRenderer<Self>) -> Result<Self>;

    fn update(
        &mut self,
        base: &BaseRenderer<Self>,
        // gui: &mut Self::Gui,
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

pub struct BaseRenderer<R: RendererBase> {
    _phantom: PhantomData<R>,
    pub context: Context,
    pub swapchain: Swapchain,
    pub command_buffers: Vec<CommandBuffer>,
    in_flight_frames: InFlightFrames,
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
            queue_family: context.device.queue_family_indices(),
        };

        let swapchain = unsafe { Swapchain::new(&swapchain_desc)? };
        let command_buffers = unsafe {
            context
                .device
                .allocate_command_buffers(true, swapchain_desc.max_frame_in_flight)?
        };
        let in_flight_frames =
            unsafe { InFlightFrames::new(&context, config.max_frame_in_flight)? };

        Ok(Self {
            _phantom: Default::default(),
            context,
            swapchain,
            command_buffers,
            in_flight_frames,
        })
    }

    fn recreate_swapchain(&mut self, width: u32, height: u32) -> Result<()> {
        unsafe {
            self.context.device.raw().device_wait_idle()?;
            self.swapchain.resize(&self.context, width, height)?;
        }

        Ok(())
    }

    fn draw(
        &mut self,
        window: &Window,
        base_app: &mut R,
        // gui_context: &mut GuiContext,
        // gui: &mut R::Gui,
        frame_stats: &mut FrameStats,
    ) -> Result<bool, DeviceError> {
        self.in_flight_frames.next();
        unsafe {
            self.in_flight_frames.fence().wait(None)?;
        }

        let AcquiredImage { image_index, .. } = unsafe {
            self.swapchain
                .acquire_next_image(u64::MAX, self.in_flight_frames.image_available_semaphore())?
        };
        unsafe {
            self.in_flight_frames.fence().reset()?;
        }

        // Generate UI
        // gui_context
        //     .platform
        //     .prepare_frame(gui_context.imgui.io_mut(), window)?;
        // let ui = gui_context.imgui.frame();
        //
        // gui.build(&ui);
        // gui_context.platform.prepare_render(&ui, window);
        // let draw_data = gui_context.imgui.render();

        base_app.update(self, image_index as usize, frame_stats.frame_time)?;
        let command_buffer = &self.command_buffers[image_index as usize];

        // self.context.device.submit(
        //     command_buffer,
        //     Some(SemaphoreSubmitInfo {
        //         semaphore: self.in_flight_frames.image_available_semaphore(),
        //         stage_mask: vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
        //     }),
        //     Some(SemaphoreSubmitInfo {
        //         semaphore: self.in_flight_frames.render_finished_semaphore(),
        //         stage_mask: vk::PipelineStageFlags2::ALL_COMMANDS,
        //     }),
        //     self.in_flight_frames.fence(),
        // )?;

        let signal_semaphores = [self.in_flight_frames.render_finished_semaphore().raw];
        let suboptimal = unsafe {
            self.swapchain.queue_present(
                self.context.device.present_queue(),
                image_index as _,
                &signal_semaphores,
            )?
        };

        Ok(false)
    }

    fn record_command_buffer(
        &self,
        buffer: &CommandBuffer,
        image_index: usize,
        base_app: &R,
        // gui_renderer: &mut Renderer,
        // draw_data: &DrawData,
    ) -> Result<(), DeviceError> {
        let swapchain_image = &self.swapchain.swapchain_images[image_index];
        let swapchain_image_view = &self.swapchain.image_views[image_index];

        todo!()
    }
}

struct InFlightFrames {
    per_frames: Vec<PerFrame>,
    current_frame: usize,
}

struct PerFrame {
    image_available_semaphore: Semaphore,
    render_finished_semaphore: Semaphore,
    fence: Fence,
}

impl InFlightFrames {
    unsafe fn new(context: &Context, frame_count: u32) -> Result<Self> {
        let sync_objects = (0..frame_count)
            .map(|_i| {
                let image_available_semaphore = unsafe { context.create_semaphore()? };
                let render_finished_semaphore = unsafe { context.create_semaphore()? };
                let fence = unsafe { context.create_fence(Some(vk::FenceCreateFlags::SIGNALED))? };

                Ok(PerFrame {
                    image_available_semaphore,
                    render_finished_semaphore,
                    fence,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            per_frames: sync_objects,
            current_frame: 0,
        })
    }

    fn next(&mut self) {
        self.current_frame = (self.current_frame + 1) % self.per_frames.len();
    }

    fn image_available_semaphore(&self) -> &Semaphore {
        &self.per_frames[self.current_frame].image_available_semaphore
    }

    fn render_finished_semaphore(&self) -> &Semaphore {
        &self.per_frames[self.current_frame].render_finished_semaphore
    }

    fn fence(&self) -> &Fence {
        &self.per_frames[self.current_frame].fence
    }
}

#[derive(Debug)]
struct FrameStats {
    // we collect gpu timings the frame after it was computed
    // so we keep frame times for the two last frames
    previous_frame_time: Duration,
    frame_time: Duration,
    cpu_time: Duration,
    gpu_time: Duration,
    frame_time_ms_log: Queue<f32>,
    cpu_time_ms_log: Queue<f32>,
    gpu_time_ms_log: Queue<f32>,
    total_frame_count: u32,
    frame_count: u32,
    fps_counter: u32,
    timer: Duration,
}

impl Default for FrameStats {
    fn default() -> Self {
        Self {
            previous_frame_time: Default::default(),
            frame_time: Default::default(),
            cpu_time: Default::default(),
            gpu_time: Default::default(),
            frame_time_ms_log: Queue::new(FrameStats::MAX_LOG_SIZE),
            cpu_time_ms_log: Queue::new(FrameStats::MAX_LOG_SIZE),
            gpu_time_ms_log: Queue::new(FrameStats::MAX_LOG_SIZE),
            total_frame_count: Default::default(),
            frame_count: Default::default(),
            fps_counter: Default::default(),
            timer: Default::default(),
        }
    }
}

impl FrameStats {
    const ONE_SEC: Duration = Duration::from_secs(1);
    const MAX_LOG_SIZE: usize = 1000;

    fn tick(&mut self) {
        // compute cpu time
        self.cpu_time = self.previous_frame_time.saturating_sub(self.gpu_time);

        // push log
        self.frame_time_ms_log
            .push(self.previous_frame_time.as_millis() as _);
        self.cpu_time_ms_log.push(self.cpu_time.as_millis() as _);
        self.gpu_time_ms_log.push(self.gpu_time.as_millis() as _);

        // increment counter
        self.total_frame_count += 1;
        self.frame_count += 1;
        self.timer += self.frame_time;

        // reset counter if a sec has passed
        if self.timer > FrameStats::ONE_SEC {
            self.fps_counter = self.frame_count;
            self.frame_count = 0;
            self.timer -= FrameStats::ONE_SEC;
        }
    }

    fn set_frame_time(&mut self, frame_time: Duration) {
        self.previous_frame_time = self.frame_time;
        self.frame_time = frame_time;
    }

    fn set_gpu_time_time(&mut self, gpu_time: Duration) {
        self.gpu_time = gpu_time;
    }
}

#[derive(Debug)]
struct Queue<T>(Vec<T>, usize);

impl<T> Queue<T> {
    fn new(max_size: usize) -> Self {
        Self(Vec::with_capacity(max_size), max_size)
    }

    fn push(&mut self, value: T) {
        if self.0.len() == self.1 {
            self.0.remove(0);
        }
        self.0.push(value);
    }
}
