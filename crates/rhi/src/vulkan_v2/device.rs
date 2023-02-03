use std::ffi::CStr;
use std::sync::Arc;

use ash::extensions::khr;
use ash::vk;

use crate::vulkan_v2::{conv, DeviceShared};

impl crate::Device<super::Api> for super::Device {
    unsafe fn present_queue(&self, image_index: u32, wait_semaphore: &[super::Semaphore]) {
        todo!()
    }

    unsafe fn shutdown(self, queue: super::Queue) {
        self.shared.free_resources();
    }
}

impl DeviceShared {
    unsafe fn free_resources(&self) {
        self.render_passes
            .iter()
            .for_each(|&x| self.raw.destroy_render_pass(x, None));
        self.framebuffers
            .iter()
            .for_each(|&x| self.raw.destroy_framebuffer(x, None));

        self.raw.destroy_device(None);
    }

    pub unsafe fn set_object_name(
        &self,
        object_type: vk::ObjectType,
        object: impl vk::Handle,
        name: &str,
    ) {
        let debug_utils = match &self.instance.debug_utils {
            Some(utils) => utils,
            None => return,
        };

        let mut buffer: [u8; 64] = [0u8; 64];
        let buffer_vec: Vec<u8>;

        // Append a null terminator to the string
        let name_bytes = if name.len() < buffer.len() {
            // Common case, string is very small. Allocate a copy on the stack.
            buffer[..name.len()].copy_from_slice(name.as_bytes());
            // Add null terminator
            buffer[name.len()] = 0;
            &buffer[..name.len() + 1]
        } else {
            // Less common case, the string is large.
            // This requires a heap allocation.
            buffer_vec = name
                .as_bytes()
                .iter()
                .cloned()
                .chain(std::iter::once(0))
                .collect();
            &buffer_vec
        };
        let extension = &debug_utils.extension;
        let _result = extension.set_debug_utils_object_name(
            self.raw.handle(),
            &vk::DebugUtilsObjectNameInfoEXT::builder()
                .object_type(object_type)
                .object_handle(object.as_raw())
                .object_name(CStr::from_bytes_with_nul_unchecked(name_bytes)),
        );
    }
}

impl super::Device {
    pub unsafe fn create_swapchain(
        &self,
        surface: &mut super::Surface,
        config: &crate::SurfaceConfiguration,
        old_swapchain: Option<super::Swapchain>,
    ) -> Result<super::Swapchain, crate::SurfaceError> {
        profiling::scope!("create_swapchain");

        let old_swapchain = match old_swapchain {
            Some(osc) => osc.raw,
            None => vk::SwapchainKHR::null(),
        };
        let info = vk::SwapchainCreateInfoKHR::builder()
            .flags(vk::SwapchainCreateFlagsKHR::empty())
            .surface(surface.shared.raw)
            .min_image_count(config.swap_chain_size)
            .image_format(conv::map_texture_format(config.format))
            .image_extent(vk::Extent2D {
                width: config.extent.width,
                height: config.extent.height,
            })
            // 对于 VR 相关的应用程序来说，会使用更多的层次。
            .image_array_layers(config.extent.depth_or_array_layers)
            // 这里，我们进行绘制操作
            // 如果要进行后处理，可以改成 TRANSFER_DST，让交换链 image 可以作为传输目的
            .image_usage(conv::map_image_usage(config.usage))
            // vk::SharingMode::CONCURRENT 图像可以在多个队列族间使用，不需要显式地改变图像所有权。
            // 如果图形和呈现不是同一个队列族，我们使用协同模式来避免处理图像所有权问题。
            // vk::SharingMode::EXCLUSIVE 一张图像同一时间只能被一个队列族所拥有，在另一队列族使用它之前，
            // 必须显式地改变图像所有权。这一模式下性能表现最佳。
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            // 指定一个固定的变换操作，比如顺时针旋转90度或是水平翻转，这里不进行任何变换
            .pre_transform(vk::SurfaceTransformFlagsKHR::IDENTITY)
            // 指定 alpha 通道是否被用来和窗口系统中的其它窗口进行混合操作。
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(conv::map_present_mode(config.present_mode))
            // 我们不关心被窗口系统中的其它窗口遮挡的像素的颜色，这允许 Vulkan 采
            // 取一定的优化措施，但如果我们回读窗口的像素值就可能出现问题。
            .clipped(true)
            // 交换链需要重建，重建时需要之前的交换链
            .old_swapchain(old_swapchain);
        let swapchain_loader = khr::Swapchain::new(&surface.instance.raw, &self.shared.raw);
        let result = {
            profiling::scope!("vkCreateSwapchainKHR");
            swapchain_loader.create_swapchain(&info, None)
        };

        if old_swapchain != vk::SwapchainKHR::null() {
            swapchain_loader.destroy_swapchain(old_swapchain, None)
        }
        let raw = match result {
            Ok(swapchain) => swapchain,
            Err(error) => {
                return Err(match error {
                    vk::Result::ERROR_SURFACE_LOST_KHR => crate::SurfaceError::Lost,
                    vk::Result::ERROR_NATIVE_WINDOW_IN_USE_KHR => {
                        crate::SurfaceError::Other("Native window is in use")
                    }
                    other => crate::DeviceError::from(other).into(),
                })
            }
        };

        // 交换链图像由交换链自己负责创建，并在交换链清除时自动被清除，不需要我们自己进行创建和清除操作。
        let swapchain_images = swapchain_loader
            .get_swapchain_images(raw)
            .map_err(crate::DeviceError::from)?;

        let vk_info = vk::FenceCreateInfo::builder().build();
        let fence = self
            .shared
            .raw
            .create_fence(&vk_info, None)
            .map_err(crate::DeviceError::from)?;

        log::info!("Vulkan swapchain created.");

        Ok(super::Swapchain {
            raw,
            loader: swapchain_loader,
            surface: surface.shared.clone(),
            device: Arc::clone(&self.shared),
            fence,
            images: swapchain_images,
            image_index: 0,
            config: config.clone(),
        })
    }
}
