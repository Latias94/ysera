use anyhow::Result;
use ash::vk;
use std::sync::Arc;

use crate::vulkan::context::Context;
use crate::vulkan::device::Device;

pub struct Semaphore {
    device: Arc<Device>,
    pub(crate) raw: vk::Semaphore,
}

impl Semaphore {
    pub(crate) unsafe fn new(device: &Arc<Device>) -> Result<Self> {
        let semaphore_info = vk::SemaphoreCreateInfo::builder();
        let raw = unsafe { device.raw().create_semaphore(&semaphore_info, None)? };

        Ok(Self {
            device: device.clone(),
            raw,
        })
    }
}

impl Context {
    pub unsafe fn create_semaphore(&self) -> Result<Semaphore> {
        unsafe { Semaphore::new(&self.device.clone()) }
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        unsafe {
            self.device.raw().destroy_semaphore(self.raw, None);
        }
    }
}

pub struct Fence {
    device: Arc<Device>,
    pub(crate) raw: vk::Fence,
}

impl Fence {
    pub(crate) unsafe fn new(
        device: &Arc<Device>,
        flags: Option<vk::FenceCreateFlags>,
    ) -> Result<Self> {
        let flags = flags.unwrap_or_else(vk::FenceCreateFlags::empty);

        let fence_info = vk::FenceCreateInfo::builder().flags(flags);
        let raw = unsafe { device.raw().create_fence(&fence_info, None)? };

        Ok(Self {
            device: device.clone(),
            raw,
        })
    }

    pub unsafe fn wait(&self, timeout: Option<u64>) -> Result<()> {
        let timeout = timeout.unwrap_or(u64::MAX);

        unsafe {
            self.device
                .raw()
                .wait_for_fences(&[self.raw], true, timeout)?
        };

        Ok(())
    }

    pub unsafe fn reset(&self) -> Result<()> {
        unsafe { self.device.raw().reset_fences(&[self.raw])? };

        Ok(())
    }
}

impl Context {
    pub unsafe fn create_fence(&self, flags: Option<vk::FenceCreateFlags>) -> Result<Fence> {
        unsafe { Fence::new(&self.device.clone(), flags) }
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        unsafe {
            self.device.raw().destroy_fence(self.raw, None);
        }
    }
}
