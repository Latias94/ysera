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

impl super::Swapchain {
    pub unsafe fn release_resources(self, device: &ash::Device) -> Self {
        profiling::scope!("Swapchain::release_resources");
        {
            let _ = device.device_wait_idle();
        };
        device.destroy_fence(self.fence, None);
        self
    }
}
