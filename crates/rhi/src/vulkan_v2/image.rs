use crate::Extent3d;
use ash::vk;
use fxhash::FxHashMap;
use std::sync::Mutex;

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
pub struct ImageViewDescriptor {
    pub view_type: Option<vk::ImageViewType>,
    pub format: Option<vk::Format>,
    // vk::ImageAspectFlags::COLOR
    pub aspect_mask: vk::ImageAspectFlags,
    pub base_mip_level: u32,
    pub mip_level_count: Option<u32>,
}

pub struct Image {
    pub raw: vk::Image,
    pub desc: ImageDescriptor,
    pub views: Mutex<FxHashMap<ImageViewDescriptor, vk::ImageView>>,
}

impl Image {}
