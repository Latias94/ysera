use crate::vulkan_v2::Api;
use crate::{
    ExposedAdapter, InstanceDescriptor, InstanceError, SurfaceConfiguration, SurfaceError,
};
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

impl crate::Instance<super::Api> for super::Instance {
    unsafe fn init(desc: &InstanceDescriptor) -> Result<Self, InstanceError> {
        todo!()
    }

    unsafe fn create_surface(
        &self,
        display_handle: RawDisplayHandle,
        window_handle: RawWindowHandle,
    ) -> Result<super::Surface, InstanceError> {
        todo!()
    }

    unsafe fn destroy_surface(&self, surface: super::Surface) {
        todo!()
    }

    unsafe fn enumerate_physical_devices(
        &self,
        surface: &super::Surface,
    ) -> Vec<ExposedAdapter<Api>> {
        todo!()
    }
}

impl crate::Surface<super::Api> for super::Surface {
    unsafe fn configure(
        &mut self,
        device: &super::Device,
        config: &SurfaceConfiguration,
    ) -> Result<(), SurfaceError> {
        todo!()
    }

    unsafe fn unconfigure(&mut self, device: &super::Device) {
        todo!()
    }
}
