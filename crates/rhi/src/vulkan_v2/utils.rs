use crate::RHIError;
use ash::vk;

pub unsafe fn create_image_views(
    device: &ash::Device,
    image: vk::Image,
    format: vk::Format,
    aspect_mask: vk::ImageAspectFlags,
    view_type: vk::ImageViewType,
    layer_count: u32,
    mip_levels: u32,
) -> Result<vk::ImageView, RHIError> {
    let range = vk::ImageSubresourceRange::builder()
        .aspect_mask(aspect_mask)
        .base_array_layer(0)
        .layer_count(layer_count)
        .base_mip_level(0)
        .level_count(mip_levels)
        .build();
    let info = vk::ImageViewCreateInfo::builder()
        .flags(vk::ImageViewCreateFlags::empty())
        .image(image)
        // 用于指定图像被看作是一维纹理、二维纹理、三维纹理还是立方体贴图
        .view_type(view_type)
        .format(format)
        // 指定图像的用途和图像的哪一部分可以被访问。
        // 如果编写 VR 一类的应用程序，可能会使用支持多个层次的交换链。这时，应该为每个图像创建多个图像视图，
        // 分别用来访问左眼和右眼两个不同的图层。
        .subresource_range(range)
        // 用于进行图像颜色通道的映射。比如，对于单色纹理，我们可以将所有颜色通道映射到红色通道。
        // 我们也可以直接将颜色通道的值映射为常数 0 或 1。在这里，我们只使用默认的映射。
        .components(vk::ComponentMapping {
            r: vk::ComponentSwizzle::IDENTITY,
            g: vk::ComponentSwizzle::IDENTITY,
            b: vk::ComponentSwizzle::IDENTITY,
            a: vk::ComponentSwizzle::IDENTITY,
        })
        .build();
    let raw = unsafe { device.create_image_view(&info, None)? };

    Ok(raw)
}
