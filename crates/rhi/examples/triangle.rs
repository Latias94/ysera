use ash::vk;
use gpu_allocator::vulkan::{Allocation, Allocator};
use math::Mat4;
use parking_lot::Mutex;
use std::sync::Arc;

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

struct State {
    vertex_buffer: VertexBuffer,
    index_buffer: IndexBuffer,
    uniform_buffer: VertexShaderUniformBuffer,

    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_set: vk::DescriptorSet,
    present_complete_semaphore: vk::Semaphore,
    render_complete_semaphore: vk::Semaphore,
    fences: Vec<vk::Fence>,
}

fn main() {}
