use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use ysera_rhi::vulkan_v2::adapter::{Adapter, Surface};
use ysera_rhi::vulkan_v2::device::Device;
use ysera_rhi::vulkan_v2::instance::Instance;
use ysera_rhi::vulkan_v2::swapchain::{Swapchain, SwapchainDescriptor};
use ysera_rhi::vulkan_v2::utils;
use ysera_rhi::{AdapterRequirements, InstanceDescriptor};

struct Application {
    adapter: Adapter,
    surface: Arc<Surface>,
    instance: Instance,
    device: Arc<Device>,
    swapchain: Swapchain,
}

const DESIRED_FRAMES: u32 = 3;

impl Application {
    fn init_log() {
        init_logger();
        log::info!("hello");
    }

    fn exit_profile() {}

    fn init(window: &winit::window::Window) -> anyhow::Result<Self> {
        Self::init_log();

        let instance_desc = InstanceDescriptor::builder()
            // .flags(crate::vulkan::instance::InstanceFlags::empty())
            // .debug_level_filter(log::LevelFilter::Info)
            .build();
        let instance = unsafe { Instance::init(&instance_desc)? };
        let surface = unsafe { instance.create_surface(window)? };
        let adapters = instance.enumerate_adapters()?;
        assert!(!adapters.is_empty());

        let requirements = AdapterRequirements::builder()
            .compute(true)
            .adapter_extension_names(vec![])
            .build();
        let mut selected_adapter = None;
        for adapter in adapters {
            if unsafe { adapter.meet_requirements(&instance.shared.raw, &surface, &requirements) }
                .is_ok()
            {
                selected_adapter = Some(adapter);
                break;
            }
        }
        let adapter = match selected_adapter {
            None => panic!("Cannot find the require device."),
            Some(adapter) => adapter,
        };

        log::debug!("Find the require device.");

        let indices =
            utils::get_queue_family_indices(&instance.shared.raw, adapter.shared.raw, &surface)?;
        indices.log_debug();

        let device = unsafe { Device::create(&instance, &adapter, indices, &requirements)? };
        let device = Arc::new(device);
        let surface = Arc::new(surface);

        let inner_size = window.inner_size();
        let swapchain = unsafe {
            Swapchain::create(
                &device,
                &surface,
                SwapchainDescriptor {
                    width: inner_size.width,
                    height: inner_size.height,
                    vsync: false,
                },
            )
        }?;

        Ok(Self {
            adapter,
            surface,
            instance,
            device,
            swapchain,
        })
    }

    fn exit(mut self) {
        unsafe {
            self.device.raw().device_wait_idle().unwrap();
            // self.swapchain.destroy();
            self.device.destroy();
        }
        Self::exit_profile();
    }

    fn update(&mut self, event: winit::event::WindowEvent) {}

    fn render(&mut self) {}
}

fn load_pre_compiled_spv_bytes_from_name(shader_file_name: &str) -> Vec<u32> {
    let path = format!("{}/{}.spv", env!("OUT_DIR"), shader_file_name);
    log::debug!("load shader spv file from: {}", path);
    load_pre_compiled_spv_bytes_from_path(Path::new(&path))
}

fn load_pre_compiled_spv_bytes_from_path<P: AsRef<Path>>(path: P) -> Vec<u32> {
    use std::fs::File;
    use std::io::Read;
    let spv_file = File::open(path).unwrap();
    let bytes_code: Vec<u8> = spv_file.bytes().filter_map(|byte| byte.ok()).collect();
    let (_prefix, bytes, _suffix) = unsafe { bytes_code.align_to::<u32>() };
    bytes.into()
}

fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("Ysera example")
        .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
        .build(&event_loop)
        .unwrap();
    let mut last_frame_inst = Instant::now();
    let (mut frame_count, mut accum_time) = (0, 0.0);

    let result = Application::init(&window);
    let mut application = Some(result.expect("Select backend is not supported"));

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
                    application.as_mut().unwrap().update(event);
                }
            },
            winit::event::Event::RedrawRequested(_window_id) => {
                let app = application.as_mut().unwrap();
                {
                    accum_time += last_frame_inst.elapsed().as_secs_f32();
                    last_frame_inst = Instant::now();
                    frame_count += 1;
                    if frame_count == 5000 {
                        log::debug!(
                            "Avg frame time {}ms",
                            accum_time * 1000.0 / frame_count as f32
                        );
                        accum_time = 0.0;
                        frame_count = 0;
                    }
                }
                app.render();
                profiling::finish_frame!();
            }
            winit::event::Event::LoopDestroyed => {
                application.take().unwrap().exit();
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
