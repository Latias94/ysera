use std::rc::Rc;
use std::time::Instant;

use ash::vk;
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};
use imgui::Context as ImguiContext;
use parking_lot::Mutex;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyboardInput, WindowEvent};
use winit::window::Window;

use math::Mat4;
use ysera_imgui::gui::GuiContext;

use crate::gui::GuiState;
use crate::vulkan::adapter::Adapter;
use crate::vulkan::command_buffer_allocator::CommandBufferAllocator;
use crate::vulkan::descriptor_set_allocator::DescriptorSetAllocator;
use crate::vulkan::imgui::{ImguiRenderer, ImguiRendererDescriptor};
use crate::vulkan::model::{Model, ModelDescriptor};
use crate::vulkan::swapchain::SwapchainDescriptor;
use crate::vulkan::texture::{VulkanTexture, VulkanTextureFromPathDescriptor};
use crate::vulkan::utils;
use crate::vulkan_v2::debug::DebugUtils;
use crate::{
    AdapterRequirements, InstanceDescriptor, QueueFamilyIndices, SurfaceError, MAX_FRAMES_IN_FLIGHT,
};

use super::device::Device;
use super::instance::Instance;
use super::surface::Surface;
use super::swapchain::Swapchain;

pub struct VulkanRenderer {
    adapter: Rc<Adapter>,
    instance: Rc<Instance>,
    surface: Rc<Surface>,
    device: Rc<Device>,
    allocator: Rc<Mutex<Allocator>>,
    swapchain: Option<Swapchain>,
    debug_utils: Option<DebugUtils>,
    present_queue: vk::Queue,
    graphics_queue: vk::Queue,
    command_pool: vk::CommandPool,
    extent: vk::Extent2D,
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    indices: QueueFamilyIndices,
    command_buffer_allocator: Rc<CommandBufferAllocator>,
    model: Rc<Model>,
    mip_levels: u32,
    frame: usize,
    instant: Instant,
    imgui_renderer: ImguiRenderer,
    gui_state: GuiState,
    misc: Misc,
}

pub struct Misc {
    test_texture: VulkanTexture,
}

impl VulkanRenderer {
    pub fn new(window: &Window, gui_context: &mut ImguiContext) -> anyhow::Result<Self> {
        let instance_desc = InstanceDescriptor::builder()
            // .flags(crate::vulkan::instance::InstanceFlags::empty())
            // .debug_level_filter(log::LevelFilter::Info)
            .build();
        let instance = unsafe { Instance::init(&instance_desc)? };
        let surface = unsafe { instance.create_surface(window)? };
        let adapters = instance.enumerate_adapters()?;
        assert!(!adapters.is_empty());

        let requirements = AdapterRequirements::builder()
            .compute(true)
            .adapter_extension_names(vec![])
            .build();
        let mut selected_adapter = None;
        for adapter in adapters {
            if unsafe { adapter.meet_requirements(instance.raw(), &surface, &requirements) }.is_ok()
            {
                selected_adapter = Some(adapter);
                break;
            }
        }
        let adapter = match selected_adapter {
            None => panic!("Cannot find the require device."),
            Some(adapter) => adapter,
        };

        let adapter = Rc::new(adapter);
        let instance = Rc::new(instance);

        log::debug!("Find the require device.");
        let debug_utils = instance.debug_utils().clone();

        let indices = utils::get_queue_family_indices(instance.raw(), adapter.raw(), &surface)?;
        indices.log_debug();

        let device =
            unsafe { adapter.open(&instance, indices, &requirements, debug_utils.clone())? };

        let allocator = Allocator::new(&AllocatorCreateDesc {
            instance: instance.raw().clone(),
            device: device.raw().clone(),
            physical_device: adapter.raw(),
            debug_settings: Default::default(),
            // check https://stackoverflow.com/questions/73341075/rust-gpu-allocator-bufferdeviceaddress-must-be-enabbled
            buffer_device_address: false,
        });

        let allocator = match allocator {
            Ok(x) => x,
            Err(e) => {
                log::error!("gpu-allocator allocator create failed!");
                panic!("{e}");
            }
        };

        // this queue should support graphics and present
        let graphics_queue = device.get_device_queue(indices.graphics_family.unwrap(), 0);
        let present_queue = device.get_device_queue(indices.present_family.unwrap(), 0);
        let device = Rc::new(device);
        let inner_size = window.inner_size();

        let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(indices.graphics_family.unwrap())
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .build();
        let command_pool = device.create_command_pool(&command_pool_create_info)?;

        let command_buffer_allocator = Rc::new(CommandBufferAllocator::new(
            &device,
            command_pool,
            graphics_queue,
        ));

        let allocator = Rc::new(Mutex::new(allocator));
        let instant = Instant::now();

        let model_desc = ModelDescriptor {
            file_name: "viking_room",
            device: &device,
            allocator: allocator.clone(),
            command_buffer_allocator: &command_buffer_allocator,
            adapter: adapter.clone(),
            instance: instance.clone(),
        };
        let model = Rc::new(Model::load_obj(&model_desc)?);
        let mip_levels = model.texture().image().get_max_mip_levels();

        let swapchain_desc = SwapchainDescriptor {
            adapter: adapter.clone(),
            surface: &surface,
            instance: instance.clone(),
            device: &device,
            max_frame_in_flight: MAX_FRAMES_IN_FLIGHT as u32,
            queue_family: indices,
            dimensions: [inner_size.width, inner_size.height],
            command_pool,
            graphics_queue,
            present_queue,
            allocator: allocator.clone(),
            command_buffer_allocator: command_buffer_allocator.clone(),
            model: model.clone(),
            old_swapchain: None,
            instant,
            mip_levels,
        };

        let swapchain = Swapchain::new(&swapchain_desc)?;

        let imgui_command_pool = {
            let create_info = vk::CommandPoolCreateInfo::builder()
                .flags(
                    vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER
                        | vk::CommandPoolCreateFlags::TRANSIENT,
                )
                .queue_family_index(indices.graphics_family.unwrap())
                .build();
            device.create_command_pool(&create_info)?
        };
        let imgui_descriptor_set_allocator = Rc::new(DescriptorSetAllocator::new(&device, 2)?);

        let mut imgui_descriptor = ImguiRendererDescriptor {
            instance: instance.clone(),
            adapter: adapter.clone(),
            device: device.clone(),
            queue: graphics_queue,
            format: swapchain.surface_format().format,
            command_pool: imgui_command_pool,
            render_pass: swapchain.imgui_render_pass().raw(),
            context: gui_context,
            descriptor_set_allocator: imgui_descriptor_set_allocator,
        };

        let mut imgui_renderer = ImguiRenderer::new(&mut imgui_descriptor)?;

        let semaphore_create_info = vk::SemaphoreCreateInfo::builder().build();
        let fence_create_info = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED)
            .build();
        let mut image_available_semaphores = vec![];
        let mut render_finished_semaphores = vec![];
        let mut in_flight_fences = vec![];
        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            image_available_semaphores.push(device.create_semaphore(&semaphore_create_info)?);
            render_finished_semaphores.push(device.create_semaphore(&semaphore_create_info)?);
            in_flight_fences.push(device.create_fence(&fence_create_info)?);
        }

        let mut texture_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        texture_path.push(format!("../../resources/textures/{}.png", "texture"));
        let texture_desc = VulkanTextureFromPathDescriptor {
            adapter: &adapter,
            instance: &instance,
            device: &device,
            allocator: allocator.clone(),
            command_buffer_allocator: &command_buffer_allocator,
            path: &texture_path,
            format: vk::Format::R8G8B8A8_UNORM,
            enable_mip_levels: false,
        };

        let test_texture = VulkanTexture::new_from_path(texture_desc)?;
        let test_texture_id =
            imgui_renderer.add_texture(&test_texture, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)?;

        Ok(Self {
            adapter,
            instance,
            surface: Rc::new(surface),
            device,
            allocator,
            extent: swapchain.extent(),
            swapchain: Some(swapchain),
            debug_utils,
            present_queue,
            graphics_queue,
            command_pool,
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            indices,
            command_buffer_allocator,
            model,
            mip_levels,
            frame: 0,
            instant,
            imgui_renderer,
            gui_state: GuiState::new(Some(test_texture_id)),
            misc: Misc { test_texture },
        })
    }

    pub fn render(&mut self, window: &Window, gui_context: &mut GuiContext) -> anyhow::Result<()> {
        if self.swapchain.is_none() {
            self.recreate_swapchain(PhysicalSize {
                width: self.extent.width,
                height: self.extent.height,
            })?;
        }

        let in_flight_fence = self.in_flight_fences[self.frame];
        let in_flight_fences = [in_flight_fence];
        self.device
            .wait_for_fence(&in_flight_fences, true, u64::MAX)?;

        let swapchain = self.swapchain.as_mut().unwrap();
        let result =
            swapchain.acquire_next_image(u64::MAX, self.image_available_semaphores[self.frame]);
        let image_index = match result {
            Ok((image_index, _)) => image_index,
            Err(SurfaceError::OutOfDate) => {
                self.swapchain = None;
                return Ok(());
            }
            Err(e) => panic!("failed to acquire_next_image. Err: {}", e),
        };
        self.device.reset_fence(&in_flight_fences)?;

        let command_buffer = swapchain.render(
            image_index as usize,
            window,
            gui_context,
            self.imgui_renderer.renderer_mut(),
            &mut self.gui_state,
            crate::gui::draw_imgui,
        )?;

        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

        let wait_semaphores = &[self.image_available_semaphores[self.frame]];
        let signal_semaphores = &[self.render_finished_semaphores[self.frame]];

        let command_buffers = [command_buffer];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(signal_semaphores)
            .build();

        self.device
            .queue_submit(self.graphics_queue, &[submit_info], in_flight_fence)?;
        swapchain.update_submitted_command_buffer(self.frame);

        let swapchains = [swapchain.raw()];
        let image_indices = [image_index];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        match swapchain.queue_present(&present_info) {
            Ok(suboptimal) => suboptimal,
            Err(SurfaceError::OutOfDate) => {
                self.swapchain = None;
                return Ok(());
            }
            Err(e) => panic!("failed to acquire_next_image. Err: {}", e),
        };
        self.frame = (self.frame + 1) % MAX_FRAMES_IN_FLIGHT;
        Ok(())
    }

    pub fn recreate_swapchain(&mut self, inner_size: PhysicalSize<u32>) -> anyhow::Result<()> {
        self.device.wait_idle();
        log::debug!("======== Swapchain start recreate.========");

        let mut old_swapchain = None;
        if let Some(swapchain) = &self.swapchain {
            old_swapchain = Some(swapchain.raw())
        }
        let swapchain_desc = SwapchainDescriptor {
            adapter: self.adapter.clone(),
            surface: &self.surface,
            instance: self.instance.clone(),
            device: &self.device,
            max_frame_in_flight: MAX_FRAMES_IN_FLIGHT as u32,
            queue_family: self.indices,
            dimensions: [inner_size.width, inner_size.height],
            command_pool: self.command_pool,
            graphics_queue: self.graphics_queue,
            present_queue: self.present_queue,
            allocator: self.allocator.clone(),
            command_buffer_allocator: self.command_buffer_allocator.clone(),
            model: self.model.clone(),
            mip_levels: self.mip_levels,
            old_swapchain,
            instant: self.instant,
        };

        let swapchain = Swapchain::new(&swapchain_desc)?;
        self.swapchain = Some(swapchain);
        self.extent = vk::Extent2D {
            width: inner_size.width,
            height: inner_size.height,
        };
        log::debug!("======== Swapchain recreated.========");
        Ok(())
    }

    pub fn handle_event(&mut self, window: &Window, event: &Event<()>) {
        match *event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(key),
                            ..
                        },
                    ..
                } => {
                    // log::info!("press {:?}", key);
                }
                _ => {}
            },
            _ => (),
        }
    }
}

impl VulkanRenderer {
    pub fn set_view(&mut self, view: Mat4) {
        if self.swapchain.is_some() {
            self.swapchain
                .as_mut()
                .unwrap()
                .set_render_system_state(view);
        }
    }
}

impl Drop for VulkanRenderer {
    fn drop(&mut self) {
        self.device.wait_idle();
        self.swapchain = None; // drop first
        self.image_available_semaphores
            .iter()
            .for_each(|s| self.device.destroy_semaphore(*s));
        self.render_finished_semaphores
            .iter()
            .for_each(|s| self.device.destroy_semaphore(*s));
        self.in_flight_fences
            .iter()
            .for_each(|s| self.device.destroy_fence(*s));
        self.device.destroy_command_pool(self.command_pool);
        if let Some(DebugUtils {
            extension,
            messenger,
        }) = self.debug_utils.take()
        {
            unsafe {
                extension.destroy_debug_utils_messenger(messenger, None);
            }
        }

        unsafe {
            self.surface
                .loader()
                .destroy_surface(self.surface.raw(), None);
        }
        log::debug!("surface destroyed.");
    }
}
