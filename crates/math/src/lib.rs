pub use nalgebra_glm::*;

pub use rect::*;
pub use vertex::*;

mod rect;
mod vertex;

pub const PI: f32 = std::f32::consts::PI;
pub const PI_2: f32 = 2.0f32 * PI;
pub const HALF_PI: f32 = 0.5f32 * PI;
pub const QUARTER_PI: f32 = 0.25f32 * PI;
pub const ONE_OVER_PI: f32 = 1.0f32 / PI;
pub const ONE_OVER_2_PI: f32 = 1.0f32 / PI_2;
pub const SQRT_2: f32 = std::f32::consts::SQRT_2;
pub const SQRT_3: f32 = 1.73205080756887729352_f32;
pub const FRAC_1_SQRT_2: f32 = std::f32::consts::FRAC_1_SQRT_2;
pub const FRAC_1_SQRT_3: f32 = 0.57735026918962576450_f32;
pub const DEG2RAD_MULTIPLIER: f32 = PI / 180.0f32;
pub const RAD2DEG_MULTIPLIER: f32 = 180.0f32 / PI;

pub const SEC_TO_MS_MULTIPLIER: f32 = 1000.0_f32;
pub const MS_TO_SEC_MULTIPLIER: f32 = 0.001_f32;
pub const INFINITY: f32 = f32::INFINITY;
pub const FLOAT_EPSILON: f32 = f32::EPSILON;

pub fn is_power_of_2(value: u64) -> bool {
    (value != 0) && ((value & (value - 1)) == 0)
}

pub mod prelude {
    #[doc(hidden)]
    pub use crate::{
        mat2, mat2x2, mat2x3, mat2x4, mat3, mat3x2, mat3x3, mat3x4, mat4, mat4x2, mat4x3, mat4x4,
        quat, vec2, vec3, vec4, BVec2, BVec3, BVec4, IVec2, IVec3, IVec4, Mat2, Mat3, Mat4, Quat,
        Rect2D, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4, Vertex3D,
    };
}
