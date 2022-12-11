use crate::vulkan::adapter::Adapter;
use crate::vulkan::device::Device;
use crate::vulkan::instance::Instance;
use crate::vulkan::surface::Surface;
use crate::vulkan::texture_view::TextureView;
use crate::{DeviceError, QueueFamilyIndices};
use ash::extensions::khr;
use ash::vk;
use std::rc::Rc;
use typed_builder::TypedBuilder;

pub struct Swapchain {
    raw: vk::SwapchainKHR,
    loader: khr::Swapchain,
    device: Rc<Device>,
    family_index: QueueFamilyIndices,
    textures: Vec<vk::Image>,
    texture_views: Vec<TextureView>,
    surface_format: vk::SurfaceFormatKHR,
    extent: vk::Extent2D,
    capabilities: vk::SurfaceCapabilitiesKHR,
}

#[derive(Clone, Copy, Debug)]
struct SwapchainProperties {
    pub surface_format: vk::SurfaceFormatKHR,
    pub present_mode: vk::PresentModeKHR,
    pub extent: vk::Extent2D,
}

struct SwapChainSupportDetail {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

#[derive(Clone, TypedBuilder)]
pub struct SwapchainDescriptor<'a> {
    pub adapter: &'a Adapter,
    pub surface: &'a Surface,
    pub instance: &'a Instance,
    pub device: &'a Rc<Device>,
    pub queue: vk::Queue,
    pub queue_family: QueueFamilyIndices,
    pub dimensions: [u32; 2],
}

impl Swapchain {
    pub fn raw(&self) -> vk::SwapchainKHR {
        self.raw
    }

    pub fn loader(&self) -> &khr::Swapchain {
        &self.loader
    }

    pub fn new(desc: &SwapchainDescriptor) -> Result<Self, DeviceError> {
        let (swapchain_loader, swapchain, properties, support) = Self::create_swapchain(
            desc.adapter,
            desc.surface,
            desc.instance,
            desc.device,
            &desc.queue_family,
            desc.dimensions,
        )?;
        // 交换链图像由交换链自己负责创建，并在交换链清除时自动被清除，不需要我们自己进行创建和清除操作。
        let swapchain_textures = unsafe { swapchain_loader.get_swapchain_images(swapchain)? };

        let mut capabilities = support.capabilities;

        capabilities.current_extent.width = capabilities.current_extent.width.max(1);
        capabilities.current_extent.height = capabilities.current_extent.height.max(1);

        let texture_views = swapchain_textures
            .iter()
            .map(|i| {
                TextureView::new_color_texture_view(
                    Some("swapchain texture view"),
                    desc.device,
                    *i,
                    properties.surface_format.format,
                )
                .unwrap()
            })
            .collect();
        // // 返回的 vk::PhysicalDeviceMemoryProperties 结构有两个数组内存类型和内存堆。内存堆是不同的内存资源，
        // // 如专用 VRAM 和当 VRAM 耗尽时 RAM 中的交换空间。不同类型的内存存在于这些堆中。现在我们只关心内存的类型，
        // // 而不关心它来自的堆，但是可以想象这可能会影响性能。
        // let mem_properties = {
        //     // profiling::scope!("vkGetPhysicalDeviceMemoryProperties");
        //     instance_fp.get_physical_device_memory_properties(self.raw)
        // };
        // // 我们首先找到适合缓冲区本身的内存类型
        // // 来自 requirements 参数的内存类型位字段将用于指定合适的内存类型的位字段。
        // // 这意味着我们可以通过简单地遍历它们并检查相应的位是否设置为 1 来找到合适的内存类型的索引。
        // let memory_types =
        //     &mem_properties.memory_types[..mem_properties.memory_type_count as usize];
        // let valid_memory_types: u32 = memory_types.iter().enumerate().fold(0, |u, (i, mem)| {
        //     if self.known_memory_flags.contains(mem.property_flags) {
        //         u | (1 << i)
        //     } else {
        //         u
        //     }
        // });
        // let swapchain_loader = khr::Swapchain::new(&instance_fp, &ash_device);
        // let queue_family_index = indices.graphics_family.unwrap();
        // let raw_queue = {
        //     profiling::scope!("vkGetDeviceQueue");
        //     // queueFamilyIndex is the index of the queue family to which the queue belongs.
        //     // queueIndex is the index within this queue family of the queue to retrieve.
        //     ash_device.get_device_queue(queue_family_index, 0)
        // };
        Ok(Self {
            raw: swapchain,
            loader: swapchain_loader,
            device: desc.device.clone(),
            family_index: desc.queue_family,
            textures: swapchain_textures,
            surface_format: properties.surface_format,
            extent: properties.extent,
            capabilities,
            texture_views,
        })
    }

    fn create_swapchain(
        adapter: &Adapter,
        surface: &Surface,
        instance: &Instance,
        device: &Device,
        queue_family: &QueueFamilyIndices,
        dimensions: [u32; 2],
    ) -> Result<
        (
            khr::Swapchain,
            vk::SwapchainKHR,
            SwapchainProperties,
            SwapChainSupportDetail,
        ),
        DeviceError,
    > {
        profiling::scope!("create_swapchain");

        let swapchain_support =
            unsafe { SwapChainSupportDetail::new(adapter.raw(), surface.loader(), surface.raw()) }?;
        let properties = swapchain_support.get_ideal_swapchain_properties(dimensions);
        let SwapchainProperties {
            surface_format,
            present_mode,
            extent,
        } = properties;

        let image_count = swapchain_support.capabilities.min_image_count + 1;
        let image_count = if swapchain_support.capabilities.max_image_count > 0 {
            image_count.min(swapchain_support.capabilities.max_image_count)
        } else {
            image_count
        };
        let (image_sharing_mode, queue_family_indices) =
            if queue_family.graphics_family != queue_family.present_family {
                (
                    // 图像可以在多个队列族间使用，不需要显式地改变图像所有权。
                    // 如果图形和呈现不是同一个队列族，我们使用协同模式来避免处理图像所有权问题。
                    vk::SharingMode::CONCURRENT,
                    vec![
                        queue_family.graphics_family.unwrap(),
                        queue_family.present_family.unwrap(),
                    ],
                )
            } else {
                // 一张图像同一时间只能被一个队列族所拥有，在另一队列族使用它之前，必须显式地改变图像所有权。
                // 这一模式下性能表现最佳。
                (vk::SharingMode::EXCLUSIVE, vec![])
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
            .pre_transform(swapchain_support.capabilities.current_transform)
            // 指定 alpha 通道是否被用来和窗口系统中的其它窗口进行混合操作。
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            // 我们不关心被窗口系统中的其它窗口遮挡的像素的颜色，这允许 Vulkan 采
            // 取一定的优化措施，但如果我们回读窗口的像素值就可能出现问题。
            .clipped(true)
            // 对于 VR 相关的应用程序来说，会使用更多的层次。
            .image_array_layers(1)
            .build();
        let swapchain_loader = khr::Swapchain::new(instance.raw(), device.raw());
        let swapchain = unsafe { swapchain_loader.create_swapchain(&create_info, None)? };
        log::info!("Vulkan swapchain created.");

        Ok((swapchain_loader, swapchain, properties, swapchain_support))
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
        let formats = surface.get_physical_device_surface_formats(physical_device, surface_khr)?;
        let present_modes =
            surface.get_physical_device_surface_present_modes(physical_device, surface_khr)?;

        Ok(SwapChainSupportDetail {
            capabilities,
            formats,
            present_modes,
        })
    }

    pub fn get_ideal_swapchain_properties(
        &self,
        preferred_dimensions: [u32; 2],
    ) -> SwapchainProperties {
        let format = Self::choose_swapchain_format(&self.formats);
        let present_mode = Self::choose_swapchain_present_mode(&self.present_modes);
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
        for available_format in available_formats {
            if available_format.format == vk::Format::B8G8R8A8_SRGB
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
    ) -> vk::PresentModeKHR {
        // 当展示新的图像时，就把它标记为待处理图像，在下一次（可能在下一个垂直刷新之后），系统将把它展示给用户。
        // 如果新的图像在此之前展示，那么将展示该图像，并会丢弃之前展示的图像。
        // 通常，如果要开启垂直同步，选择 VK_PRESENT_MODE_FIFO_KHR，并且如果要程序尽量快速运行，选择
        // VK_PRESENT_MODE_IMMEDIATE_KHR 或者 VK_PRESENT_MODE_MAILBOX_KHR。 VK_PRESENT_MODE_IMMEDIATE_KHR
        // 将会导致很多场景下可见的图像撕裂，但是会尽量少地造成延迟。 VK_PRESENT_MODE_MAILBOX_KHR
        // 以一定的间隔持续翻转，会造成垂直刷新的最大延迟，但是不会出现撕裂。
        let mut best_mode = vk::PresentModeKHR::FIFO;
        for &available_present_mode in available_present_modes.iter() {
            if available_present_mode == vk::PresentModeKHR::MAILBOX {
                return available_present_mode;
            } else if available_present_mode == vk::PresentModeKHR::IMMEDIATE {
                // 目前为止，还有许多驱动程序对 FIFO 呈现模式的支持不够好，
                // 所以，如果 Mailbox 呈现模式不可用，我们应该使用 IMMEDIATE 模式
                best_mode = vk::PresentModeKHR::IMMEDIATE;
            }
        }

        best_mode
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
        log::debug!("swapchain destroyed.");
    }
}