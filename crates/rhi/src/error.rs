use thiserror::Error;

// refer to spec: https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkResult.html
#[derive(Debug, Error)]
pub enum DeviceError {
    #[error("out of memory")]
    OutOfMemory,
    #[error("not support")]
    NotSupport,
    #[error("The logical or physical device has been lost")]
    Lost,
    #[error("The physical device not meet requirement")]
    NotMeetRequirement,
    #[error("other reason: {0}")]
    Other(&'static str),
    #[error(transparent)]
    #[cfg(all(feature = "vulkan"))]
    Vulkan(#[from] ash::vk::Result),
    #[error("Allocation error: {0}")]
    Allocator(#[from] gpu_allocator::AllocationError),
    #[error(transparent)]
    AnyOther(#[from] anyhow::Error),
}

#[derive(Debug, Error)]
pub enum SurfaceError {
    #[error("A surface is no longer available")]
    Lost,
    #[error("A surface has changed in such a way that it is no longer compatible with the swapchain, \
    and further presentation requests using the swapchain will fail. Applications must query the new \
    surface properties and recreate their swapchain if they wish to continue presenting to the surface.")]
    OutOfDate,
    #[error(transparent)]
    #[cfg(all(feature = "vulkan"))]
    Vulkan(#[from] ash::vk::Result),
    #[error("other reason: {0}")]
    Other(&'static str),
}

#[derive(Debug, Error)]
pub enum ShaderError {
    #[error("compilation failed: {0:?}")]
    Compilation(String),
    #[error(transparent)]
    Device(#[from] DeviceError),
    #[error(transparent)]
    #[cfg(all(feature = "vulkan"))]
    VulkanError(#[from] ash::vk::Result),
    #[error(transparent)]
    #[cfg(all(feature = "dx12"))]
    Dx12Error(#[from] windows::core::Error),
}

#[derive(Clone, Debug, Eq, PartialEq, Error)]
pub enum InstanceError {
    #[error("Not supported")]
    NotSupport(),
    #[error(transparent)]
    #[cfg(all(feature = "vulkan"))]
    VulkanError(#[from] ash::vk::Result),
    #[error(transparent)]
    #[cfg(all(feature = "dx12"))]
    Dx12Error(#[from] windows::core::Error),
}
