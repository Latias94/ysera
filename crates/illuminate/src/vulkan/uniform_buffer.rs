use math::Mat4;

/// 统一缓冲区对象（UBO）
#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct UniformBufferObject {
    pub model: Mat4,
    pub view: Mat4,
    pub projection: Mat4,
}
