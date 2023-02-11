use crate::types::{AttachmentLoadOp, AttachmentStoreOp, Color, ImageFormat};
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

impl ImageFormat {
    pub fn to_vk(&self) -> vk::Format {
        match self {
            ImageFormat::Bgra8UnormSrgb => vk::Format::B8G8R8A8_UNORM,
            ImageFormat::Depth32Float => vk::Format::D32_SFLOAT,
            ImageFormat::Depth24Stencil8 => vk::Format::D24_UNORM_S8_UINT,
        }
    }
}

impl AttachmentStoreOp {
    pub fn to_vk(&self) -> vk::AttachmentStoreOp {
        match self {
            AttachmentStoreOp::Discard => vk::AttachmentStoreOp::DONT_CARE,
            AttachmentStoreOp::Store => vk::AttachmentStoreOp::STORE,
        }
    }
}

impl AttachmentLoadOp {
    pub fn to_vk(&self) -> vk::AttachmentLoadOp {
        match self {
            AttachmentLoadOp::Discard => vk::AttachmentLoadOp::DONT_CARE,
            AttachmentLoadOp::Clear => vk::AttachmentLoadOp::CLEAR,
            AttachmentLoadOp::Load => vk::AttachmentLoadOp::LOAD,
        }
    }
}
