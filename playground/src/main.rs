use std::borrow::{Borrow, Cow};
use std::time::Instant;

use illuminate::vulkan::renderer::VulkanRenderer;
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

fn main() {
    std::env::set_var("RUST_BACKTRACE", "full");
    std::env::set_var("RUST_LOG", "debug");

    profiling::tracy_client::Client::start();

    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();
    window.set_inner_size(LogicalSize::new(1080, 720));

    let mut builder = env_logger::Builder::from_default_env();
    builder.target(env_logger::Target::Stdout);
    builder.init();

    run(event_loop, window);
}

struct State {}
impl State {
    fn new(window: &Window) -> Self {
        VulkanRenderer::new(window);

        Self {}
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {}

    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {}

    fn render(&mut self) {}

    fn exit(mut self) {}
}

pub fn run(event_loop: EventLoop<()>, window: Window) {
    // State::new uses async code, so we're going to wait for it to finish
    let mut state = Some(State::new(&window));

    let mut last_frame_inst = Instant::now();
    let (mut frame_count, mut accum_time) = (0, 0.0);
    // workaround of vulkan window resize warning https://github.com/rust-windowing/winit/issues/2094
    let mut is_init = false;

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            let app = state.as_mut().unwrap();
            if !app.input(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        if is_init {
                            return;
                        }
                        let app = state.as_mut().unwrap();
                        app.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        let app = state.as_mut().unwrap();
                        app.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
        }
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            let app = state.as_mut().unwrap();
            {
                accum_time += last_frame_inst.elapsed().as_secs_f32();
                last_frame_inst = Instant::now();
                frame_count += 1;
                if accum_time >= 1f32 {
                    // second per frame
                    let avg_frame_time = accum_time / frame_count as f32;
                    let frame_rate = (1f32 / avg_frame_time).round() as i32;
                    let text = format!("Eureka Engine | FPS: {}", frame_rate);
                    window.set_title(text.as_str());
                    accum_time = 0.0;
                    frame_count = 0;
                }
            }
            app.update();
            app.render();

            profiling::finish_frame!();
            // match state.render() {
            //     Ok(_) => {}
            //     // 所有其他错误（过期、超时等）应在下一帧解决
            //     Err(e) => error!("{:?}", e),
            // }
        }
        Event::MainEventsCleared => {
            // 除非我们手动请求，RedrawRequested 将只会触发一次。
            window.request_redraw();
        }
        Event::LoopDestroyed => {
            state.take().unwrap().exit();
        }
        Event::NewEvents(cause) => {
            if cause == StartCause::Init {
                is_init = true;
            } else {
                is_init = false;
            }
        }
        _ => {}
    });
}
