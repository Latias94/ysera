use std::sync::Arc;

use ash::vk;

use crate::vulkan::device::Device;
use crate::DeviceError;

#[derive(Clone)]
pub struct Sampler {
    device: Arc<Device>,
    sampler: vk::Sampler,
}

impl Sampler {
    pub fn raw(&self) -> vk::Sampler {
        self.sampler
    }

    pub unsafe fn new(device: &Arc<Device>, mip_levels: u32) -> Result<Self, DeviceError> {
        let create_info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .anisotropy_enable(true)
            .max_anisotropy(16.0)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            // 如果启用了比较功能，则首先会将纹素与一个值进行比较，并将比较结果用于过滤操作。这主要用于阴影贴图上的百分比接近过滤
            .compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .mip_lod_bias(0.0)
            .min_lod(0.0)
            // .min_lod(mip_levels as f32 / 2.0) // test mip_levels
            .max_lod(mip_levels as f32);
        let sampler = unsafe { device.raw().create_sampler(&create_info, None)? };
        Ok(Self {
            device: device.clone(),
            sampler,
        })
    }
}

impl Drop for Sampler {
    fn drop(&mut self) {
        unsafe {
            self.device.raw().destroy_sampler(self.sampler, None);
        }
    }
}
