use nalgebra_glm::Vec3;

pub struct Vertex3D {
    pub position: Vec3,
    pub color: Vec3,
}

impl Vertex3D {
    fn new(position: Vec3, color: Vec3) -> Self {
        Self { position, color }
    }
}
