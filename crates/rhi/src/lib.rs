#![allow(clippy::missing_safety_doc)]

extern crate alloc;
extern crate core;

use core::fmt::Debug;
pub use error::*;

use crate::vulkan::instance::InstanceFlags;

mod error;
mod gui;
pub mod types;
pub mod vulkan;
mod vulkan_v2;

pub use types::*;

pub use ash;
pub use winit;

const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub mod api {
    // #[cfg(feature = "vulkan")]
    pub use super::vulkan_v2::Api as Vulkan;
}

// refer to wgpu-hal
pub trait GraphicsApi: Clone + Sized {
    type Instance: Instance<Self>;
    type Surface: Surface<Self>;
    type PhysicalDevice: PhysicalDevice<Self>;
    type Device: Device<Self>;

    type Queue: Queue<Self>;
    type Buffer: Debug + Send + Sync + 'static;
    type Image: Debug + Send + Sync + 'static;
    type Sampler: Debug + Send + Sync;
    type Pipeline: Send + Sync;
    type Shader: Debug + Send + Sync;
}

pub trait Instance<Api: GraphicsApi>: Sized {
    unsafe fn init(desc: &InstanceDescriptor) -> Result<Self, InstanceError>;
    unsafe fn create_surface(
        &self,
        display_handle: raw_window_handle::RawDisplayHandle,
        window_handle: raw_window_handle::RawWindowHandle,
    ) -> Result<Api::Surface, InstanceError>;
    unsafe fn destroy_surface(&self, surface: Api::Surface);
    unsafe fn enumerate_physical_devices(
        &self,
        surface: &Api::Surface,
    ) -> Vec<ExposedPhysicalDevice<Api>>;
}

pub trait PhysicalDevice<Api: GraphicsApi>: Send + Sync {
    unsafe fn open(&self, features: Features) -> Result<OpenDevice<Api>, DeviceError>;
    unsafe fn surface_capabilities(&self, surface: &Api::Surface) -> Option<SurfaceCapabilities>;
}

pub trait Surface<Api: GraphicsApi> {
    unsafe fn configure(
        &mut self,
        device: &Api::Device,
        config: &SurfaceConfiguration,
    ) -> Result<(), SurfaceError>;

    unsafe fn unconfigure(&mut self, device: &Api::Device);
}

pub trait Device<Api: GraphicsApi> {
    unsafe fn shutdown(self, queue: Api::Queue);
}

#[derive(Debug)]
pub struct OpenDevice<Api: GraphicsApi> {
    pub device: Api::Device,
    pub queue: Api::Queue,
}

pub trait Queue<Api: GraphicsApi> {}

pub trait Buffer<Api: GraphicsApi> {}

pub trait Image<Api: GraphicsApi> {}

pub trait Sampler<Api: GraphicsApi> {}

pub trait Pipeline<Api: GraphicsApi> {}

pub trait Shader<Api: GraphicsApi> {}
