use std::collections::hash_map::Entry;

use ash::vk;
use fxhash::FxHashMap;
use parking_lot::Mutex;

use crate::vulkan_v2::device::Device;
use crate::{DeviceError, Extent3d, Label};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum ImageType {
    Tex1d = 0,
    Tex1dArray = 1,
    Tex2d = 2,
    Tex2dArray = 3,
    Tex3d = 4,
    Cube = 5,
    CubeArray = 6,
}

impl ImageType {
    pub fn to_vk_image_view_type(self) -> vk::ImageViewType {
        match self {
            ImageType::Tex1d => vk::ImageViewType::TYPE_1D,
            ImageType::Tex1dArray => vk::ImageViewType::TYPE_1D_ARRAY,
            ImageType::Tex2d => vk::ImageViewType::TYPE_2D,
            ImageType::Tex2dArray => vk::ImageViewType::TYPE_2D_ARRAY,
            ImageType::Tex3d => vk::ImageViewType::TYPE_3D,
            ImageType::Cube => vk::ImageViewType::CUBE,
            ImageType::CubeArray => vk::ImageViewType::CUBE_ARRAY,
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct ImageDescriptor {
    pub image_type: ImageType,
    pub usage: vk::ImageUsageFlags,
    pub flags: vk::ImageCreateFlags,
    pub format: vk::Format,
    pub extent: Extent3d,
    pub tiling: vk::ImageTiling,
    pub mip_levels: u16,
    pub array_elements: u32,
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Hash)]
pub struct ImageViewDescriptorKey {
    /// if None use ImageDescriptor.image_type
    pub view_type: Option<vk::ImageViewType>,
    /// if None use ImageDescriptor.format
    pub format: Option<vk::Format>,
    /// vk::ImageAspectFlags::COLOR
    pub aspect_mask: vk::ImageAspectFlags,
    pub base_mip_level: u32,
    /// if None use ImageDescriptor.mip_levels
    pub mip_levels: Option<u32>,
}

pub struct Image {
    pub raw: vk::Image,
    pub desc: ImageDescriptor,
    pub views: Mutex<FxHashMap<ImageViewDescriptorKey, vk::ImageView>>,
}

impl Image {
    pub unsafe fn get_view(
        &self,
        device: &Device,
        desc: ImageViewDescriptorKey,
        label: Label,
    ) -> Result<vk::ImageView, DeviceError> {
        match self.views.lock().entry(desc) {
            Entry::Occupied(e) => Ok(*e.get()),
            Entry::Vacant(e) => {
                let view = device.create_image_view(desc, &self.desc, self.raw, label)?;
                e.insert(view);
                Ok(view)
            }
        }
    }

    unsafe fn view_desc_impl(
        image_view_desc: ImageViewDescriptorKey,
        image_desc: &ImageDescriptor,
    ) -> vk::ImageViewCreateInfo {
        let range = vk::ImageSubresourceRange::builder()
            .aspect_mask(image_view_desc.aspect_mask)
            .base_array_layer(0)
            .layer_count(match image_desc.image_type {
                ImageType::Cube | ImageType::CubeArray => 6,
                _ => 1,
            })
            .base_mip_level(image_view_desc.base_mip_level)
            .level_count(
                image_view_desc
                    .mip_levels
                    .unwrap_or(image_desc.mip_levels as u32),
            )
            .build();
        vk::ImageViewCreateInfo::builder()
            .flags(vk::ImageViewCreateFlags::empty())
            // 用于指定图像被看作是一维纹理、二维纹理、三维纹理还是立方体贴图
            .view_type(
                image_view_desc
                    .view_type
                    .unwrap_or_else(|| image_desc.image_type.to_vk_image_view_type()),
            )
            .format(image_view_desc.format.unwrap_or(image_desc.format))
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
            .build()
    }
}

impl Device {
    unsafe fn create_image_view(
        &self,
        image_view_desc: ImageViewDescriptorKey,
        image_desc: &ImageDescriptor,
        image_raw: vk::Image,
        label: Label,
    ) -> Result<vk::ImageView, DeviceError> {
        if image_desc.format == vk::Format::D32_SFLOAT
            && !image_view_desc
                .aspect_mask
                .contains(vk::ImageAspectFlags::DEPTH)
        {
            return Err(DeviceError::Other(
                "image_view_desc.aspect_mask should contain vk::ImageAspectFlags::DEPTH ",
            ));
        }

        let create_info = vk::ImageViewCreateInfo {
            image: image_raw,
            ..Image::view_desc_impl(image_view_desc, image_desc)
        };

        let raw = unsafe { self.raw.create_image_view(&create_info, None)? };

        if let Some(label) = label {
            unsafe { self.set_object_name(vk::ObjectType::IMAGE_VIEW, raw, label) };
        }

        Ok(raw)
    }
}
