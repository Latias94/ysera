use nalgebra_glm::{Vec2, Vec3};

pub struct Vertex3D {
    pub position: Vec3,
    pub color: Vec3,
    pub tex_coord: Vec2,
}

// Texture Coordinates
// In DirectX, Metal, and Vulkan, the top left corner of a texture is (0, 0).
// In OpenGL, the bottom left corner of a texture is (0, 0).
impl Vertex3D {
    pub fn new(position: Vec3, color: Vec3, tex_coord: Vec2) -> Self {
        Self {
            position,
            color,
            tex_coord,
        }
    }
}
