use crate::DeviceError;
use ash::vk;

pub struct Texture {
    raw: vk::Image,
}

impl Texture {
    pub fn raw(&self) -> vk::Image {
        self.raw
    }

    pub fn get_supported_format(
        instance: &ash::Instance,
        adapter: vk::PhysicalDevice,
        formats: &[vk::Format],
        tiling: vk::ImageTiling,
        features: vk::FormatFeatureFlags,
    ) -> Result<vk::Format, DeviceError> {
        formats
            .iter()
            .cloned()
            .find(|f| {
                let properties =
                    unsafe { instance.get_physical_device_format_properties(adapter, *f) };
                match tiling {
                    vk::ImageTiling::LINEAR => properties.linear_tiling_features.contains(features),
                    vk::ImageTiling::OPTIMAL => {
                        properties.optimal_tiling_features.contains(features)
                    }
                    _ => false,
                }
            })
            .ok_or(DeviceError::Other("Failed to find supported format!"))
    }
}
