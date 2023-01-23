use nalgebra_glm::{Vec2, Vec3};
use std::hash::{Hash, Hasher};

#[derive(Clone, Copy)]
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

impl Eq for Vertex3D {}

impl PartialEq for Vertex3D {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
            && self.color == other.color
            && self.tex_coord == other.tex_coord
    }
}

impl Hash for Vertex3D {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.position[0].to_bits().hash(state);
        self.position[1].to_bits().hash(state);
        self.position[2].to_bits().hash(state);
        self.color[0].to_bits().hash(state);
        self.color[1].to_bits().hash(state);
        self.color[2].to_bits().hash(state);
        self.tex_coord[0].to_bits().hash(state);
        self.tex_coord[1].to_bits().hash(state);
    }
}
