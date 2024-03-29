use crate::vulkan::device::Device;
use crate::Label;
use ash::vk;
use std::rc::Rc;
use typed_builder::TypedBuilder;

#[derive(Clone, Debug, TypedBuilder)]
pub struct ImageViewDescriptor<'a> {
    pub label: Label<'a>,
    pub format: vk::Format,
    pub dimension: vk::ImageViewType,
    pub aspect_mask: vk::ImageAspectFlags,
    pub mip_levels: u32,
    // pub usage: vk::ImageUsageFlags,
    // pub range: vk::ImageSubresourceRange,
}

pub struct ImageView {
    raw: vk::ImageView,
    device: Rc<Device>,
}

impl ImageView {
    pub fn raw(&self) -> vk::ImageView {
        self.raw
    }

    pub fn new_color_image_view(
        label: Label,
        device: &Rc<Device>,
        image: vk::Image,
        format: vk::Format,
        mip_levels: u32,
    ) -> Result<ImageView, crate::DeviceError> {
        let desc = ImageViewDescriptor {
            label,
            format,
            dimension: vk::ImageViewType::TYPE_2D,
            aspect_mask: vk::ImageAspectFlags::COLOR,
            mip_levels,
        };
        Self::new(device, image, &desc)
    }

    pub fn new_depth_image_view(
        label: Label,
        device: &Rc<Device>,
        image: vk::Image,
        format: vk::Format,
    ) -> Result<ImageView, crate::DeviceError> {
        let desc = ImageViewDescriptor {
            label,
            format,
            dimension: vk::ImageViewType::TYPE_2D,
            aspect_mask: vk::ImageAspectFlags::DEPTH,
            mip_levels: 1,
        };
        Self::new(device, image, &desc)
    }

    fn new(
        device: &Rc<Device>,
        image: vk::Image,
        desc: &ImageViewDescriptor,
    ) -> Result<ImageView, crate::DeviceError> {
        let range = vk::ImageSubresourceRange::builder()
            .aspect_mask(desc.aspect_mask)
            .base_array_layer(0)
            .layer_count(1)
            .base_mip_level(0)
            .level_count(desc.mip_levels)
            .build();
        let info = vk::ImageViewCreateInfo::builder()
            .flags(vk::ImageViewCreateFlags::empty())
            .image(image)
            // 用于指定图像被看作是一维纹理、二维纹理、三维纹理还是立方体贴图
            .view_type(desc.dimension)
            .format(desc.format)
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
        let raw = device.create_image_view(&info)?;
        if let Some(label) = desc.label {
            unsafe { device.set_object_name(vk::ObjectType::IMAGE_VIEW, raw, label) };
        }
        Ok(ImageView {
            raw,
            device: device.clone(),
        })
    }
}

impl Drop for ImageView {
    fn drop(&mut self) {
        self.device.destroy_image_view(self.raw);
        log::debug!("ImageView destroyed.");
    }
}
