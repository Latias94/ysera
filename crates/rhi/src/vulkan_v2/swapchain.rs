use crate::vulkan_v2::adapter::Surface;
use crate::vulkan_v2::device::Device;
use crate::vulkan_v2::image::{Image, ImageDescriptor, ImageType};
use crate::{DeviceError, Extent3d, MAX_FRAMES_IN_FLIGHT};
use ash::extensions::khr;
use ash::vk;
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub struct SwapchainProperties {
    pub surface_format: vk::SurfaceFormatKHR,
    pub present_mode: vk::PresentModeKHR,
    pub extent: vk::Extent2D,
}

pub struct SwapChainSupportDetail {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub surface_formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

#[derive(Clone, Copy, Default)]
pub struct SwapchainDescriptor {
    pub width: u32,
    pub height: u32,
    pub vsync: bool,
}

pub struct Swapchain {
    pub(crate) loader: khr::Swapchain,
    pub(crate) raw: vk::SwapchainKHR,
    pub config: SwapchainDescriptor,
    // pub images: Vec<Arc<crate::Image>>,
    pub properties: SwapchainProperties,
    pub support_detail: SwapChainSupportDetail,
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,

    // 防止在 swapchain drop 之前 drop
    pub device: Arc<Device>,
    pub surface: Arc<Surface>,
}

impl Swapchain {
    pub unsafe fn create(
        device: &Arc<Device>,
        surface: &Arc<Surface>,
        config: SwapchainDescriptor,
    ) -> Result<Self, DeviceError> {
        unsafe { Self::create_impl(device, surface, config, None) }
    }
}

impl Swapchain {
    unsafe fn create_impl(
        device: &Arc<Device>,
        surface: &Arc<Surface>,
        config: SwapchainDescriptor,
        old_swapchain: Option<vk::SwapchainKHR>,
    ) -> Result<Self, DeviceError> {
        profiling::scope!("create_swapchain");

        let support_detail = unsafe {
            SwapChainSupportDetail::new(device.adapter.raw, surface.loader(), surface.raw())
        }?;
        let properties = support_detail
            .get_ideal_swapchain_properties([config.width, config.height], config.vsync);
        let SwapchainProperties {
            surface_format,
            present_mode,
            extent,
        } = properties;

        let mut image_count = support_detail.capabilities.min_image_count + 1;
        image_count = image_count.max(MAX_FRAMES_IN_FLIGHT as u32 + 1);
        image_count = if support_detail.capabilities.max_image_count > 0 {
            image_count.min(support_detail.capabilities.max_image_count)
        } else {
            image_count
        };

        let (image_sharing_mode, queue_family_indices) =
            if device.indices.graphics_family != device.indices.present_family {
                (
                    // 图像可以在多个队列族间使用，不需要显式地改变图像所有权。
                    // 如果图形和呈现不是同一个队列族，我们使用协同模式来避免处理图像所有权问题。
                    vk::SharingMode::CONCURRENT,
                    vec![
                        device.indices.graphics_family.unwrap(),
                        device.indices.present_family.unwrap(),
                    ],
                )
            } else {
                // 一张图像同一时间只能被一个队列族所拥有，在另一队列族使用它之前，必须显式地改变图像所有权。
                // 这一模式下性能表现最佳。
                (vk::SharingMode::EXCLUSIVE, vec![])
            };

        let old_swapchain = match old_swapchain {
            None => vk::SwapchainKHR::null(),
            Some(swapchain) => swapchain,
        };

        let create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface.raw())
            .min_image_count(image_count)
            .image_color_space(surface_format.color_space)
            .image_format(surface_format.format)
            .image_extent(extent)
            // 这里，我们进行绘制操作
            // 如果要进行后处理，可以改成 TRANSFER_DST，让交换链 image 可以作为传输目的
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(image_sharing_mode)
            .queue_family_indices(&queue_family_indices)
            // 指定一个固定的变换操作，比如顺时针旋转 90 度或是水平翻转，这里不进行任何变换
            .pre_transform(support_detail.capabilities.current_transform)
            // 指定 alpha 通道是否被用来和窗口系统中的其它窗口进行混合操作。
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            // 我们不关心被窗口系统中的其它窗口遮挡的像素的颜色，这允许 Vulkan 采
            // 取一定的优化措施，但如果我们回读窗口的像素值就可能出现问题。
            .clipped(true)
            // 对于 VR 相关的应用程序来说，会使用更多的层次。
            .image_array_layers(1)
            .old_swapchain(old_swapchain);

        let loader = khr::Swapchain::new(device.instance.raw(), device.raw());
        let raw = unsafe { loader.create_swapchain(&create_info, None)? };
        log::debug!("Vulkan swapchain created. min_image_count: {}", image_count);

        let swapchain_images = unsafe { loader.get_swapchain_images(raw)? };

        let images: Vec<Arc<Image>> = swapchain_images
            .into_iter()
            .map(|vk_image| {
                Arc::new(Image {
                    raw: vk_image,
                    desc: ImageDescriptor {
                        image_type: ImageType::Tex2d,
                        usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
                        flags: vk::ImageCreateFlags::empty(),
                        format: vk::Format::B8G8R8A8_UNORM,
                        extent: Extent3d::new(extent.width, extent.height, 0),
                        tiling: vk::ImageTiling::OPTIMAL,
                        mip_levels: 1,
                        array_elements: 1,
                    },
                    views: Default::default(),
                })
            })
            .collect();

        let image_available_semaphores = (0..images.len())
            .map(|_| {
                unsafe {
                    device
                        .raw
                        .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                }
                .unwrap()
            })
            .collect();

        let render_finished_semaphores = (0..images.len())
            .map(|_| {
                unsafe {
                    device
                        .raw
                        .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                }
                .unwrap()
            })
            .collect();

        Ok(Swapchain {
            loader,
            raw,
            config,
            properties,
            support_detail,
            image_available_semaphores,
            render_finished_semaphores,
            device: device.clone(),
            surface: surface.clone(),
        })
    }
}

impl SwapChainSupportDetail {
    pub unsafe fn new(
        physical_device: vk::PhysicalDevice,
        surface: &khr::Surface,
        surface_khr: vk::SurfaceKHR,
    ) -> Result<SwapChainSupportDetail, DeviceError> {
        let capabilities =
            surface.get_physical_device_surface_capabilities(physical_device, surface_khr)?;
        let surface_formats =
            surface.get_physical_device_surface_formats(physical_device, surface_khr)?;
        let present_modes =
            surface.get_physical_device_surface_present_modes(physical_device, surface_khr)?;

        Ok(SwapChainSupportDetail {
            capabilities,
            surface_formats,
            present_modes,
        })
    }

    pub fn get_ideal_swapchain_properties(
        &self,
        preferred_dimensions: [u32; 2],
        vsync: bool,
    ) -> SwapchainProperties {
        let format = Self::choose_swapchain_format(&self.surface_formats);
        let present_mode = Self::choose_swapchain_present_mode(&self.present_modes, vsync);
        let extent = Self::choose_swapchain_extent(&self.capabilities, preferred_dimensions);
        SwapchainProperties {
            surface_format: format,
            present_mode,
            extent,
        }
    }

    fn choose_swapchain_format(
        available_formats: &Vec<vk::SurfaceFormatKHR>,
    ) -> vk::SurfaceFormatKHR {
        // check if list contains most widely used R8G8B8A8 format with nonlinear color space
        // if you want to use SRGB, check https://github.com/ocornut/imgui/issues/578
        // and https://github.com/ocornut/imgui/issues/4890
        for available_format in available_formats {
            if available_format.format == vk::Format::B8G8R8A8_UNORM
                && available_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                return *available_format;
            }
        }

        // return the first format from the list
        return *available_formats.first().unwrap();
    }

    fn choose_swapchain_present_mode(
        available_present_modes: &[vk::PresentModeKHR],
        vsync: bool,
    ) -> vk::PresentModeKHR {
        // 当展示新的图像时，就把它标记为待处理图像，在下一次（可能在下一个垂直刷新之后），系统将把它展示给用户。
        // 如果新的图像在此之前展示，那么将展示该图像，并会丢弃之前展示的图像。
        // 通常，如果要开启垂直同步，选择 VK_PRESENT_MODE_FIFO_KHR，并且如果要程序尽量快速运行，选择
        // VK_PRESENT_MODE_IMMEDIATE_KHR 或者 VK_PRESENT_MODE_MAILBOX_KHR。 VK_PRESENT_MODE_IMMEDIATE_KHR
        // 将会导致很多场景下可见的图像撕裂，但是会尽量少地造成延迟。 VK_PRESENT_MODE_MAILBOX_KHR
        // 以一定的间隔持续翻转，会造成垂直刷新的最大延迟，但是不会出现撕裂。

        let best_mode = if vsync {
            vec![vk::PresentModeKHR::FIFO_RELAXED, vk::PresentModeKHR::FIFO]
        } else {
            vec![vk::PresentModeKHR::MAILBOX, vk::PresentModeKHR::IMMEDIATE]
        };
        if vsync {}

        for &available_present_mode in available_present_modes.iter() {
            if best_mode.contains(&available_present_mode) {
                return available_present_mode;
            }
        }
        // 目前为止，还有许多驱动程序对 FIFO 呈现模式的支持不够好，
        // 所以，如果 Mailbox 呈现模式不可用，我们应该使用 IMMEDIATE 模式
        vk::PresentModeKHR::IMMEDIATE
    }

    fn choose_swapchain_extent(
        capabilities: &vk::SurfaceCapabilitiesKHR,
        preferred_dimensions: [u32; 2],
    ) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            use num::clamp;
            let width = preferred_dimensions[0];
            let height = preferred_dimensions[1];
            log::debug!("\t\tInner Window Size: ({}, {})", width, height);
            vk::Extent2D {
                width: clamp(
                    width,
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: clamp(
                    height,
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        }
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe {
            self.loader.destroy_swapchain(self.raw, None);
        }
        log::debug!("Swapchain destroyed.");
    }
}
