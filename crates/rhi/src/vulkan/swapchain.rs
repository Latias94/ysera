use std::rc::Rc;
use std::time::Instant;

use ash::extensions::khr;
use ash::vk;
use gpu_allocator::vulkan::Allocator;
use imgui_rs_vulkan_renderer::Renderer as GuiRenderer;
use parking_lot::Mutex;
use typed_builder::TypedBuilder;
use winit::window::Window;

use eureka_imgui::gui::GuiContext;
use math::prelude::*;

use crate::gui::GuiState;
use crate::vulkan::adapter::Adapter;
use crate::vulkan::buffer::{Buffer, BufferType, StagingBufferDescriptor, UniformBufferDescriptor};
use crate::vulkan::command_buffer::{CommandBuffer, CommandBufferState};
use crate::vulkan::command_buffer_allocator::CommandBufferAllocator;
use crate::vulkan::conv;
use crate::vulkan::descriptor_set_allocator::{
    DescriptorSetAllocator, PerFrameDescriptorSetsCreateInfo,
};
use crate::vulkan::device::Device;
use crate::vulkan::image::{DepthImageDescriptor, Image, ImageDescriptor};
use crate::vulkan::image_view::ImageView;
use crate::vulkan::instance::Instance;
use crate::vulkan::model::Model;
use crate::vulkan::pipeline::Pipeline;
use crate::vulkan::render_pass::{ImguiRenderPassDescriptor, RenderPass, RenderPassDescriptor};
use crate::vulkan::shader::{Shader, ShaderDescriptor};
use crate::vulkan::surface::Surface;
use crate::vulkan::texture::{VulkanTexture, VulkanTextureDescriptor};
use crate::vulkan::uniform_buffer::UniformBufferObject;
use crate::{Color, DeviceError, QueueFamilyIndices, SurfaceError};

pub struct Swapchain {
    raw: vk::SwapchainKHR,
    loader: khr::Swapchain,
    adapter: Rc<Adapter>,
    instance: Rc<Instance>,
    device: Rc<Device>,
    family_index: QueueFamilyIndices,
    swapchain_images: Vec<vk::Image>,
    image_views: Vec<ImageView>,
    surface_format: vk::SurfaceFormatKHR,
    depth_format: vk::Format,
    extent: vk::Extent2D,
    capabilities: vk::SurfaceCapabilitiesKHR,
    render_pass: RenderPass,
    imgui_render_pass: RenderPass,
    pipeline: Pipeline,
    command_buffers: Vec<CommandBuffer>,
    framebuffers: Vec<vk::Framebuffer>,
    imgui_framebuffers: Vec<vk::Framebuffer>,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    command_buffer_allocator: Rc<CommandBufferAllocator>,
    descriptor_set_allocator: Rc<DescriptorSetAllocator>,
    depth_texture: VulkanTexture,
    color_texture: VulkanTexture,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    uniform_buffers: Vec<Buffer>,
    per_frame_descriptor_sets: Vec<vk::DescriptorSet>,
    model: Rc<Model>,
    mip_levels: u32,
    instant: Instant,
    render_system_state: RenderSystemState,
}

pub struct RenderSystemState {
    projection: Mat4,
    view: Mat4,
    near_clip: f32,
    far_clip: f32,
}

#[derive(Clone, Copy, Debug)]
struct SwapchainProperties {
    pub surface_format: vk::SurfaceFormatKHR,
    pub present_mode: vk::PresentModeKHR,
    pub extent: vk::Extent2D,
}

struct SwapChainSupportDetail {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub surface_formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

#[derive(TypedBuilder)]
pub struct SwapchainDescriptor<'a> {
    pub adapter: Rc<Adapter>,
    pub surface: &'a Surface,
    pub instance: Rc<Instance>,
    pub device: &'a Rc<Device>,
    pub max_frame_in_flight: u32,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
    pub queue_family: QueueFamilyIndices,
    pub dimensions: [u32; 2],
    pub command_pool: vk::CommandPool,
    pub allocator: Rc<Mutex<Allocator>>,
    pub command_buffer_allocator: Rc<CommandBufferAllocator>,
    pub old_swapchain: Option<vk::SwapchainKHR>,
    pub model: Rc<Model>,
    pub mip_levels: u32,
    pub instant: Instant,
}

#[derive(Clone, TypedBuilder, Hash, PartialEq, Eq)]
pub struct FramebufferDescriptor {
    render_pass: vk::RenderPass,
    texture_views: Vec<vk::ImageView>,
    swapchain_extent: vk::Extent2D,
}

impl Swapchain {
    pub fn raw(&self) -> vk::SwapchainKHR {
        self.raw
    }

    pub fn loader(&self) -> &khr::Swapchain {
        &self.loader
    }

    pub fn surface_format(&self) -> vk::SurfaceFormatKHR {
        self.surface_format
    }

    pub fn extent(&self) -> vk::Extent2D {
        self.extent
    }

    pub fn color_texture(&self) -> &VulkanTexture {
        &self.color_texture
    }

    pub fn render_pass(&self) -> &RenderPass {
        &self.render_pass
    }

    pub fn imgui_render_pass(&self) -> &RenderPass {
        &self.imgui_render_pass
    }

    pub fn pipeline(&self) -> &Pipeline {
        &self.pipeline
    }

    pub fn command_buffer_allocator(&self) -> &CommandBufferAllocator {
        &self.command_buffer_allocator
    }

    pub fn new(desc: &SwapchainDescriptor) -> anyhow::Result<Self> {
        let device = desc.device;
        let (swapchain_loader, swapchain, properties, support, image_count) =
            Self::create_swapchain(&desc)?;
        let extent = properties.extent;
        // 交换链图像由交换链自己负责创建，并在交换链清除时自动被清除，不需要我们自己进行创建和清除操作。
        let swapchain_images = unsafe { swapchain_loader.get_swapchain_images(swapchain)? };

        let mut capabilities = support.capabilities;

        capabilities.current_extent.width = capabilities.current_extent.width.max(1);
        capabilities.current_extent.height = capabilities.current_extent.height.max(1);
        let swapchain_image_views = swapchain_images
            .iter()
            .map(|i| {
                ImageView::new_color_image_view(
                    Some("swapchain image view"),
                    device,
                    *i,
                    properties.surface_format.format,
                    1,
                )
            })
            .collect::<Result<Vec<ImageView>, DeviceError>>()?;

        // let memory_properties = unsafe {
        //     desc.instance
        //         .raw()
        //         .get_physical_device_memory_properties(desc.adapter.raw())
        // };

        let color_format = properties.surface_format.format;
        let color_texture = Self::create_color_objects(desc, color_format, extent)?;

        let depth_texture = Self::create_depth_objects(desc, extent)?;
        let depth_format = depth_texture.image().format();

        let clear_color = Color::new(0.65, 0.8, 0.9, 1.0);
        let rect2d = Rect2D {
            x: 0.0,
            y: 0.0,
            width: extent.width as f32,
            height: extent.height as f32,
        };

        let map = Default::default();

        let render_pass_desc = RenderPassDescriptor {
            device,
            surface_format: color_format,
            depth_format,
            render_area: rect2d,
            clear_color,
            max_msaa_samples: desc.adapter.max_msaa_samples(),
            depth: 1.0,
            stencil: 0,
        };
        let render_pass = RenderPass::new(&render_pass_desc)?;

        let framebuffers = swapchain_image_views
            .iter()
            .map(|i| {
                let image_view = i.raw();
                let framebuffer_desc = FramebufferDescriptor::builder()
                    .texture_views(vec![
                        color_texture.image_view().raw(),
                        depth_texture.image_view().raw(),
                        image_view,
                    ])
                    .swapchain_extent(extent)
                    .render_pass(render_pass.raw())
                    .build();
                Self::create_framebuffer(device, &map, framebuffer_desc)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let imgui_render_pass_desc = ImguiRenderPassDescriptor {
            device,
            surface_format: properties.surface_format.format,
            render_area: rect2d,
        };
        let imgui_render_pass = RenderPass::new_imgui_render_pass(&imgui_render_pass_desc)?;

        let imgui_framebuffers = swapchain_image_views
            .iter()
            .map(|i| {
                let image_view = i.raw();
                let framebuffer_desc = FramebufferDescriptor::builder()
                    .texture_views(vec![image_view])
                    .swapchain_extent(extent)
                    .render_pass(imgui_render_pass.raw())
                    .build();
                Self::create_framebuffer(device, &map, framebuffer_desc)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let vert_shader_desc = ShaderDescriptor {
            label: Some("Triangle Vert"),
            device,
            spv_bytes: &Shader::load_pre_compiled_spv_bytes_from_name(
                "triangle_push_constant.vert",
            ),
            entry_name: "main",
        };
        let vert_shader = Shader::new_vert(&vert_shader_desc)?;
        let frag_shader_desc = ShaderDescriptor {
            label: Some("Triangle Frag"),
            device,
            spv_bytes: &Shader::load_pre_compiled_spv_bytes_from_name(
                "triangle_uniform.frag",
            ),
            entry_name: "main",
        };
        let frag_shader = Shader::new_frag(&frag_shader_desc)?;

        let vertex_buffer_desc = StagingBufferDescriptor {
            label: Some("Vertex Buffer"),
            device,
            allocator: desc.allocator.clone(),
            elements: desc.model.vertices(),
            command_buffer_allocator: &desc.command_buffer_allocator,
        };
        let vertex_buffer =
            Buffer::new_buffer_copy_from_staging_buffer(&vertex_buffer_desc, BufferType::Vertex)?;

        let index_buffer_desc = StagingBufferDescriptor {
            label: Some("Index Buffer"),
            device,
            allocator: desc.allocator.clone(),
            elements: desc.model.indices(),
            command_buffer_allocator: &desc.command_buffer_allocator,
        };
        let index_buffer =
            Buffer::new_buffer_copy_from_staging_buffer(&index_buffer_desc, BufferType::Index)?;

        let uniform_buffer_desc = UniformBufferDescriptor {
            label: Some("Uniform Buffer"),
            device,
            allocator: desc.allocator.clone(),
            elements: &[Default::default()] as &[UniformBufferObject],
            buffer_type: BufferType::Uniform,
            command_buffer_allocator: &desc.command_buffer_allocator,
        };
        let uniform_buffers = swapchain_image_views
            .iter()
            .map(|_| Buffer::new_uniform_buffer(&uniform_buffer_desc))
            .collect::<Result<Vec<_>, _>>()?;

        let descriptor_set_allocator = Rc::new(DescriptorSetAllocator::new(device, image_count)?);

        let descriptor_set_layouts = &[
            descriptor_set_allocator.raw_per_frame_layout(),
            descriptor_set_allocator.raw_texture_layout(),
        ];

        let shaders = &[vert_shader, frag_shader];
        let pipeline = Pipeline::new(
            device,
            render_pass.raw(),
            desc.adapter.max_msaa_samples(),
            descriptor_set_layouts,
            shaders,
        )?;

        let command_buffers = desc
            .command_buffer_allocator
            .allocate_command_buffers(true, swapchain_image_views.len() as u32)?;

        let model_texture = desc.model.texture();
        let descriptor_sets_create_info = PerFrameDescriptorSetsCreateInfo {
            uniform_buffers: &uniform_buffers,
            texture_image_view: model_texture.raw_image_view(),
            texture_sampler: model_texture.raw_sampler(),
        };

        let per_frame_descriptor_sets = descriptor_set_allocator
            .allocate_per_frame_descriptor_sets(&descriptor_sets_create_info)?;

        // init renderer state
        let near_clip = 0.1f32;
        let far_clip = 1000.0f32;
        let view = math::look_at(
            &vec3(2.0, 2.0, 2.0),
            &vec3(0.0, 0.0, 0.0),
            &vec3(0.0, 0.0, 1.0),
        );
        let projection = math::perspective_rh_zo(
            extent.width as f32 / extent.height as f32,
            math::radians(&math::vec1(90.0))[0],
            // math::radians(&math::vec1(ui_state.fovy))[0],
            near_clip,
            far_clip,
        );

        let render_system_state = RenderSystemState {
            projection,
            view,
            near_clip,
            far_clip,
        };

        let swapchain = Self {
            raw: swapchain,
            loader: swapchain_loader,
            adapter: desc.adapter.clone(),
            instance: desc.instance.clone(),
            device: desc.device.clone(),
            family_index: desc.queue_family,
            swapchain_images,
            surface_format: properties.surface_format,
            depth_format,
            extent: properties.extent,
            capabilities,
            image_views: swapchain_image_views,
            framebuffers,
            render_pass,
            imgui_framebuffers,
            imgui_render_pass,
            pipeline,
            command_buffers,
            graphics_queue: desc.graphics_queue,
            present_queue: desc.present_queue,
            command_buffer_allocator: desc.command_buffer_allocator.clone(),
            descriptor_set_allocator,
            depth_texture,
            color_texture,
            vertex_buffer,
            index_buffer,
            uniform_buffers,
            per_frame_descriptor_sets,
            model: desc.model.clone(),
            mip_levels: desc.mip_levels,
            instant: desc.instant,
            render_system_state,
        };

        Ok(swapchain)
    }

    pub fn render(
        &mut self,
        image_index: usize,
        window: &Window,
        gui_context: &mut GuiContext,
        gui_renderer: &mut GuiRenderer,
        ui_state: &mut GuiState,
        ui_func: impl FnOnce(&mut GuiState, &mut imgui::Ui),
    ) -> Result<vk::CommandBuffer, DeviceError> {
        self.update_uniform_buffer(image_index);

        let command_buffer = self.update_command_buffers(
            image_index,
            window,
            gui_context,
            gui_renderer,
            ui_state,
            ui_func,
        )?;

        Ok(command_buffer.raw())
    }

    fn update_command_buffers(
        &mut self,
        image_index: usize,
        window: &Window,
        gui_context: &mut GuiContext,
        gui_renderer: &mut GuiRenderer,
        ui_state: &mut GuiState,
        ui_func: impl FnOnce(&mut GuiState, &mut imgui::Ui),
    ) -> Result<&CommandBuffer, DeviceError> {
        let command_buffer = &self.command_buffers[image_index];

        self.device
            .reset_command_buffer(command_buffer.raw(), vk::CommandBufferResetFlags::empty())?;
        self.device.begin_command_buffer(
            command_buffer.raw(),
            &vk::CommandBufferBeginInfo::builder()
                // since we are now only submitting our command buffers once before resetting them,
                // we can add ONE_TIME_SUBMIT for better optimize by Vulkan driver
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
                .build(),
        )?;

        let framebuffer = self.framebuffers[image_index];
        self.render_pass.begin(command_buffer, framebuffer);

        self.device.cmd_bind_pipeline(
            command_buffer.raw(),
            vk::PipelineBindPoint::GRAPHICS,
            self.pipeline.raw(),
        );

        // 改为左手坐标系 NDC
        let viewport_rect2d = Rect2D {
            x: 0f32,
            y: self.extent.height as f32,
            width: self.extent.width as f32,
            height: -(self.extent.height as f32),
        };
        self.device
            .cmd_set_viewport(command_buffer.raw(), viewport_rect2d);

        let scissor_rect2d = Rect2D {
            x: 0.0,
            y: 0.0,
            width: self.extent.width as f32,
            height: self.extent.height as f32,
        };
        self.device.cmd_set_scissor(
            command_buffer.raw(),
            0,
            &[conv::convert_rect2d(scissor_rect2d)],
        );

        self.device.cmd_bind_vertex_buffers(
            command_buffer.raw(),
            0,
            &[self.vertex_buffer.raw()],
            &[0],
        );

        self.device.cmd_bind_index_buffer(
            command_buffer.raw(),
            self.index_buffer.raw(),
            0,
            vk::IndexType::UINT32, // Model.indices
        );

        self.device.cmd_bind_descriptor_sets(
            command_buffer.raw(),
            vk::PipelineBindPoint::GRAPHICS,
            self.pipeline.raw_pipeline_layout(),
            0,
            &[self.per_frame_descriptor_sets[image_index]],
            &[],
        );

        let time = self.instant.elapsed().as_secs_f32();
        let model = math::rotate(
            &math::identity(),
            // time *  math::radians(&math::vec1(90.0))[0],
            math::radians(&math::vec1(90.0f32))[0],
            &vec3(0.0, 0.0, 1.0),
        );

        let (_, model_bytes, _) = unsafe { model.as_slice().align_to::<u8>() };

        self.device.cmd_push_constants(
            command_buffer.raw(),
            self.pipeline.raw_pipeline_layout(),
            vk::ShaderStageFlags::VERTEX,
            0,
            model_bytes,
        );

        // self.device.cmd_push_constants(
        //     command_buffer.raw(),
        //     self.pipeline.raw_pipeline_layout(),
        //     vk::ShaderStageFlags::FRAGMENT,
        //     64,
        //     // &0.75f32.to_ne_bytes()[..],
        //     &1f32.to_ne_bytes()[..],
        // );

        self.device.cmd_draw_indexed(
            command_buffer.raw(),
            self.model.indices().len() as u32,
            1,
            0,
            0,
            0,
        );

        self.render_pass.end(command_buffer);

        self.imgui_render_pass
            .begin(command_buffer, self.imgui_framebuffers[image_index]);

        let draw_data = gui_context.render(window, ui_state, ui_func);
        gui_renderer
            .cmd_draw(command_buffer.raw(), draw_data)
            .unwrap();

        self.imgui_render_pass.end(command_buffer);

        self.device.end_command_buffer(command_buffer.raw())?;
        Ok(command_buffer)
    }

    fn update_uniform_buffer(&mut self, image_index: usize) {
        // projection[(1, 1)] *= -1.0; // openGL clip space y 和 vulkan 相反，不过我们在 cmd_set_viewport 处理了
        let ubo = UniformBufferObject {
            view: self.render_system_state.view,
            projection: self.render_system_state.projection,
        };

        let uniform_buffer = &mut self.uniform_buffers[image_index];
        uniform_buffer.copy_memory(&[ubo]);
    }

    pub fn set_render_system_state(&mut self, view: Mat4) {
        self.render_system_state.view = view;
    }

    pub fn update_submitted_command_buffer(&mut self, command_buffer_index: usize) {
        let command_buffer = &mut self.command_buffers[command_buffer_index];
        command_buffer.set_state(CommandBufferState::Submitted);
    }

    fn create_swapchain(
        desc: &SwapchainDescriptor,
    ) -> Result<
        (
            khr::Swapchain,
            vk::SwapchainKHR,
            SwapchainProperties,
            SwapChainSupportDetail,
            u32,
        ),
        DeviceError,
    > {
        profiling::scope!("create_swapchain");

        let swapchain_support = unsafe {
            SwapChainSupportDetail::new(
                desc.adapter.raw(),
                desc.surface.loader(),
                desc.surface.raw(),
            )
        }?;
        let properties = swapchain_support.get_ideal_swapchain_properties(desc.dimensions);
        let SwapchainProperties {
            surface_format,
            present_mode,
            extent,
        } = properties;

        let mut image_count = swapchain_support.capabilities.min_image_count + 1;
        image_count = image_count.max(desc.max_frame_in_flight);
        image_count = if swapchain_support.capabilities.max_image_count > 0 {
            image_count.min(swapchain_support.capabilities.max_image_count)
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
            .surface(desc.surface.raw())
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
            .old_swapchain(old_swapchain);

        let swapchain_loader = khr::Swapchain::new(desc.instance.raw(), desc.device.raw());
        let swapchain = unsafe { swapchain_loader.create_swapchain(&create_info, None)? };
        log::debug!("Vulkan swapchain created. min_image_count: {}", image_count);

        Ok((
            swapchain_loader,
            swapchain,
            properties,
            swapchain_support,
            image_count,
        ))
    }

    pub fn create_framebuffer(
        device: &Device,
        map: &Mutex<fxhash::FxHashMap<FramebufferDescriptor, vk::Framebuffer>>,
        desc: FramebufferDescriptor,
    ) -> Result<vk::Framebuffer, DeviceError> {
        use std::collections::hash_map::Entry;
        Ok(match map.lock().entry(desc) {
            Entry::Occupied(e) => *e.get(),
            Entry::Vacant(e) => {
                let desc = e.key();
                let create_info = vk::FramebufferCreateInfo::builder()
                    .render_pass(desc.render_pass)
                    .attachments(&desc.texture_views)
                    .width(desc.swapchain_extent.width)
                    .height(desc.swapchain_extent.height)
                    .layers(1)
                    .build();
                device.create_framebuffer(&create_info)?
            }
        })
    }

    pub fn acquire_next_image(
        &self,
        timeout: u64,
        semaphore: vk::Semaphore,
    ) -> Result<(u32, bool), SurfaceError> {
        match unsafe {
            self.loader
                .acquire_next_image(self.raw, timeout, semaphore, vk::Fence::null())
        } {
            Ok(pair) => Ok(pair),
            Err(error) => match error {
                vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::NOT_READY => {
                    Err(SurfaceError::OutOfDate)
                }
                vk::Result::ERROR_SURFACE_LOST_KHR => Err(SurfaceError::Lost),
                other => Err(DeviceError::from(other).into()),
            },
        }
    }

    pub fn queue_present(&self, present_info: &vk::PresentInfoKHR) -> Result<bool, SurfaceError> {
        match unsafe { self.loader.queue_present(self.present_queue, present_info) } {
            Ok(suboptimal) => Ok(suboptimal),
            Err(error) => match error {
                vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::NOT_READY => {
                    Err(SurfaceError::OutOfDate)
                }
                vk::Result::ERROR_SURFACE_LOST_KHR => Err(SurfaceError::Lost),
                other => Err(DeviceError::from(other).into()),
            },
        }
    }

    pub fn get_memory_type_index(
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

    fn create_depth_objects(
        desc: &SwapchainDescriptor,
        extent: vk::Extent2D,
    ) -> Result<VulkanTexture, DeviceError> {
        let depth_image_desc = DepthImageDescriptor {
            device: desc.device,
            instance: &desc.instance,
            adapter: &desc.adapter,
            allocator: desc.allocator.clone(),
            width: extent.width,
            height: extent.height,
            command_buffer_allocator: &desc.command_buffer_allocator,
        };
        let depth_image = Image::new_depth_image(&depth_image_desc)?;

        let depth_image_view = ImageView::new_depth_image_view(
            Some("Depth Image View"),
            desc.device,
            depth_image.raw(),
            depth_image.format(),
        )?;

        let texture_desc = VulkanTextureDescriptor {
            adapter: &desc.adapter,
            instance: &desc.instance,
            device: desc.device,
            command_buffer_allocator: &desc.command_buffer_allocator,
            image: depth_image,
            image_view: depth_image_view,
            generate_mipmaps: false,
        };
        let texture = VulkanTexture::new(texture_desc)?;

        Ok(texture)
    }

    fn create_color_objects(
        desc: &SwapchainDescriptor,
        format: vk::Format,
        extent: vk::Extent2D,
    ) -> Result<VulkanTexture, DeviceError> {
        let color_image_desc = ImageDescriptor {
            device: desc.device,
            image_type: vk::ImageType::TYPE_2D,
            format,
            dimension: [extent.width, extent.height],
            mip_levels: 1,
            array_layers: 1,
            samples: desc.adapter.max_msaa_samples(),
            tiling: vk::ImageTiling::OPTIMAL,
            usage: vk::ImageUsageFlags::COLOR_ATTACHMENT
                | vk::ImageUsageFlags::TRANSIENT_ATTACHMENT,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            allocator: desc.allocator.clone(),
        };

        let color_image = Image::new(&color_image_desc)?;

        let color_image_view = ImageView::new_color_image_view(
            Some("Color Image View"),
            desc.device,
            color_image.raw(),
            format,
            1,
        )?;

        let texture_desc = VulkanTextureDescriptor {
            adapter: &desc.adapter,
            instance: &desc.instance,
            device: desc.device,
            command_buffer_allocator: &desc.command_buffer_allocator,
            image: color_image,
            image_view: color_image_view,
            generate_mipmaps: false,
        };
        let texture = VulkanTexture::new(texture_desc)?;

        Ok(texture)
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

impl Drop for Swapchain {
    fn drop(&mut self) {
        log::debug!("Swapchain start destroy!");
        self.framebuffers
            .iter()
            .for_each(|e| self.device.destroy_framebuffer(*e));

        unsafe {
            self.loader.destroy_swapchain(self.raw, None);
        }
        log::debug!("Swapchain destroyed.");
    }
}
