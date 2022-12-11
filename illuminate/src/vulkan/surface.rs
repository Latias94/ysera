use ash::{extensions::*, vk};

pub struct Surface {
    raw: vk::SurfaceKHR,
    fp: khr::Surface,
}

impl Surface {
    pub fn new(raw: vk::SurfaceKHR, fp: khr::Surface) -> Self {
        Self { raw, fp }
    }
    pub fn vk_surface_khr(&self) -> vk::SurfaceKHR {
        self.raw
    }

    pub fn khr_surface(&self) -> &khr::Surface {
        &self.fp
    }
}
