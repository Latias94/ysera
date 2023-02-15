use std::cell::{Ref, RefCell, RefMut};
use std::sync::Arc;

use ash::extensions::khr;
use ash::vk;
use typed_builder::TypedBuilder;

use math::prelude::*;

use crate::types::{
    AttachmentLoadOp, AttachmentStoreOp, Color, ImageFormat, QueueFamilyIndices,
    RenderPassClearFlags, RenderPassDescriptor, RenderTargetAttachment, RenderTargetAttachmentType,
};
use crate::vulkan::command_buffer::{CommandBuffer, CommandBufferState};
use crate::vulkan::context::Context;
use crate::vulkan::device::Device;
use crate::vulkan::framebuffer::{Framebuffer, FramebufferDescriptor};
use crate::vulkan::image::{DepthImageDescriptor, Image, ImageDescriptor};
use crate::vulkan::image_view::ImageView;
use crate::vulkan::render_pass::RenderPass;
use crate::vulkan::sync::{Fence, Semaphore};
use crate::vulkan::texture::{VulkanTexture, VulkanTextureDescriptor};
use crate::{DeviceError, SurfaceError};

pub struct Swapchain {
    raw: vk::SwapchainKHR,
    loader: khr::Swapchain,
    context: Context,
    command_buffers: Vec<RefCell<CommandBuffer>>,
    render_pass: RefCell<RenderPass>,
    framebuffers: Vec<Framebuffer>,
    pub swapchain_images: Vec<vk::Image>,
    pub image_views: Vec<ImageView>,
    pub surface_format: vk::SurfaceFormatKHR,
    // depth_format: vk::Format,
    pub depth_texture: VulkanTexture,
    pub extent: vk::Extent2D,
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub max_frames_in_flight: u32,
    pub semaphores: Vec<SwapchainSemaphore>,
    pub fences: Vec<Fence>,
    pub image_index: u32,
}

pub struct RenderSystemState {
    projection: Mat4,
    view: Mat4,
    near_clip: f32,
    far_clip: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct AcquiredImage {
    pub image_index: u32,
    pub is_suboptimal: bool,
}

#[derive(TypedBuilder)]
pub struct SwapchainDescriptor<'a> {
    pub context: &'a Context,
    pub dimensions: [u32; 2],
    pub old_swapchain: Option<vk::SwapchainKHR>,
    pub max_frames_in_flight: u32,
    pub queue_family: QueueFamilyIndices,
}

// #[derive(Clone, TypedBuilder, Hash, PartialEq, Eq)]
// pub struct FramebufferDescriptor {
//     pub render_pass: vk::RenderPass,
//     pub image_views: Vec<vk::ImageView>,
//     pub dimensions: [u32; 2],
// }

pub struct SwapchainSemaphore {
    present: Semaphore,
    render: Semaphore,
}

impl Swapchain {
    pub fn raw(&self) -> vk::SwapchainKHR {
        self.raw
    }

    pub fn loader(&self) -> &khr::Swapchain {
        &self.loader
    }

    pub fn surface_format(&self) -> vk::Format {
        self.surface_format.format
    }

    pub fn extent(&self) -> vk::Extent2D {
        self.extent
    }

    pub fn render_pass(&self) -> &RefCell<RenderPass> {
        &self.render_pass
    }

    pub fn current_image_index(&self) -> u32 {
        self.image_index
    }

    pub fn get_draw_command_buffer(&self, index: u32) -> &RefCell<CommandBuffer> {
        &self.command_buffers[index as usize]
    }

    pub fn get_framebuffer(&self, index: u32) -> &Framebuffer {
        &self.framebuffers[index as usize]
    }

    pub unsafe fn new(desc: &SwapchainDescriptor) -> anyhow::Result<Self> {
        let swapchain_objects = unsafe { Self::create_swapchain(desc)? };

        let SwapchainRelatedObjects {
            extent,
            capabilities,
            render_pass,
            framebuffers,
            depth_texture,
            swapchain_images,
            image_views,
        } = unsafe { Self::create_swapchain_related_objects(desc, &swapchain_objects)? };

        let SwapchainObjects {
            swapchain_loader,
            swapchain,
            properties,
            ..
        } = swapchain_objects;

        let device = &desc.context.device;

        let command_buffers =
            unsafe { Self::create_draw_command_buffers(device, desc.max_frames_in_flight)? };

        let (semaphores, fences) =
            unsafe { Self::create_semaphores_and_fences(device, desc.max_frames_in_flight)? };

        let swapchain = Self {
            raw: swapchain,
            loader: swapchain_loader,
            context: desc.context.clone(),
            render_pass: RefCell::new(render_pass),
            framebuffers,
            command_buffers,
            swapchain_images,
            image_views,
            surface_format: properties.surface_format,
            extent,
            capabilities,
            depth_texture,
            max_frames_in_flight: desc.max_frames_in_flight,
            semaphores,
            fences,
            image_index: 0,
        };

        Ok(swapchain)
    }

    pub unsafe fn resize(
        &mut self,
        context: &Context,
        width: u32,
        height: u32,
    ) -> Result<(), DeviceError> {
        let desc = &SwapchainDescriptor {
            context,
            dimensions: [width, height],
            old_swapchain: Some(self.raw),
            max_frames_in_flight: self.max_frames_in_flight,
            queue_family: context.device.queue_family_indices(),
        };
        let swapchain_objects = unsafe { Self::create_swapchain(desc)? };

        unsafe {
            self.destroy_all();
        }

        let SwapchainRelatedObjects {
            extent,
            capabilities,
            render_pass,
            framebuffers,
            depth_texture,
            swapchain_images,
            image_views,
        } = unsafe { Self::create_swapchain_related_objects(desc, &swapchain_objects)? };

        let SwapchainObjects {
            swapchain_loader,
            swapchain,
            properties,
            ..
        } = swapchain_objects;

        self.raw = swapchain;
        self.loader = swapchain_loader;
        self.framebuffers = framebuffers;
        self.render_pass = RefCell::new(render_pass);
        self.capabilities = capabilities;
        self.depth_texture = depth_texture;
        self.swapchain_images = swapchain_images;
        self.surface_format = properties.surface_format;
        self.image_views = image_views;
        self.extent = extent;
        Ok(())
    }

    pub unsafe fn cleanup(&mut self) {
        unsafe { self.destroy_all() }
    }

    pub unsafe fn begin_frame(&mut self) -> Result<(), DeviceError> {
        let in_flight_fence = self.fences[self.image_index as usize].raw;
        let in_flight_fences = [in_flight_fence];
        unsafe {
            self.context
                .device
                .raw()
                .wait_for_fences(&in_flight_fences, true, u64::MAX)?;
        };

        let result = unsafe {
            self.acquire_next_image(
                u64::MAX,
                &self.semaphores[self.image_index as usize].present,
            )?
        };

        unsafe {
            self.context.device.raw().reset_fences(&in_flight_fences)?;
        }
        Ok(())
    }

    pub unsafe fn present(&mut self) -> Result<(), DeviceError> {
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

        let wait_semaphores = &[self.semaphores[self.image_index as usize].present.raw];
        let signal_semaphores = &[self.semaphores[self.image_index as usize].render.raw];

        let command_buffer = &mut self.command_buffers[self.image_index as usize];
        let command_buffers = [command_buffer.borrow().raw()];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(signal_semaphores)
            .build();
        let in_flight_fence = self.fences[self.image_index as usize].raw;
        let device = &self.context.device;
        unsafe {
            device
                .raw()
                .queue_submit(device.graphics_queue(), &[submit_info], in_flight_fence)?;
        }

        command_buffer
            .borrow_mut()
            .set_state(CommandBufferState::Submitted);

        match self.queue_present(device.present_queue(), self.image_index, signal_semaphores) {
            Ok(suboptimal) => suboptimal,
            Err(SurfaceError::OutOfDate) => {
                self.resize(&self.context.clone(), self.extent.width, self.extent.height)?;
                return Ok(());
            }
            Err(e) => panic!("failed to acquire_next_image. Err: {}", e),
        };
        self.image_index = (self.image_index + 1) % self.max_frames_in_flight;

        Ok(())
    }
}

/// temp objects
struct SwapchainProperties {
    pub surface_format: vk::SurfaceFormatKHR,
    pub present_mode: vk::PresentModeKHR,
    pub extent: vk::Extent2D,
}

/// temp objects
struct SwapChainSupportDetail {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub surface_formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

/// temp objects
pub struct SwapchainObjects {
    swapchain_loader: khr::Swapchain,
    swapchain: vk::SwapchainKHR,
    properties: SwapchainProperties,
    support: SwapChainSupportDetail,
    image_count: u32,
}

/// temp objects
pub struct SwapchainRelatedObjects {
    extent: vk::Extent2D,
    capabilities: vk::SurfaceCapabilitiesKHR,
    render_pass: RenderPass,
    framebuffers: Vec<Framebuffer>,
    depth_texture: VulkanTexture,
    swapchain_images: Vec<vk::Image>,
    image_views: Vec<ImageView>,
}

impl Swapchain {
    unsafe fn create_swapchain(
        desc: &SwapchainDescriptor,
    ) -> Result<SwapchainObjects, DeviceError> {
        profiling::scope!("create_swapchain");

        let support = unsafe {
            SwapChainSupportDetail::new(
                desc.context.device.adapter().raw(),
                desc.context.surface.loader(),
                desc.context.surface.raw(),
            )
        }?;
        let properties = support.get_ideal_swapchain_properties(desc.dimensions);
        let SwapchainProperties {
            surface_format,
            present_mode,
            extent,
        } = properties;

        let mut image_count = support.capabilities.min_image_count + 1;
        image_count = image_count.max(desc.max_frames_in_flight);
        image_count = if support.capabilities.max_image_count > 0 {
            image_count.min(support.capabilities.max_image_count)
        } else {
            image_count
        };

        let (image_sharing_mode, queue_family_indices) =
            if desc.queue_family.graphics_family != desc.queue_family.present_family {
                (
                    // 图像可以在多个队列族间使用，不需要显式地改变图像所有权。
                    // 如果图形和呈现不是同一个队列族，我们使用协同模式来避免处理图像所有权问题。
                    vk::SharingMode::CONCURRENT,
                    vec![
                        desc.queue_family.graphics_family.unwrap(),
                        desc.queue_family.present_family.unwrap(),
                    ],
                )
            } else {
                // 一张图像同一时间只能被一个队列族所拥有，在另一队列族使用它之前，必须显式地改变图像所有权。
                // 这一模式下性能表现最佳。
                (vk::SharingMode::EXCLUSIVE, vec![])
            };

        let old_swapchain = match desc.old_swapchain {
            None => vk::SwapchainKHR::null(),
            Some(swapchain) => swapchain,
        };

        let create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(desc.context.surface.raw())
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
            .pre_transform(support.capabilities.current_transform)
            // 指定 alpha 通道是否被用来和窗口系统中的其它窗口进行混合操作。
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            // 我们不关心被窗口系统中的其它窗口遮挡的像素的颜色，这允许 Vulkan 采
            // 取一定的优化措施，但如果我们回读窗口的像素值就可能出现问题。
            .clipped(true)
            // 对于 VR 相关的应用程序来说，会使用更多的层次。
            .image_array_layers(1)
            .old_swapchain(old_swapchain);

        let swapchain_loader = khr::Swapchain::new(
            desc.context.instance.shared_instance().raw(),
            desc.context.device.raw(),
        );
        let swapchain = unsafe { swapchain_loader.create_swapchain(&create_info, None)? };
        log::debug!("Vulkan swapchain created. min_image_count: {}", image_count);

        Ok(SwapchainObjects {
            swapchain_loader,
            swapchain,
            properties,
            support,
            image_count,
        })
    }

    unsafe fn create_swapchain_related_objects(
        desc: &SwapchainDescriptor,
        swapchain_objects: &SwapchainObjects,
    ) -> Result<SwapchainRelatedObjects, DeviceError> {
        let SwapchainObjects {
            swapchain_loader,
            swapchain,
            properties,
            support,
            ..
        } = swapchain_objects;
        let device = &desc.context.device;
        let extent = properties.extent;
        let surface_format = properties.surface_format.format;
        // 交换链图像由交换链自己负责创建，并在交换链清除时自动被清除，不需要我们自己进行创建和清除操作。
        let swapchain_images = unsafe { swapchain_loader.get_swapchain_images(swapchain.clone())? };

        let mut capabilities = support.capabilities;

        capabilities.current_extent.width = capabilities.current_extent.width.max(1);
        capabilities.current_extent.height = capabilities.current_extent.height.max(1);
        let image_views = swapchain_images
            .iter()
            .map(|i| {
                ImageView::new_color_image_view(
                    Some("swapchain image view"),
                    device,
                    *i,
                    surface_format,
                    1,
                )
            })
            .collect::<Result<Vec<ImageView>, DeviceError>>()?;

        let depth_texture = unsafe { Self::create_depth_objects(desc, extent)? };

        // Note that we recreate the renderpass here. In theory it can be possible for the swap chain
        // image format to change during an applications' lifetime, e.g. when moving a window from
        // an standard range to an high dynamic range monitor. This may require the application to
        // recreate the renderpass to make sure the change between dynamic ranges is properly reflected.
        let render_pass_desc = RenderPassDescriptor {
            label: Some("Swapchain Render Pass"),
            depth: 1.0,
            stencil: 0,
            render_area: Rect2D {
                x: 0.0,
                y: 0.0,
                width: extent.width as f32,
                height: extent.height as f32,
            },
            clear_color: Color::new(0.65, 0.8, 0.9, 1.0),
            clear_flags: RenderPassClearFlags::COLOR_BUFFER | RenderPassClearFlags::DEPTH_BUFFER,
            attachments: &[
                RenderTargetAttachment {
                    format: ImageFormat::from(surface_format),
                    ty: RenderTargetAttachmentType::Color,
                    load_op: AttachmentLoadOp::Clear,
                    store_op: AttachmentStoreOp::Store,
                },
                RenderTargetAttachment {
                    format: ImageFormat::from(depth_texture.image().format()),
                    ty: RenderTargetAttachmentType::Depth,
                    load_op: AttachmentLoadOp::Clear,
                    store_op: AttachmentStoreOp::Store,
                },
            ],
        };
        let render_pass = unsafe { RenderPass::new(device, &render_pass_desc)? };

        let framebuffers = unsafe {
            Self::create_framebuffers(
                device,
                desc.max_frames_in_flight,
                render_pass.raw(),
                &image_views,
                &[depth_texture.raw_image_view()],
                [extent.width, extent.height],
            )?
        };
        Ok(SwapchainRelatedObjects {
            extent,
            capabilities,
            render_pass,
            framebuffers,
            depth_texture,
            swapchain_images,
            image_views,
        })
    }

    fn get_memory_type_index(
        memory_properties: vk::PhysicalDeviceMemoryProperties,
        properties: vk::MemoryPropertyFlags,
        requirements: vk::MemoryRequirements,
    ) -> u32 {
        // 我们首先找到适合缓冲区本身的内存类型
        // 来自 requirements 参数的内存类型位字段将用于指定合适的内存类型的位字段。
        // 这意味着我们可以通过简单地遍历它们并检查相应的位是否设置为 1 来找到合适的内存类型的索引。

        // 然而，我们不只是对适合顶点缓冲区的内存类型感兴趣。我们还需要能够将我们的顶点数据写入该内存中。
        // memory_types 数组由 vk::MemoryType 结构组成，指定每种类型的内存的堆和属性。属性定义了内存的特殊功能，
        // 比如能够映射它，所以我们可以从 CPU 写到它。这个属性用 vk::MemoryPropertyFlags::HOST_VISIBLE 表示，
        // 但是我们也需要使用 vk::MemoryPropertyFlags::HOST_COHERENT 属性。我们将在映射内存时看到原因。
        (0..memory_properties.memory_type_count)
            .find(|i| {
                let suitable = (requirements.memory_type_bits & (1 << i)) != 0;
                let memory_type = memory_properties.memory_types[*i as usize];
                suitable && memory_type.property_flags.contains(properties)
            })
            .expect("Failed to find suitable memory type!")
    }

    unsafe fn create_depth_objects(
        desc: &SwapchainDescriptor,
        extent: vk::Extent2D,
    ) -> Result<VulkanTexture, DeviceError> {
        let depth_image_desc = DepthImageDescriptor {
            device: &desc.context.device,
            instance: &desc.context.instance,
            allocator: desc.context.allocator.clone(),
            dimension: [extent.width, extent.height],
        };
        let depth_image = unsafe { Image::new_depth_image(&depth_image_desc)? };

        let depth_image_view = unsafe {
            ImageView::new_depth_image_view(
                Some("Depth Image View"),
                &desc.context.device,
                depth_image.raw(),
                depth_image.format(),
            )?
        };

        let texture_desc = VulkanTextureDescriptor {
            instance: &desc.context.instance,
            device: &desc.context.device,
            image: depth_image,
            image_view: depth_image_view,
            generate_mipmaps: false,
        };
        let texture = unsafe { VulkanTexture::new(texture_desc)? };

        Ok(texture)
    }

    unsafe fn create_color_objects(
        desc: &SwapchainDescriptor,
        format: vk::Format,
        extent: vk::Extent2D,
    ) -> Result<VulkanTexture, DeviceError> {
        let color_image_desc = ImageDescriptor {
            device: &desc.context.device,
            image_type: vk::ImageType::TYPE_2D,
            format,
            dimension: [extent.width, extent.height],
            mip_levels: 1,
            array_layers: 1,
            samples: desc.context.device.adapter().max_msaa_samples(),
            tiling: vk::ImageTiling::OPTIMAL,
            usage: vk::ImageUsageFlags::COLOR_ATTACHMENT
                | vk::ImageUsageFlags::TRANSIENT_ATTACHMENT,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            allocator: desc.context.allocator.clone(),
        };

        let color_image = unsafe { Image::new(&color_image_desc)? };

        let color_image_view = unsafe {
            ImageView::new_color_image_view(
                Some("Color Image View"),
                &desc.context.device,
                color_image.raw(),
                format,
                1,
            )?
        };

        let texture_desc = VulkanTextureDescriptor {
            instance: &desc.context.instance,
            device: &desc.context.device,
            image: color_image,
            image_view: color_image_view,
            generate_mipmaps: false,
        };
        let texture = unsafe { VulkanTexture::new(texture_desc)? };

        Ok(texture)
    }

    unsafe fn queue_present(
        &self,
        present_queue: vk::Queue,
        image_index: u32,
        wait_semaphores: &[vk::Semaphore],
    ) -> Result<bool, SurfaceError> {
        let swapchains = &[self.raw];
        let indices = &[image_index];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(wait_semaphores)
            .swapchains(swapchains)
            .image_indices(indices);
        match unsafe { self.loader.queue_present(present_queue, &present_info) } {
            Ok(suboptimal) => Ok(suboptimal),
            Err(error) => match error {
                vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::NOT_READY => {
                    Err(SurfaceError::OutOfDate)
                }
                vk::Result::ERROR_SURFACE_LOST_KHR => Err(SurfaceError::Lost),
                other => Err(SurfaceError::from(other).into()),
            },
        }
    }
    // unsafe fn create_framebuffer(
    //     device: &Device,
    //     map: &Mutex<fxhash::FxHashMap<FramebufferDescriptor, vk::Framebuffer>>,
    //     desc: FramebufferDescriptor,
    // ) -> Result<vk::Framebuffer, DeviceError> {
    //     use std::collections::hash_map::Entry;
    //     Ok(match map.lock().entry(desc) {
    //         Entry::Occupied(e) => *e.get(),
    //         Entry::Vacant(e) => {
    //             let desc = e.key();
    //             let create_info = vk::FramebufferCreateInfo::builder()
    //                 .render_pass(desc.render_pass)
    //                 .attachments(&desc.image_views)
    //                 .width(desc.dimensions[0])
    //                 .height(desc.dimensions[1])
    //                 .layers(1)
    //                 .build();
    //             let framebuffer = unsafe { device.raw().create_framebuffer(&create_info, None)? };
    //             e.insert(framebuffer);
    //             framebuffer
    //         }
    //     })
    // }

    unsafe fn create_framebuffers(
        device: &Arc<Device>,
        frames_in_flight: u32,
        render_pass: vk::RenderPass,
        swapchain_image_views: &[ImageView],
        other_image_views: &[vk::ImageView],
        dimensions: [u32; 2],
    ) -> Result<Vec<Framebuffer>, DeviceError> {
        let mut framebuffers = Vec::with_capacity(frames_in_flight as usize);

        for index in 0..frames_in_flight {
            let mut image_views = Vec::with_capacity(other_image_views.len() + 1);
            image_views.push(swapchain_image_views[index as usize].raw());
            for other_image_view in other_image_views {
                image_views.push(*other_image_view);
            }

            let desc = FramebufferDescriptor {
                label: Some("Swapchain Framebuffer"),
                render_pass,
                image_views: &image_views,
                render_area: Rect2D {
                    x: 0.0,
                    y: 0.0,
                    width: dimensions[0] as f32,
                    height: dimensions[1] as f32,
                },
                layers: 1,
            };
            let framebuffer = unsafe { Framebuffer::new(device, &desc)? };
            framebuffers.push(framebuffer);
        }

        Ok(framebuffers)
    }

    unsafe fn create_semaphores_and_fences(
        device: &Arc<Device>,
        frames_in_flight: u32,
    ) -> Result<(Vec<SwapchainSemaphore>, Vec<Fence>), DeviceError> {
        let mut semaphores = vec![];
        let mut in_flight_fences = vec![];
        for _ in 0..frames_in_flight {
            unsafe {
                let swapchain_semaphore = SwapchainSemaphore {
                    present: Semaphore::new(device)?,
                    render: Semaphore::new(device)?,
                };
                semaphores.push(swapchain_semaphore);
                in_flight_fences.push(Fence::new(device, Some(vk::FenceCreateFlags::SIGNALED))?);
            }
        }
        Ok((semaphores, in_flight_fences))
    }

    unsafe fn acquire_next_image(
        &self,
        timeout: u64,
        semaphore: &Semaphore,
    ) -> Result<AcquiredImage, DeviceError> {
        match unsafe {
            self.loader
                .acquire_next_image(self.raw, timeout, semaphore.raw, vk::Fence::null())
        } {
            Ok(pair) => Ok(AcquiredImage {
                image_index: pair.0,
                is_suboptimal: pair.1,
            }),
            Err(error) => match error {
                vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::NOT_READY => {
                    Err(DeviceError::Other("Out of Date"))
                }
                vk::Result::ERROR_SURFACE_LOST_KHR => Err(DeviceError::Other("Lost")),
                other => Err(DeviceError::from(other)),
            },
        }
    }

    unsafe fn create_draw_command_buffers(
        device: &Arc<Device>,
        frames_in_flight: u32,
    ) -> Result<Vec<RefCell<CommandBuffer>>, DeviceError> {
        let cbs = unsafe { device.allocate_command_buffers(true, frames_in_flight)? };
        let result = cbs.into_iter().map(|x| RefCell::new(x)).collect();
        Ok(result)
    }

    unsafe fn destroy_swapchain_recreate_objects(&mut self) {
        unsafe {
            // for framebuffer in &self.framebuffers {
            //     self.context
            //         .device
            //         .raw()
            //         .destroy_framebuffer(framebuffer.raw(), None);
            // }
            for image_view in &self.image_views {
                self.context
                    .device
                    .raw()
                    .destroy_image_view(image_view.raw(), None);
            }

            self.loader.destroy_swapchain(self.raw, None);
        }
    }

    unsafe fn destroy_swapchain_raw(&mut self) {
        unsafe {
            self.loader.destroy_swapchain(self.raw, None);
            log::debug!("Swapchain destroyed.");
        }
    }

    unsafe fn destroy_all(&mut self) {
        unsafe {
            self.destroy_swapchain_recreate_objects();
            self.destroy_swapchain_raw();
        }
    }
}

impl SwapChainSupportDetail {
    pub unsafe fn new(
        adapter: vk::PhysicalDevice,
        surface: &khr::Surface,
        surface_khr: vk::SurfaceKHR,
    ) -> Result<SwapChainSupportDetail, DeviceError> {
        let capabilities =
            surface.get_physical_device_surface_capabilities(adapter, surface_khr)?;
        let surface_formats = surface.get_physical_device_surface_formats(adapter, surface_khr)?;
        let present_modes =
            surface.get_physical_device_surface_present_modes(adapter, surface_khr)?;

        Ok(SwapChainSupportDetail {
            capabilities,
            surface_formats,
            present_modes,
        })
    }

    pub fn get_ideal_swapchain_properties(
        &self,
        preferred_dimensions: [u32; 2],
    ) -> SwapchainProperties {
        let format = Self::choose_swapchain_format(&self.surface_formats);
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
