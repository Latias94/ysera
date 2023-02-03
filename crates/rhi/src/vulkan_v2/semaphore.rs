use ash::vk;

use crate::{DeviceError, Label};

impl crate::Semaphore<super::Api> for super::Semaphore {
    fn is_timeline_semaphore(&self) -> bool {
        self.is_timeline
    }

    unsafe fn create(
        device: &super::Device,
        is_timeline: bool,
        name: Label,
    ) -> Result<super::Semaphore, DeviceError> {
        let raw = unsafe { Self::create_raw(&device.shared, is_timeline)? };
        if let Some(name) = name {
            unsafe {
                device
                    .shared
                    .set_object_name(vk::ObjectType::SEMAPHORE, raw, name)
            };
        }
        Ok(Self {
            raw,
            is_timeline,
            device: device.shared.clone(),
        })
    }

    unsafe fn wait(&self, value: u64, timeout: u64) -> Result<(), DeviceError> {
        if !self.is_timeline {
            log::error!("Semaphore::wait is called on a binary Semaphore!");
        }
        let semaphores = &[self.raw];
        let values = &[value];
        let wait_info = vk::SemaphoreWaitInfo::builder()
            .semaphores(semaphores)
            .values(values);
        unsafe {
            self.device.raw.wait_semaphores(&wait_info, timeout)?;
        }
        Ok(())
    }

    unsafe fn signal(&self, value: u64) -> Result<(), DeviceError> {
        if !self.is_timeline {
            log::error!("Semaphore::signal is called on a binary Semaphore!");
        }
        let signal_info = vk::SemaphoreSignalInfo::builder()
            .semaphore(self.raw)
            .value(value);

        unsafe {
            self.device.raw.signal_semaphore(&signal_info)?;
        }
        Ok(())
    }

    unsafe fn reset(&mut self) -> Result<(), DeviceError> {
        self.raw = unsafe {
            self.device.raw.device_wait_idle()?;
            self.destroy_raw(&self.device.raw);
            Self::create_raw(&self.device, self.is_timeline)?
        };
        Ok(())
    }

    unsafe fn get_value(&self) -> Result<u64, DeviceError> {
        let value = unsafe { self.device.raw.get_semaphore_counter_value(self.raw)? };
        Ok(value)
    }
}

impl super::Semaphore {
    pub unsafe fn destroy_raw(&self, device: &ash::Device) {
        unsafe {
            device.destroy_semaphore(self.raw, None);
        }
    }

    pub unsafe fn create_raw(
        device: &super::DeviceShared,
        is_timeline: bool,
    ) -> Result<vk::Semaphore, DeviceError> {
        let mut create_info = vk::SemaphoreCreateInfo::builder();

        let mut sem_type_create_info =
            vk::SemaphoreTypeCreateInfo::builder().semaphore_type(vk::SemaphoreType::TIMELINE);
        if is_timeline {
            create_info = create_info.push_next(&mut sem_type_create_info);
        }
        let raw = unsafe { device.raw.create_semaphore(&create_info, None)? };
        Ok(raw)
    }
}

impl Drop for super::Semaphore {
    fn drop(&mut self) {
        unsafe {
            self.destroy_raw(&self.device.raw);
        }
    }
}
