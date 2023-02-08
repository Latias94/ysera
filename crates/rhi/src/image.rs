#[doc = "<https://www.khronos.org/registry/vulkan/specs/1.3-extensions/man/html/VkFormat.html>"]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum ImageFormat {
    Bgra8UnormSrgb,
    Depth32Float,
    Depth24Stencil8,
}

impl ImageFormat {
    pub fn is_depth_format(&self) -> bool {
        *self == ImageFormat::Depth24Stencil8 || *self == ImageFormat::Depth32Float
    }
}

pub enum ImageUsage {
    Texture,
    Attachment,
    Storage,
}

pub enum ImageWrap {
    Clamp,
    Repeat,
}

pub enum ImageFilter {
    Linear,
    Nearest,
}

pub enum ImageType {
    D2,
    Cube,
}

pub struct ImageProperties {
    pub sampler_wrap: ImageWrap,
    pub sampler_filter: ImageFilter,
    pub generate_mipmaps: bool,
    pub storage: bool,
}

impl Default for ImageProperties {
    fn default() -> Self {
        Self {
            sampler_wrap: ImageWrap::Repeat,
            sampler_filter: ImageFilter::Linear,
            generate_mipmaps: true,
            storage: false,
        }
    }
}

pub struct ImageSpecification {
    pub format: ImageFormat,
    pub usage: ImageUsage,
    pub width: u32,
    pub height: u32,
    pub mip_levels: u32,
    pub layers: u32,
}

impl Default for ImageSpecification {
    fn default() -> Self {
        Self {
            format: ImageFormat::Bgra8UnormSrgb,
            usage: ImageUsage::Texture,
            width: 0,
            height: 0,
            mip_levels: 1,
            layers: 1,
        }
    }
}

pub trait Image {
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
    fn get_specification(&self) -> ImageSpecification;
    fn release(self);

    fn max_mip_levels(&self) -> u32 {
        get_max_mip_levels(self.get_width(), self.get_height())
    }
}

pub fn get_max_mip_levels(width: u32, height: u32) -> u32 {
    (width.max(height) as f32).log2().floor() as u32 + 1
}
