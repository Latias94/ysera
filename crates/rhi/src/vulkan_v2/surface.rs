use ash::{extensions::*, vk};

pub struct Surface {
    raw: vk::SurfaceKHR,
    loader: khr::Surface,
}

impl Surface {
    pub fn raw(&self) -> vk::SurfaceKHR {
        self.raw
    }

    pub fn loader(&self) -> &khr::Surface {
        &self.loader
    }

    pub fn new(raw: vk::SurfaceKHR, loader: khr::Surface) -> Self {
        Self { raw, loader }
    }
}
