use crate::DeviceError;
use ash::vk;

impl crate::Semaphore<super::Api> for super::Semaphore {
    fn is_timeline_semaphore(&self) -> bool {
        self.is_timeline
    }

    unsafe fn wait(&self, value: u64, timeout: u64) -> Self {
        todo!()
    }

    unsafe fn signal(&self, value: u64) {
        todo!()
    }

    unsafe fn reset(&self) {
        todo!()
    }
}

impl super::Semaphore {
    pub unsafe fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            device.destroy_semaphore(self.raw, None);
        }
    }

    pub unsafe fn create(
        &mut self,
        device: &ash::Device,
        is_timeline: bool,
    ) -> Result<(), DeviceError> {
        let semaphore_type = if is_timeline {
            vk::SemaphoreType::TIMELINE
        } else {
            vk::SemaphoreType::BINARY
        };
        let mut sem_type_create_info =
            vk::SemaphoreTypeCreateInfo::builder().semaphore_type(semaphore_type);
        let create_info = vk::SemaphoreCreateInfo::builder().push_next(&mut sem_type_create_info);
        unsafe {
            device.create_semaphore(&create_info, None)?;
        }
        Ok(())
    }
}
