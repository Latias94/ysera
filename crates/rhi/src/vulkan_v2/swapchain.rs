use crate::ImageFormat;
use ash::vk;

pub struct SwapChainSupportDetail {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

#[derive(Clone, Copy, Debug)]
pub struct SwapchainProperties {
    pub surface_format: vk::SurfaceFormatKHR,
    pub present_mode: vk::PresentModeKHR,
    pub extent: vk::Extent2D,
}

impl crate::Swapchain<super::Api> for super::Swapchain {
    fn get_width(&self) -> u32 {
        self.config.extent.width
    }

    fn get_height(&self) -> u32 {
        self.config.extent.height
    }

    fn get_image_index(&self) -> u32 {
        self.image_index
    }

    fn get_format(&self) -> ImageFormat {
        self.config.format
    }

    unsafe fn release_resources(self) -> Self {
        {
            let _ = self.device.raw.device_wait_idle();
        };
        self.device.raw.destroy_fence(self.fence, None);
        self
    }

    unsafe fn destroy(self) {
        let sc = self.release_resources();
        sc.loader.destroy_swapchain(sc.raw, None);
    }
}
