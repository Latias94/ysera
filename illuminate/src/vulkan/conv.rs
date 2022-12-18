use crate::Color;
use ash::vk;
use ash::vk::ClearDepthStencilValue;

pub fn convert_rect2d(rect: math::Rect2D) -> vk::Rect2D {
    vk::Rect2D::builder()
        .extent(vk::Extent2D {
            width: rect.width as u32,
            height: rect.height as u32,
        })
        .offset(vk::Offset2D {
            x: rect.x as i32,
            y: rect.y as i32,
        })
        .build()
}

pub fn convert_clear_color(color: Color) -> vk::ClearValue {
    vk::ClearValue {
        color: vk::ClearColorValue {
            float32: [color.r, color.g, color.b, color.a],
        },
    }
}

pub fn convert_clear_depth_stencil(depth: f32, stencil: u32) -> vk::ClearValue {
    vk::ClearValue {
        depth_stencil: ClearDepthStencilValue { depth, stencil },
    }
}
