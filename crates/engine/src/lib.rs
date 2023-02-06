// use std::cell::RefCell;
// use std::rc::Rc;
// use std::time::Instant;
//
// use winit::dpi::{LogicalSize, PhysicalSize};
// use winit::event::{ElementState, Event, KeyboardInput, StartCause, VirtualKeyCode, WindowEvent};
// use winit::event_loop::{ControlFlow, EventLoop};
// use winit::window::{Window, WindowId};
//
// use rhi::vulkan::renderer::VulkanRenderer;
// pub use rhi::winit;
// use ysera_imgui::controls::InputState;
// use ysera_imgui::gui::{GuiContext, GuiContextDescriptor};
// use ysera_imgui::GuiTheme;
//
// pub mod engine;
// pub mod event;
// pub use engine::Engine;
//
// #[derive(Copy, Clone)]
// pub struct AppConfig<'a> {
//     pub name: &'a str,
//     pub window_size: [u32; 2],
// }
//
// pub trait EurekaEngine {
//     fn new() -> Self
//     where
//         Self: Sized;
//
//     fn on_init(&mut self);
//     fn on_update(&mut self, delta_time: f32, engine: &mut Engine);
//     fn on_render(&mut self, delta_time: f32);
//     fn on_shutdown(&mut self);
//
//     fn on_window_resize(&mut self, width: u32, height: u32);
//     fn on_window_input(&mut self, keycode: VirtualKeyCode);
// }
//
// pub fn create<T: 'static + EurekaEngine + Send>(config: AppConfig) {
//     let game = T::new();
//
//     let event_loop = EventLoop::new();
//     let window = Window::new(&event_loop).unwrap();
//     window.set_inner_size(LogicalSize::new(
//         config.window_size[0],
//         config.window_size[1],
//     ));
//
//     window.set_title(config.name);
//
//     run(event_loop, window, config, game);
// }
//
// struct State<T: 'static + EurekaEngine + Send> {
//     name: String,
//     window_size: [u32; 2],
//     engine: Engine,
//     gui_context: GuiContext,
//     window_id: WindowId,
//     game: T,
// }
//
// impl<T: 'static + EurekaEngine + Send> State<T> {
//     fn new(window: &Window, config: AppConfig, mut game: T) -> Self {
//         let editor_context_desc = GuiContextDescriptor {
//             window,
//             hidpi_factor: window.scale_factor(),
//             theme: GuiTheme::Dark,
//         };
//
//         let mut gui_context = GuiContext::new(&editor_context_desc);
//         let renderer = VulkanRenderer::new(window, gui_context.get_context()).unwrap();
//         let engine = Engine {
//             renderer: Rc::new(RefCell::new(renderer)),
//         };
//         game.on_init();
//
//         Self {
//             name: config.name.to_string(),
//             window_size: config.window_size,
//             engine,
//             gui_context,
//             window_id: window.id(),
//             game,
//         }
//     }
//
//     fn resize(&mut self, new_size: PhysicalSize<u32>) {
//         if new_size.width > 0 && new_size.height > 0 {
//             self.window_size = [new_size.width, new_size.height];
//             self.engine
//                 .renderer
//                 .borrow_mut()
//                 .recreate_swapchain(new_size)
//                 .unwrap();
//             self.game.on_window_resize(new_size.width, new_size.height);
//         }
//     }
//
//     fn input(&mut self, event: &WindowEvent) -> bool {
//         match *event {
//             WindowEvent::KeyboardInput {
//                 input:
//                     KeyboardInput {
//                         state: ElementState::Pressed,
//                         virtual_keycode: Some(key),
//                         ..
//                     },
//                 ..
//             } => {
//                 self.game.on_window_input(key);
//                 // log::info!("press {:?}", key);
//             }
//             _ => {}
//         }
//
//         false
//     }
//
//     fn update(&mut self, delta_time: f32) {
//         self.game.on_update(delta_time, &mut self.engine);
//     }
//
//     fn render(&mut self, window: &Window, delta_time: f32) {
//         self.engine
//             .renderer
//             .borrow_mut()
//             .render(window, &mut self.gui_context)
//             .unwrap();
//         self.game.on_render(delta_time);
//     }
//
//     fn exit(mut self) {
//         self.game.on_shutdown();
//     }
// }
//
// fn run<T: 'static + EurekaEngine + Send>(
//     event_loop: EventLoop<()>,
//     window: Window,
//     config: AppConfig,
//     game: T,
// ) {
//     // State::new uses async code, so we're going to wait for it to finish
//     let mut state = Some(State::new(&window, config, game));
//
//     let mut last_frame_inst = Instant::now();
//     let (mut frame_count, mut accum_time) = (0, 0.0);
//     // workaround of vulkan window resize warning https://github.com/rust-windowing/winit/issues/2094
//     let mut is_init = false;
//     let mut minimized = false;
//     let mut input_state = InputState::default();
//
//     event_loop.run(move |event, _, control_flow| {
//         let app = state.as_mut().unwrap();
//         app.gui_context.handle_event(&window, &event);
//         app.engine
//             .renderer
//             .borrow_mut()
//             .handle_event(&window, &event);
//         input_state = input_state.update(&event);
//
//         match event {
//             Event::WindowEvent {
//                 ref event,
//                 window_id,
//             } if window_id == window.id() => {
//                 if !app.input(event) {
//                     match event {
//                         WindowEvent::CloseRequested
//                         | WindowEvent::KeyboardInput {
//                             input:
//                                 KeyboardInput {
//                                     state: ElementState::Pressed,
//                                     virtual_keycode: Some(VirtualKeyCode::Escape),
//                                     ..
//                                 },
//                             ..
//                         } => *control_flow = ControlFlow::Exit,
//                         WindowEvent::Resized(size) => {
//                             if is_init {
//                                 return;
//                             }
//                             if size.width == 0 || size.height == 0 {
//                                 minimized = true;
//                             } else {
//                                 minimized = false;
//                             }
//                             let app = state.as_mut().unwrap();
//                             app.resize(*size);
//                         }
//                         WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
//                             let app = state.as_mut().unwrap();
//                             app.resize(**new_inner_size);
//                         }
//                         _ => {}
//                     }
//                 }
//             }
//             Event::RedrawRequested(window_id) if window_id == window.id() => {
//                 let now = Instant::now();
//                 let delta_time =
//                     last_frame_inst.elapsed().as_secs_f32() - now.elapsed().as_secs_f32();
//                 let app = state.as_mut().unwrap();
//                 {
//                     accum_time += last_frame_inst.elapsed().as_secs_f32();
//                     last_frame_inst = now;
//                     frame_count += 1;
//                     if accum_time >= 1f32 {
//                         // second per frame
//                         let avg_frame_time = accum_time / frame_count as f32;
//                         let frame_rate = (1f32 / avg_frame_time).round() as i32;
//                         let text = format!("{} | FPS: {}", app.name, frame_rate);
//                         window.set_title(text.as_str());
//                         accum_time = 0.0;
//                         frame_count = 0;
//                     }
//                 }
//
//                 app.gui_context.prepare_frame(&window);
//
//                 app.update(delta_time);
//                 if !minimized {
//                     app.render(&window, delta_time);
//                 }
//
//                 profiling::finish_frame!();
//                 // match state.render() {
//                 //     Ok(_) => {}
//                 //     // 所有其他错误（过期、超时等）应在下一帧解决
//                 //     Err(e) => error!("{:?}", e),
//                 // }
//             }
//             Event::MainEventsCleared => {
//                 // 除非我们手动请求，RedrawRequested 将只会触发一次。
//                 window.request_redraw();
//             }
//             Event::LoopDestroyed => {
//                 state.take().unwrap().exit();
//             }
//             Event::NewEvents(cause) => {
//                 if cause == StartCause::Init {
//                     is_init = true;
//                 } else {
//                     is_init = false;
//                 }
//                 app.gui_context.update_delta_time();
//             }
//             _ => {}
//         }
//     });
// }
