use ash::vk;

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
