use math::Mat4;

/// 统一缓冲区对象（UBO）
#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct UniformBufferObject {
    pub view: Mat4,
    pub projection: Mat4,
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
