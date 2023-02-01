use crate::{SurfaceConfiguration, SurfaceError};

impl crate::Surface<super::Api> for super::Surface {
    unsafe fn configure(
        &mut self,
        device: &super::Device,
        config: &SurfaceConfiguration,
    ) -> Result<(), SurfaceError> {
        let old = self
            .swapchain
            .take()
            .map(|sc| sc.release_resources(&device.shared.raw));
        let swapchain = device.create_swapchain(self, config, old)?;

        self.swapchain = Some(swapchain);
        Ok(())
    }

    unsafe fn unconfigure(&mut self, device: &super::Device) {
        if let Some(sc) = self.swapchain.take() {
            let swapchain = sc.release_resources(&device.shared.raw);
            swapchain.loader.destroy_swapchain(swapchain.raw, None);
        }
    }
}
