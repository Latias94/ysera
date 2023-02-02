use eureka_rhi::{
    Device, Extent3d, GraphicsApi, ImageFormat, ImageUses, Instance, InstanceDescriptor,
    OpenDevice, PhysicalDevice, PresentMode, ShaderInput, Surface, SurfaceConfiguration, Swapchain,
};
use eureka_rhi::{Features, InstanceError};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::path::Path;
use std::time::Instant;

struct Application<Api: GraphicsApi> {
    instance: Api::Instance,
    surface: Api::Surface,
    physical_device: Api::PhysicalDevice,
    device: Api::Device,
    swapchain: Api::Swapchain,
    queue: Api::Queue,
    surface_format: ImageFormat,
    // command_pool: Api::CommandPool,
    // local_buffer: Api::Buffer,
}

const DESIRED_FRAMES: u32 = 3;

impl<Api: GraphicsApi> Application<Api> {
    fn init_log() {
        init_logger();
        log::info!("hello");
    }

    fn exit_profile() {}

    fn init(window: &winit::window::Window) -> Result<Self, InstanceError> {
        Self::init_log();

        let instance_desc = InstanceDescriptor::default();
        let instance = unsafe { Api::Instance::init(&instance_desc).expect("error") };
        let mut surface = unsafe {
            instance
                .create_surface(window.raw_display_handle(), window.raw_window_handle())
                .unwrap()
        };

        let mut physical_devices = unsafe { instance.enumerate_physical_devices(&surface) };
        if physical_devices.is_empty() {
            return Err(InstanceError::NotSupport());
        }

        let physical_device: <Api as GraphicsApi>::PhysicalDevice = {
            let exposed = physical_devices.swap_remove(0);
            (exposed.physical_device)
        };

        let surface_caps = unsafe { physical_device.surface_capabilities(&surface) }
            .ok_or(InstanceError::NotSupport())?;
        log::info!("Surface caps: {:#?}", surface_caps);
        let OpenDevice { device, mut queue } =
            unsafe { physical_device.open(Features::empty()).unwrap() };

        let window_size: (u32, u32) = window.inner_size().into();
        let surface_config = SurfaceConfiguration {
            swap_chain_size: DESIRED_FRAMES
                .max(*surface_caps.swap_chain_sizes.start())
                .min(*surface_caps.swap_chain_sizes.end()),
            present_mode: PresentMode::Fifo,
            format: ImageFormat::Bgra8UnormSrgb,
            extent: Extent3d {
                width: window_size.0,
                height: window_size.1,
                depth_or_array_layers: 1,
            },
            usage: ImageUses::COLOR_TARGET,
        };

        let swapchain = unsafe { surface.configure(&device, &surface_config, None).unwrap() };
        log::info!(
            "Current window size: ({}, {})",
            swapchain.get_width(),
            swapchain.get_height()
        );

        let vert_shader =
            ShaderInput::SpirV(&load_pre_compiled_spv_bytes_from_name("triangle.vert"));
        let frag_shader =
            ShaderInput::SpirV(&load_pre_compiled_spv_bytes_from_name("triangle.frag"));

        // let shader_desc = rhi::ShaderModuleDescriptor {
        //     label: None,
        //     runtime_checks: true,
        // };
        // let vert_shader_module = unsafe {
        //     device
        //         .create_shader_module(&shader_desc, rhi::ShaderInput::Naga(vert_shader))
        //         .unwrap()
        // };
        // let frag_shader_module = unsafe {
        //     device
        //         .create_shader_module(&shader_desc, rhi::ShaderInput::Naga(frag_shader))
        //         .unwrap()
        // };

        // let staging_buffer_desc = rhi::BufferDescriptor {
        //     label: Some("stage"),
        //     size: texture_data.len() as BufferAddress,
        //     usage: rhi::BufferUses::MAP_WRITE | rhi::BufferUses::COPY_SRC,
        //     memory_flags: rhi::MemoryPropertyFlags::HOST_COHERENT
        //         | rhi::MemoryPropertyFlags::HOST_VISIBLE,
        // };
        // let staging_buffer = unsafe { device.create_buffer(&staging_buffer_desc).unwrap() };
        Ok(Self {
            instance,
            surface,
            physical_device,
            device,
            swapchain,
            queue,
            surface_format: surface_config.format,
        })
    }

    fn exit(mut self) {
        unsafe {
            self.swapchain.destroy();
            self.device.shutdown(self.queue);
            self.instance.destroy_surface(self.surface);
            drop(self.physical_device)
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

type Api = eureka_rhi::api::Vulkan;

fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("Eureka example")
        .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
        .build(&event_loop)
        .unwrap();
    let mut last_frame_inst = Instant::now();
    let (mut frame_count, mut accum_time) = (0, 0.0);

    let result = Application::<Api>::init(&window);
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
