use std::sync::Arc;
use std::time::Duration;

use ash::vk;
use gpu_allocator::vulkan::{Allocation, Allocator};
use parking_lot::Mutex;

use math::Mat4;
use ysera_rhi::vulkan::base_renderer::{BaseRenderer, RendererBase};
use ysera_rhi::vulkan::buffer::{Buffer, BufferType, StagingBufferDescriptor};
use ysera_rhi::vulkan::context::Context;
use ysera_rhi::vulkan::pipeline::Pipeline;
use ysera_rhi::vulkan::pipeline_layout::PipelineLayout;
use ysera_rhi::DeviceError;

// math::Vertex3D
struct VertexBuffer {
    buffer: vk::Buffer,
    allocator: Arc<Mutex<Allocator>>,
    allocation: Option<Allocation>,
}

struct IndexBuffer {
    buffer: vk::Buffer,
    allocator: Arc<Mutex<Allocator>>,
    allocation: Option<Allocation>,
    count: u32,
}

struct VertexShaderUniformBuffer {
    buffer: vk::Buffer,
    allocator: Arc<Mutex<Allocator>>,
    allocation: Option<Allocation>,
    descriptor: vk::DescriptorBufferInfo,
}

/// 统一缓冲区对象（UBO）
#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
struct UniformBufferObject {
    pub proj: Mat4,
    pub model: Mat4,
    pub view: Mat4,
}

// alignment requirements: https://www.khronos.org/registry/vulkan/specs/1.1-extensions/html/chap14.html#interfaces-resources-layout
// #[repr(C)]
// #[derive(Copy, Clone, Debug)]
// struct UniformBufferObject {
//     foo: glm::Vec2,
//     _padding: [u8; 8],
//     model: glm::Mat4,
//     view: glm::Mat4,
//     proj: glm::Mat4,
// }

struct Triangle {
    vertex_buffer: Buffer,
    pipeline_layout: PipelineLayout,
    pipline: Pipeline,
}

impl RendererBase for Triangle {
    type Gui = ();

    fn new(base: &mut BaseRenderer<Self>) -> anyhow::Result<Self> {
        let context = &mut base.context;
        let vertex_buffer = create_vertex_buffer(context)?;

        todo!()
    }

    fn update(
        &mut self,
        base: &BaseRenderer<Self>,
        image_index: usize,
        delta_time: Duration,
    ) -> anyhow::Result<()> {
        todo!()
    }

    fn on_recreate_swapchain(&mut self) -> anyhow::Result<()> {
        todo!()
    }
}

fn main() {
    // let event_loop = winit::event_loop::EventLoop::new();
    // let window = winit::window::WindowBuilder::new()
    //     .with_title("Ysera example")
    //     .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
    //     .build(&event_loop)
    //     .unwrap();
    // let mut last_frame_inst = Instant::now();
    // let (mut frame_count, mut accum_time) = (0, 0.0);
    //
    // event_loop.run(move |event, _, control_flow| {
    //     let _ = &window; // force ownership by the closure
    //     *control_flow = winit::event_loop::ControlFlow::Poll;
    //     match event {
    //         winit::event::Event::MainEventsCleared => {
    //             window.request_redraw();
    //         }
    //         winit::event::Event::WindowEvent { event, .. } => match event {
    //             winit::event::WindowEvent::KeyboardInput {
    //                 input:
    //                     winit::event::KeyboardInput {
    //                         virtual_keycode: Some(winit::event::VirtualKeyCode::Escape),
    //                         state: winit::event::ElementState::Pressed,
    //                         ..
    //                     },
    //                 ..
    //             }
    //             | winit::event::WindowEvent::CloseRequested => {
    //                 *control_flow = winit::event_loop::ControlFlow::Exit;
    //             }
    //             _ => {
    //                 application.as_mut().unwrap().update(event);
    //             }
    //         },
    //         winit::event::Event::RedrawRequested(_window_id) => {
    //             let app = application.as_mut().unwrap();
    //             {
    //                 accum_time += last_frame_inst.elapsed().as_secs_f32();
    //                 last_frame_inst = Instant::now();
    //                 frame_count += 1;
    //                 if frame_count == 5000 {
    //                     log::debug!(
    //                         "Avg frame time {}ms",
    //                         accum_time * 1000.0 / frame_count as f32
    //                     );
    //                     accum_time = 0.0;
    //                     frame_count = 0;
    //                 }
    //             }
    //             app.render();
    //             profiling::finish_frame!();
    //         }
    //         winit::event::Event::LoopDestroyed => {
    //             application.take().unwrap().exit();
    //         }
    //         _ => {}
    //     }
    // });
}

fn init_logger() {
    std::env::set_var("RUST_BACKTRACE", "full");
    std::env::set_var("RUST_LOG", "debug");

    let mut builder = env_logger::Builder::from_default_env();
    builder.target(env_logger::Target::Stdout);
    builder.init();
}

struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

impl Vertex {
    fn bindings() -> Vec<vk::VertexInputBindingDescription> {
        vec![vk::VertexInputBindingDescription {
            binding: 0,
            stride: 20,
            input_rate: vk::VertexInputRate::VERTEX,
        }]
    }

    fn attributes() -> Vec<vk::VertexInputAttributeDescription> {
        vec![
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: vk::Format::R32G32_SFLOAT,
                offset: 0,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: 8,
            },
        ]
    }
}

fn create_vertex_buffer(context: &Context) -> Result<Buffer, DeviceError> {
    let vertices: [Vertex; 3] = [
        Vertex {
            position: [-1.0, 1.0],
            color: [1.0, 0.0, 0.0],
        },
        Vertex {
            position: [1.0, 1.0],
            color: [0.0, 1.0, 0.0],
        },
        Vertex {
            position: [0.0, -1.0],
            color: [0.0, 0.0, 1.0],
        },
    ];
    let desc = StagingBufferDescriptor {
        label: Some("Staging Buffer"),
        device: &context.device,
        allocator: context.allocator.clone(),
        elements: &vertices,
    };

    let vertex_buffer =
        unsafe { Buffer::new_buffer_copy_from_staging_buffer(&desc, BufferType::Vertex)? };

    Ok(vertex_buffer)
}
