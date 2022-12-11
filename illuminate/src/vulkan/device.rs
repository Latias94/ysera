pub struct Device {
    /// Loads device local functions.
    raw: ash::Device,
}

impl Device {
    pub fn new(raw: ash::Device) -> Self {
        Self { raw }
    }
}
