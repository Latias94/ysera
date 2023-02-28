use std::time::{Duration, Instant};

use rhi::vulkan_v2::VulkanRHI;
use rhi::{InitInfo, RHI};
use rhi_types::RHIExtent2D;

type Api = VulkanRHI;

struct Sandbox {
    rhi: Api,
    // main_camera_pass: MainCameraPass<Api>,
    temp: bool,
}

impl Sandbox {
    fn new(window: &winit::window::Window) -> anyhow::Result<Self> {
        let window_size = window.inner_size();
        let init_info = InitInfo {
            window_size: RHIExtent2D {
                width: window_size.width,
                height: window_size.height,
            },
            window_handle: &window,
            display_handle: &window,
        };

        let rhi = unsafe { Api::initialize(init_info)? };
        // let rhi = Arc::new(rhi);
        //
        // let pass_init_info = MainCameraPassInitInfo { rhi: rhi.clone() };
        // let main_camera_pass = MainCameraPass::initialize(&pass_init_info)?;

        Ok(Self {
            rhi,
            // main_camera_pass,
            temp: false,
        })
    }

    fn update(&mut self, delta_time: Duration) -> anyhow::Result<()> {
        if self.temp {
            return Ok(());
        }
        self.temp = true;
        unsafe {
            self.rhi.prepare_context();

            self.rhi.wait_for_fences()?;
            self.rhi.reset_command_pool()?;
            let recreate_swapchain = self.rhi.prepare_before_render_pass(|| {})?;
            if recreate_swapchain {
                return Ok(());
            }
            // pass...
            self.rhi.submit_rendering(|| {})?;
        }
        Ok(())
    }

    fn on_recreate_swapchain(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    fn on_exit(&mut self) {}
}

fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_BACKTRACE", "full");
    std::env::set_var("RUST_LOG", "debug");

    // profiling::tracy_client::Client::start();

    let mut builder = env_logger::Builder::from_default_env();
    builder.target(env_logger::Target::Stdout);
    builder.init();

    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("Ysera example")
        .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
        .build(&event_loop)
        .unwrap();
    let mut last_frame_inst = Instant::now();
    let (mut frame_count, mut accum_time) = (0, 0.0);

    let mut sandbox = Some(Sandbox::new(&window)?);

    event_loop.run(move |event, _, control_flow| {
        let _ = &window; // force ownership by the closure
        *control_flow = winit::event_loop::ControlFlow::Poll;
        match event {
            winit::event::Event::MainEventsCleared => {
                window.request_redraw();
            }
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::KeyboardInput {
                    input:
                        winit::event::KeyboardInput {
                            virtual_keycode: Some(winit::event::VirtualKeyCode::Escape),
                            state: winit::event::ElementState::Pressed,
                            ..
                        },
                    ..
                }
                | winit::event::WindowEvent::CloseRequested => {
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }
                _ => {
                    // triangle.as_mut().unwrap().update(event);
                }
            },
            winit::event::Event::RedrawRequested(_window_id) => {
                let now = Instant::now();
                let delta_time = last_frame_inst.elapsed();
                let app = sandbox.as_mut().unwrap();
                {
                    accum_time += last_frame_inst.elapsed().as_secs_f32();
                    last_frame_inst = now;
                    frame_count += 1;
                    if accum_time >= 1f32 {
                        // second per frame
                        let avg_frame_time = accum_time / frame_count as f32;
                        let frame_rate = (1f32 / avg_frame_time).round() as i32;
                        let text = format!("Ysera Engine | FPS: {}", frame_rate);
                        window.set_title(text.as_str());
                        accum_time = 0.0;
                        frame_count = 0;
                    }
                }

                app.update(delta_time).unwrap();

                profiling::finish_frame!();
            }
            winit::event::Event::LoopDestroyed => {
                sandbox.as_mut().unwrap().on_exit();
            }
            _ => {}
        }
    });
}

fn init_logger() {
    std::env::set_var("RUST_BACKTRACE", "full");
    std::env::set_var("RUST_LOG", "debug");

    let mut builder = env_logger::Builder::from_default_env();
    builder.target(env_logger::Target::Stdout);
    builder.init();
}
