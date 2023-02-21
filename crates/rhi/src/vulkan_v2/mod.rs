pub mod conv;
pub mod debug;
pub mod platforms;
pub mod utils;

use crate::types_v2::{
    DeviceFeatures, DeviceRequirement, PhysicalDeviceRequirements, QueueFamilyIndices,
    RHICommandBufferLevel, RHICommandPoolCreateInfo, RHIFormat, RHIRenderPassCreateInfo,
};
use crate::types_v2::{
    InstanceDescriptor, InstanceFlags, RHIExtent2D, RHIOffset2D, RHIRect2D, RHIViewport,
};
use crate::utils::c_char_to_string;
use crate::vulkan::command_buffer::CommandBuffer;
use crate::vulkan_v2::debug::DebugUtils;
use crate::{CommandBufferAllocateInfo, InitInfo, RHIError};
use ash::extensions::{ext, khr};
use ash::vk;
use gpu_allocator::vulkan::{Allocation, Allocator, AllocatorCreateDesc};
use log::LevelFilter;
use parking_lot::Mutex;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::collections::HashSet;
use std::ffi::{c_char, CStr, CString};
use winit::window::Window as WinitWindow;

pub struct VulkanRHI {
    viewport: RHIViewport,
    scissor: RHIRect2D,

    /// Loads instance level functions. Needs to outlive the Devices it has created.
    instance: ash::Instance,
    /// Loads the Vulkan library. Needs to outlive Instance and Device.
    entry: ash::Entry,
    debug_utils: Option<DebugUtils>,

    // surface
    surface: vk::SurfaceKHR,
    surface_loader: khr::Surface,

    // physical_device
    physical_device_requirement: PhysicalDeviceRequirements,
    physical_device: vk::PhysicalDevice,
    max_msaa_samples: vk::SampleCountFlags,
    physical_device_properties: vk::PhysicalDeviceProperties,
    physical_device_features: vk::PhysicalDeviceFeatures,

    // device
    device_features: DeviceFeatures,
    queue_family_indices: QueueFamilyIndices,
    device_requirement: DeviceRequirement,
    device: ash::Device,
    allocator: Mutex<Allocator>,

    // queue
    graphics_queue: VulkanQueue,
    present_queue: VulkanQueue,
    compute_queue: VulkanQueue,
    depth_image_format: RHIFormat,
    command_pool: VulkanCommandPool,
    command_pools: Vec<VulkanCommandPool>,
    current_command_buffer: VulkanCommandBuffer,
    command_buffers: Vec<VulkanCommandBuffer>,
    max_material_count: u32,
    descriptor_pool: VulkanDescriptorPool,
    image_available_for_render_semaphores: Vec<VulkanSemaphore>,
    image_finished_for_presentation_semaphores: Vec<VulkanSemaphore>,
    image_available_for_textures_copy_semaphores: Vec<VulkanSemaphore>,
    frame_in_flight_fences: Vec<VulkanFence>,

    // swapchain
    swapchain: vk::SwapchainKHR,
    swapchain_loader: khr::Swapchain,
    swapchain_images: Vec<vk::Image>,
    surface_format: RHIFormat,
    swapchain_extent: RHIExtent2D,
    swapchain_image_views: Vec<VulkanImageView>,

    depth_image: VulkanImage,
    depth_image_allocation: Option<Allocation>,
    depth_image_view: VulkanImageView,
    current_frame_index: usize,
}

const MAX_FRAME_IN_FLIGHT: u8 = 3;

pub struct VulkanCommandPool {
    raw: vk::CommandPool,
}

#[derive(Copy, Clone)]
pub struct VulkanCommandBuffer {
    raw: vk::CommandBuffer,
}

pub struct VulkanQueue {
    raw: vk::Queue,
}

pub struct VulkanDescriptorPool {
    raw: vk::DescriptorPool,
}

pub struct VulkanImage {
    raw: vk::Image,
}

pub struct VulkanImageView {
    raw: vk::ImageView,
}

pub struct VulkanFence {
    raw: vk::Fence,
}

pub struct VulkanSemaphore {
    raw: vk::Semaphore,
}

pub struct VulkanRenderPass {
    raw: vk::RenderPass,
}

impl crate::RHI for VulkanRHI {
    type CommandPool = VulkanCommandPool;
    type CommandBuffer = VulkanCommandBuffer;
    type RenderPass = VulkanRenderPass;

    unsafe fn initialize(init_info: InitInfo) -> Result<Self, RHIError> {
        let viewport = RHIViewport {
            x: 0.0,
            y: 0.0,
            width: init_info.window_size.width as f32,
            height: init_info.window_size.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };

        let mut scissor = RHIRect2D {
            offset: RHIOffset2D { x: 0, y: 0 },
            extent: init_info.window_size,
        };

        let instance_desc = InstanceDescriptor::builder().build();
        let (entry, instance, debug_utils) = unsafe { VulkanRHI::create_instance(&instance_desc)? };

        let (surface, surface_loader) = unsafe {
            VulkanRHI::create_surface(
                &entry,
                &instance,
                init_info.window_handle,
                init_info.display_handle,
            )?
        };

        let physical_device_requirement = PhysicalDeviceRequirements::builder().build();
        let (
            physical_device,
            max_msaa_samples,
            physical_device_properties,
            physical_device_features,
            device_features,
            queue_family_indices,
        ) = unsafe {
            VulkanRHI::initialize_physical_device(
                &instance,
                &surface_loader,
                surface,
                &physical_device_requirement,
            )?
        };
        let required_extensions = vec!["VK_KHR_swapchain".to_string()];
        let device_requirement = DeviceRequirement::builder()
            .required_extension(required_extensions)
            .build();
        let (device, graphics_queue, present_queue, compute_queue, depth_image_format) = unsafe {
            VulkanRHI::create_logical_device(
                &instance,
                physical_device,
                &physical_device_requirement,
                &device_requirement,
                &queue_family_indices,
                &instance_desc.flags,
            )?
        };

        let allocator = VulkanRHI::create_allocator(&device, &instance, physical_device)?;

        let (command_pool, command_pools) = unsafe {
            VulkanRHI::create_command_pool(&device, &queue_family_indices, MAX_FRAME_IN_FLIGHT)?
        };

        let command_buffers = unsafe {
            VulkanRHI::create_command_buffers(&device, command_pool.raw, MAX_FRAME_IN_FLIGHT)?
        };

        let max_material_count = 256;

        let descriptor_pool = unsafe {
            VulkanRHI::create_descriptor_pool(&device, max_material_count, MAX_FRAME_IN_FLIGHT)?
        };

        let (
            image_available_for_render_semaphores,
            image_finished_for_presentation_semaphores,
            image_available_for_textures_copy_semaphores,
            frame_in_flight_fences,
        ) = unsafe { VulkanRHI::create_sync_objects(&device, MAX_FRAME_IN_FLIGHT)? };

        let (swapchain, swapchain_loader, swapchain_images, surface_format, swapchain_extent) = unsafe {
            VulkanRHI::create_swapchain(
                &device,
                &instance,
                &surface_loader,
                surface,
                physical_device,
                queue_family_indices,
                init_info.window_size,
                MAX_FRAME_IN_FLIGHT,
            )?
        };

        scissor = RHIRect2D {
            offset: RHIOffset2D { x: 0, y: 0 },
            extent: RHIExtent2D {
                width: swapchain_extent.width,
                height: swapchain_extent.height,
            },
        };

        let swapchain_image_views = unsafe {
            VulkanRHI::create_swapchain_image_views(&device, &swapchain_images, surface_format)?
        };

        let (depth_image, depth_image_allocation, depth_image_view) = unsafe {
            VulkanRHI::create_framebuffer_images_and_image_views(
                &device,
                &allocator,
                depth_image_format,
                swapchain_extent,
            )?
        };

        Ok(Self {
            viewport,
            scissor,
            instance,
            entry,
            debug_utils,
            surface,
            surface_loader,
            physical_device_requirement,
            physical_device,
            max_msaa_samples,
            physical_device_properties,
            physical_device_features,
            device_features,
            queue_family_indices,
            device_requirement,
            device,
            allocator,
            graphics_queue,
            present_queue,
            compute_queue,
            depth_image_format,
            command_pool,
            command_pools,
            current_command_buffer: command_buffers[0],
            command_buffers,
            max_material_count,
            descriptor_pool,
            image_available_for_render_semaphores,
            image_finished_for_presentation_semaphores,
            image_available_for_textures_copy_semaphores,
            frame_in_flight_fences,
            swapchain,
            swapchain_loader,
            swapchain_images,
            surface_format,
            swapchain_extent,
            swapchain_image_views,
            depth_image,
            depth_image_allocation,
            depth_image_view,
            current_frame_index: 0,
        })
    }

    unsafe fn prepare_context(&mut self) {
        self.current_command_buffer = self.command_buffers[self.current_frame_index];
    }

    unsafe fn recreate_swapchain(&mut self, size: RHIExtent2D) -> Result<(), RHIError> {
        unsafe {
            let fences = self
                .frame_in_flight_fences
                .iter()
                .map(|x| x.raw)
                .collect::<Vec<_>>();
            self.device.wait_for_fences(&fences, true, u64::MAX)?;
            self.device
                .destroy_image_view(self.depth_image_view.raw, None);
            self.device.destroy_image(self.depth_image.raw, None);
            if let Some(allocation) = self.depth_image_allocation.take() {
                self.allocator.lock().free(allocation)?;
            }
            for image_view in &self.swapchain_image_views {
                self.device.destroy_image_view(image_view.raw, None);
            }
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
            let (swapchain, swapchain_loader, swapchain_images, surface_format, swapchain_extent) = unsafe {
                VulkanRHI::create_swapchain(
                    &self.device,
                    &self.instance,
                    &self.surface_loader,
                    self.surface,
                    self.physical_device,
                    self.queue_family_indices,
                    size,
                    MAX_FRAME_IN_FLIGHT,
                )?
            };
            self.swapchain = swapchain;
            self.swapchain_loader = swapchain_loader;
            self.swapchain_images = swapchain_images;
            self.surface_format = surface_format;
            self.swapchain_extent = swapchain_extent;
            self.scissor = RHIRect2D {
                offset: RHIOffset2D { x: 0, y: 0 },
                extent: RHIExtent2D {
                    width: swapchain_extent.width,
                    height: swapchain_extent.height,
                },
            };

            let swapchain_image_views = VulkanRHI::create_swapchain_image_views(
                &self.device,
                &self.swapchain_images,
                surface_format,
            )?;
            self.swapchain_image_views = swapchain_image_views;

            let (depth_image, depth_image_allocation, depth_image_view) =
                VulkanRHI::create_framebuffer_images_and_image_views(
                    &self.device,
                    &self.allocator,
                    self.depth_image_format,
                    swapchain_extent,
                )?;
            self.depth_image = depth_image;
            self.depth_image_allocation = depth_image_allocation;
            self.depth_image_view = depth_image_view;
        }
        Ok(())
    }

    unsafe fn wait_for_fences(&mut self) -> Result<(), RHIError> {
        unsafe {
            self.device.wait_for_fences(
                &[self.frame_in_flight_fences[self.current_frame_index].raw],
                true,
                u64::MAX,
            )?
        };
        Ok(())
    }

    unsafe fn prepare_before_render_pass<F>(
        &mut self,
        pass_update_after_recreate_swapchain: F,
    ) -> Result<(), RHIError>
    where
        F: FnOnce(),
    {
        let result = unsafe {
            self.swapchain_loader.acquire_next_image(
                self.swapchain,
                u64::MAX,
                self.image_available_for_render_semaphores[self.current_frame_index].raw,
                vk::Fence::null(),
            )
        };
        if result == Err(vk::Result::ERROR_OUT_OF_DATE_KHR) {
            unsafe { self.recreate_swapchain(self.swapchain_extent)? };
            pass_update_after_recreate_swapchain();
            return Ok(());
        } else if result == Err(vk::Result::SUBOPTIMAL_KHR) {
            unsafe { self.recreate_swapchain(self.swapchain_extent)? };
            pass_update_after_recreate_swapchain();
            // NULL submit to wait semaphore
            let semaphores =
                &[self.image_available_for_render_semaphores[self.current_frame_index].raw];
            let wait_dst_stage_mask = &[vk::PipelineStageFlags::BOTTOM_OF_PIPE];
            let submit_info = vk::SubmitInfo::builder()
                .wait_semaphores(semaphores)
                .wait_dst_stage_mask(wait_dst_stage_mask)
                .build();
            let fence = self.frame_in_flight_fences[self.current_frame_index].raw;
            let fences = &[fence];
            unsafe {
                self.device.reset_fences(fences)?;
            }
            let submit_infos = &[submit_info];
            unsafe {
                self.device
                    .queue_submit(self.graphics_queue.raw, submit_infos, fence)?;
            }
            self.current_frame_index =
                (self.current_frame_index + 1) % (MAX_FRAME_IN_FLIGHT as usize);
            return Ok(());
        } else if let Err(e) = result {
            return Err(RHIError::from(e));
        }
        // let (image_index, sub_optimal) = result.unwrap();
        let begin_info = vk::CommandBufferBeginInfo::builder().build(); // Optional.

        let command_buffer = self.command_buffers[self.current_frame_index].raw;
        unsafe {
            self.device
                .begin_command_buffer(command_buffer, &begin_info)?;
        }
        Ok(())
    }

    unsafe fn allocate_command_buffers(
        &self,
        allocate_info: CommandBufferAllocateInfo<Self>,
    ) -> Result<Vec<CommandBuffer>, RHIError> {
        let level = match allocate_info.level {
            RHICommandBufferLevel::PRIMARY => vk::CommandBufferLevel::PRIMARY,
            RHICommandBufferLevel::SECONDARY => vk::CommandBufferLevel::SECONDARY,
        };
        let create_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.command_pool.raw)
            .level(level)
            .command_buffer_count(allocate_info.count)
            .build();

        let command_buffers = unsafe { self.device.allocate_command_buffers(&create_info)? };
        Ok(command_buffers
            .iter()
            .map(|x| CommandBuffer::new(*x))
            .collect())
    }

    unsafe fn create_command_pool(
        &self,
        create_info: &RHICommandPoolCreateInfo,
    ) -> Result<Self::CommandPool, RHIError> {
        let create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(create_info.queue_family_index)
            // Allow command buffers to be rerecorded individually, without this flag they all have to be reset together
            .flags(create_info.flags.to_vk())
            .build();
        let raw = unsafe { self.device.create_command_pool(&create_info, None)? };
        Ok(VulkanCommandPool { raw })
    }

    unsafe fn create_render_pass(
        &self,
        create_info: &RHIRenderPassCreateInfo,
    ) -> Result<Self::RenderPass, RHIError> {
        let mut attachments = Vec::with_capacity(create_info.attachments.len());
        for attachment in create_info.attachments {
            let vk_attachment = vk::AttachmentDescription::builder()
                .flags(attachment.flags.to_vk())
                .format(attachment.format.to_vk())
                .samples(attachment.samples.to_vk())
                .load_op(attachment.load_op.to_vk())
                .store_op(attachment.store_op.to_vk())
                .stencil_load_op(attachment.stecil_load_op.to_vk())
                .stencil_store_op(attachment.stecil_store_op.to_vk())
                .initial_layout(attachment.initial_layout.to_vk())
                .final_layout(attachment.final_layout.to_vk())
                .build();
            attachments.push(vk_attachment);
        }

        let mut subpasses = Vec::with_capacity(create_info.subpasses.len());
        for attachment in create_info.subpasses {
            let input_attachments = attachment
                .input_attachments
                .iter()
                .map(|x| x.to_vk())
                .collect::<Vec<_>>();
            let color_attachments = attachment
                .color_attachments
                .iter()
                .map(|x| x.to_vk())
                .collect::<Vec<_>>();
            let resolve_attachments = attachment
                .resolve_attachments
                .iter()
                .map(|x| x.to_vk())
                .collect::<Vec<_>>();
            let vk_subpass_desc = vk::SubpassDescription::builder()
                .flags(attachment.flags.to_vk())
                .pipeline_bind_point(attachment.pipeline_bind_point.to_vk())
                .input_attachments(&input_attachments)
                .color_attachments(&color_attachments)
                .resolve_attachments(&resolve_attachments)
                .depth_stencil_attachment(&attachment.depth_stencil_attachment.to_vk())
                .build();
            subpasses.push(vk_subpass_desc);
        }

        let mut dependencies = Vec::with_capacity(create_info.dependencies.len());
        for dependency in create_info.dependencies {
            let vk_dependency = vk::SubpassDependency::builder()
                .src_subpass(dependency.src_subpass)
                .dst_subpass(dependency.dst_subpass)
                .src_stage_mask(dependency.src_stage_mask.to_vk())
                .dst_stage_mask(dependency.dst_stage_mask.to_vk())
                .src_access_mask(dependency.src_access_mask.to_vk())
                .dst_access_mask(dependency.dst_access_mask.to_vk())
                .build();
            dependencies.push(vk_dependency);
        }

        let render_pass_create_info = vk::RenderPassCreateInfo::builder()
            .flags(create_info.flags.to_vk())
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&dependencies)
            .build();
        let raw = unsafe {
            self.device
                .create_render_pass(&render_pass_create_info, None)?
        };

        Ok(VulkanRenderPass { raw })
    }

    unsafe fn clear(&mut self) {
        if let Some(DebugUtils {
            extension,
            messenger,
        }) = self.debug_utils.take()
        {
            unsafe {
                extension.destroy_debug_utils_messenger(messenger, None);
            }
        }
    }
}

impl VulkanRHI {
    unsafe fn create_instance(
        desc: &InstanceDescriptor,
    ) -> Result<(ash::Entry, ash::Instance, Option<DebugUtils>), RHIError> {
        #[cfg(not(target_os = "macos"))]
        let vulkan_api_version = vk::API_VERSION_1_3;

        #[cfg(target_os = "macos")]
        // https://github.com/KhronosGroup/MoltenVK/issues/1567
        let vulkan_api_version = vk::API_VERSION_1_1;

        #[cfg(not(target_os = "macos"))]
        let entry = ash::Entry::linked();

        // #[cfg(target_os = "macos")]
        // let entry = ash_molten::linked();

        let app_name = CString::new(desc.name).unwrap();
        let engine_name = CString::new("Ysera Engine").unwrap();
        let app_info = vk::ApplicationInfo::builder()
            .application_version(vk::make_api_version(0, 1, 0, 0))
            .engine_version(vk::make_api_version(0, 1, 0, 0))
            .application_name(app_name.as_c_str())
            .engine_name(engine_name.as_c_str())
            .api_version(vulkan_api_version);
        let mut enable_validation = desc.flags.contains(InstanceFlags::VALIDATION);
        let mut required_layers = vec![];
        if enable_validation {
            if debug::check_validation_layer_support(&entry, required_layers.as_slice()) {
                required_layers.push("VK_LAYER_KHRONOS_validation");
            } else {
                enable_validation = false;
                log::error!("Validation layers requested, but not available!");
            }
        }

        let enable_debug = if enable_validation {
            desc.flags.contains(InstanceFlags::DEBUG)
        } else {
            false
        };

        let required_layer_raw_names: Vec<CString> = required_layers
            .iter()
            .map(|layer_name| CString::new(*layer_name).unwrap())
            .collect();

        let enable_layer_names: Vec<*const i8> = required_layer_raw_names
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect();

        let extension_cstr_names = platforms::required_extension_names(enable_debug);
        log::debug!("Required extension:");
        let extension_names: Vec<*const i8> = extension_cstr_names
            .iter()
            .map(|x| {
                log::debug!("  * {}", x.to_str().unwrap());
                x.as_ptr()
            })
            .collect();

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_layer_names(enable_layer_names.as_slice())
            .enabled_extension_names(extension_names.as_slice());

        log::debug!("Creating Vulkan instance...");
        let instance: ash::Instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .map_err(RHIError::Vulkan)?
        };

        let debug_utils: Option<DebugUtils> =
            if extension_cstr_names.contains(&ext::DebugUtils::name()) {
                log::info!("Enabling debug utils");
                let vk_msg_max_level = match desc.debug_level_filter {
                    LevelFilter::Error => vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
                    LevelFilter::Warn => vk::DebugUtilsMessageSeverityFlagsEXT::WARNING,
                    LevelFilter::Info => vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
                    LevelFilter::Trace | LevelFilter::Debug => {
                        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                    }
                    _ => vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
                };
                let (extension, messenger) =
                    debug::setup_debug_utils(&entry, &instance, vk_msg_max_level)?;
                Some(DebugUtils {
                    extension,
                    messenger,
                })
            } else {
                None
            };
        log::debug!("Vulkan instance created.");
        Ok((entry, instance, debug_utils))
    }

    unsafe fn create_surface(
        entry: &ash::Entry,
        instance: &ash::Instance,
        window_handle: &dyn HasRawWindowHandle,
        display_handle: &dyn HasRawDisplayHandle,
    ) -> Result<(vk::SurfaceKHR, khr::Surface), RHIError> {
        let surface = ash_window::create_surface(
            entry,
            instance,
            display_handle.raw_display_handle(),
            window_handle.raw_window_handle(),
            None,
        )?;
        let surface_loader = khr::Surface::new(entry, instance);
        Ok((surface, surface_loader))
    }

    unsafe fn initialize_physical_device(
        instance: &ash::Instance,
        surface_loader: &khr::Surface,
        surface: vk::SurfaceKHR,
        physical_device_requirement: &PhysicalDeviceRequirements,
    ) -> Result<
        (
            vk::PhysicalDevice,
            vk::SampleCountFlags,
            vk::PhysicalDeviceProperties,
            vk::PhysicalDeviceFeatures,
            DeviceFeatures,
            QueueFamilyIndices,
        ),
        RHIError,
    > {
        let physical_devices = unsafe { instance.enumerate_physical_devices()? };
        log::info!(
            "{} devices (GPU) found with vulkan support.",
            physical_devices.len()
        );

        let mut physical_device = physical_devices[0];
        let mut extra_features = DeviceFeatures::builder().build();
        let mut find_physical_device_success = false;

        let mut ray_tracing_feature = vk::PhysicalDeviceRayTracingPipelineFeaturesKHR::default();
        let mut acceleration_struct_feature =
            vk::PhysicalDeviceAccelerationStructureFeaturesKHR::default();
        let mut vulkan_12_features = vk::PhysicalDeviceVulkan12Features::builder()
            .runtime_descriptor_array(true)
            .buffer_device_address(true)
            .build();
        let mut vulkan_13_features = vk::PhysicalDeviceVulkan13Features::default();

        for &each_physical_device in physical_devices.iter() {
            let max_msaa_samples = Self::get_max_msaa_samples(each_physical_device, instance);
            let properties =
                unsafe { instance.get_physical_device_properties(each_physical_device) };
            let features = unsafe { instance.get_physical_device_features(each_physical_device) };
            let queue_family_properties =
                unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
            let queue_family_indices = unsafe {
                Self::get_queue_family_indices(
                    each_physical_device,
                    &queue_family_properties,
                    surface,
                    surface_loader,
                )?
            };
            let this_extra_features = Self::get_support_physical_device_features(
                features,
                vulkan_12_features,
                vulkan_13_features,
                ray_tracing_feature,
                acceleration_struct_feature,
            );
            Self::log_physical_device_information(each_physical_device, instance);

            let meet = unsafe {
                Self::physical_device_meet_requirements(
                    each_physical_device,
                    properties,
                    features,
                    &queue_family_indices,
                    &this_extra_features,
                    &physical_device_requirement,
                )
            };
            if meet {
                physical_device = each_physical_device;
                extra_features = this_extra_features;
                find_physical_device_success = true;
                break;
            }
        }
        if !find_physical_device_success {
            log::error!(
                "Cannot find a physical device which meet the requirement: {:#?}",
                physical_device_requirement
            );
            return Err(RHIError::NotMeetRequirement);
        }

        log::debug!(
            "Find a physical device which meet the requirement: {:#?}",
            physical_device_requirement
        );

        let max_msaa_samples = Self::get_max_msaa_samples(physical_device, instance);
        let properties = unsafe { instance.get_physical_device_properties(physical_device) };
        let features = unsafe { instance.get_physical_device_features(physical_device) };

        let queue_family_properties =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let mut features2 = vk::PhysicalDeviceFeatures2::builder()
            .push_next(&mut ray_tracing_feature)
            .push_next(&mut acceleration_struct_feature)
            .push_next(&mut vulkan_12_features)
            .push_next(&mut vulkan_13_features);
        unsafe { instance.get_physical_device_features2(physical_device, &mut features2) };

        let queue_family_indices = Self::get_queue_family_indices(
            physical_device,
            &queue_family_properties,
            surface,
            surface_loader,
        )?;

        Ok((
            physical_device,
            max_msaa_samples,
            properties,
            features,
            extra_features,
            queue_family_indices,
        ))
    }

    unsafe fn create_logical_device(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        physical_device_req: &PhysicalDeviceRequirements,
        device_req: &DeviceRequirement,
        queue_family_indices: &QueueFamilyIndices,
        instance_flags: &InstanceFlags,
    ) -> Result<
        (
            ash::Device,
            VulkanQueue,
            VulkanQueue,
            VulkanQueue,
            RHIFormat,
        ),
        RHIError,
    > {
        queue_family_indices.log_debug();

        let queue_priorities = &[1_f32];

        let mut unique_indices = HashSet::new();
        unique_indices.insert(queue_family_indices.graphics_family.unwrap());
        unique_indices.insert(queue_family_indices.present_family.unwrap());

        let queue_create_infos = unique_indices
            .iter()
            .map(|i| {
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(*i)
                    .queue_priorities(queue_priorities)
                    .build()
            })
            .collect::<Vec<_>>();

        let enable_validation = instance_flags.contains(InstanceFlags::VALIDATION);
        let mut required_layers = vec![];
        if enable_validation {
            required_layers.push("VK_LAYER_KHRONOS_validation");
        }
        let required_validation_layer_raw_names: Vec<CString> = required_layers
            .iter()
            .map(|layer_name| CString::new(*layer_name).unwrap())
            .collect();
        let enable_layer_names: Vec<*const c_char> = required_validation_layer_raw_names
            .iter()
            .map(|layer_name| layer_name.as_ptr())
            .collect();

        let mut device_extensions_ptrs = device_req
            .required_extension
            .iter()
            .map(|e| CString::new(e.as_str()))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        if device_req.use_swapchain {
            device_extensions_ptrs.push(khr::Swapchain::name().into());
        }

        let physical_device_features = vk::PhysicalDeviceFeatures::builder()
            .sampler_anisotropy(physical_device_req.extra_features.sampler_anisotropy)
            .sample_rate_shading(physical_device_req.extra_features.sample_rate_shading)
            .fragment_stores_and_atomics(
                physical_device_req
                    .extra_features
                    .fragment_stores_and_atomics,
            )
            .independent_blend(physical_device_req.extra_features.independent_blend)
            .geometry_shader(physical_device_req.extra_features.geometry_shader)
            .build();

        let mut ray_tracing_feature = vk::PhysicalDeviceRayTracingPipelineFeaturesKHR::builder()
            .ray_tracing_pipeline(physical_device_req.extra_features.ray_tracing_pipeline);
        let mut acceleration_struct_feature =
            vk::PhysicalDeviceAccelerationStructureFeaturesKHR::builder()
                .acceleration_structure(physical_device_req.extra_features.acceleration_structure);
        let mut vulkan_12_features = vk::PhysicalDeviceVulkan12Features::builder()
            .runtime_descriptor_array(physical_device_req.extra_features.runtime_descriptor_array)
            .buffer_device_address(physical_device_req.extra_features.buffer_device_address);
        let mut vulkan_13_features = vk::PhysicalDeviceVulkan13Features::builder()
            .dynamic_rendering(physical_device_req.extra_features.dynamic_rendering)
            .synchronization2(physical_device_req.extra_features.synchronization2);

        let mut physical_device_features2 = vk::PhysicalDeviceFeatures2::builder()
            .features(physical_device_features)
            .push_next(&mut ray_tracing_feature)
            .push_next(&mut acceleration_struct_feature)
            .push_next(&mut vulkan_12_features)
            .push_next(&mut vulkan_13_features);

        let device_extensions_ptrs = device_extensions_ptrs
            .iter()
            // Safe because `enabled_extensions` entries have static lifetime.
            .map(|e| e.as_ptr())
            .collect::<Vec<_>>();

        let support_extensions = unsafe {
            Self::check_device_extension_support(
                instance,
                physical_device,
                &device_req.required_extension,
                device_req.use_swapchain,
            )
        };

        if !support_extensions {
            log::error!("device extensions not support");
        }

        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_infos)
            .enabled_layer_names(&enable_layer_names)
            .enabled_extension_names(&device_extensions_ptrs)
            .push_next(&mut physical_device_features2);

        let device: ash::Device =
            unsafe { instance.create_device(physical_device, &device_create_info, None)? };

        log::debug!("Vulkan logical device created.");

        let graphics_queue =
            unsafe { device.get_device_queue(queue_family_indices.graphics_family.unwrap(), 0) };
        let present_queue =
            unsafe { device.get_device_queue(queue_family_indices.present_family.unwrap(), 0) };
        let compute_queue =
            unsafe { device.get_device_queue(queue_family_indices.compute_family.unwrap(), 0) };
        let depth_image_format = Self::get_depth_format(instance, physical_device)?;

        Ok((
            device,
            VulkanQueue {
                raw: graphics_queue,
            },
            VulkanQueue { raw: present_queue },
            VulkanQueue { raw: compute_queue },
            depth_image_format.into(),
        ))
    }

    fn create_allocator(
        device: &ash::Device,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
    ) -> Result<Mutex<Allocator>, RHIError> {
        let allocator = Allocator::new(&AllocatorCreateDesc {
            instance: instance.clone(),
            device: device.clone(),
            physical_device,
            debug_settings: Default::default(),
            // check https://stackoverflow.com/questions/73341075/rust-gpu-allocator-bufferdeviceaddress-must-be-enabbled
            buffer_device_address: false,
        })?;

        Ok(Mutex::new(allocator))
    }

    unsafe fn create_command_pool(
        device: &ash::Device,
        queue_family_indices: &QueueFamilyIndices,
        max_frames_in_flight: u8,
    ) -> Result<(VulkanCommandPool, Vec<VulkanCommandPool>), RHIError> {
        let create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(queue_family_indices.graphics_family.unwrap())
            // Allow command buffers to be rerecorded individually, without this flag they all have to be reset together
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .build();
        let raw = unsafe { device.create_command_pool(&create_info, None)? };
        let command_pool = VulkanCommandPool { raw };
        let mut command_pools = Vec::with_capacity(max_frames_in_flight as usize);
        for _ in 0..max_frames_in_flight {
            let each_create_info = vk::CommandPoolCreateInfo::builder()
                .queue_family_index(queue_family_indices.graphics_family.unwrap())
                //  Hint that command buffers are rerecorded with new commands very often (may change memory allocation behavior)
                .flags(vk::CommandPoolCreateFlags::TRANSIENT)
                .build();
            let raw = unsafe { device.create_command_pool(&each_create_info, None)? };
            let each_raw = VulkanCommandPool { raw };
            command_pools.push(each_raw);
        }

        Ok((command_pool, command_pools))
    }

    unsafe fn create_command_buffers(
        device: &ash::Device,
        command_pool: vk::CommandPool,
        max_frames_in_flight: u8,
    ) -> Result<Vec<VulkanCommandBuffer>, RHIError> {
        let mut command_buffers = Vec::with_capacity(max_frames_in_flight as usize);
        for _ in 0..max_frames_in_flight {
            let create_info = vk::CommandBufferAllocateInfo::builder()
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_pool(command_pool)
                .command_buffer_count(1)
                .build();
            let raw = unsafe { device.allocate_command_buffers(&create_info)?[0] };
            command_buffers.push(VulkanCommandBuffer { raw });
        }

        Ok(command_buffers)
    }

    unsafe fn create_descriptor_pool(
        device: &ash::Device,
        max_material_count: u32,
        // max_vertex_blending_mesh_count: u32,
        max_frames_in_flight: u8,
    ) -> Result<VulkanDescriptorPool, RHIError> {
        let max_frames_in_flight = max_frames_in_flight as u32;
        let uniform_buffer_size = vk::DescriptorPoolSize::builder()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(max_frames_in_flight * max_material_count)
            .build();
        let sampler_size = vk::DescriptorPoolSize::builder()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(max_frames_in_flight + 5 * max_material_count)
            .build();
        let pool_sizes = &[uniform_buffer_size, sampler_size];
        let info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(pool_sizes)
            .max_sets(max_frames_in_flight);
        let raw = unsafe { device.create_descriptor_pool(&info, None)? };
        log::debug!("Descriptor Pool created.");

        Ok(VulkanDescriptorPool { raw })
    }

    unsafe fn create_sync_objects(
        device: &ash::Device,
        max_frames_in_flight: u8,
    ) -> Result<
        (
            Vec<VulkanSemaphore>,
            Vec<VulkanSemaphore>,
            Vec<VulkanSemaphore>,
            Vec<VulkanFence>,
        ),
        RHIError,
    > {
        let max_frames_in_flight = max_frames_in_flight as usize;
        let mut image_available_for_render_semaphores = Vec::with_capacity(max_frames_in_flight);
        let mut image_finished_for_presentation_semaphores =
            Vec::with_capacity(max_frames_in_flight);
        let mut image_available_for_textures_copy_semaphores =
            Vec::with_capacity(max_frames_in_flight);
        let mut frame_in_flight_fences = Vec::with_capacity(max_frames_in_flight);

        let semaphore_create_info = vk::SemaphoreCreateInfo::builder();
        let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
        for _ in 0..max_frames_in_flight {
            unsafe {
                image_available_for_render_semaphores.push(VulkanSemaphore {
                    raw: device.create_semaphore(&semaphore_create_info, None)?,
                });
                image_finished_for_presentation_semaphores.push(VulkanSemaphore {
                    raw: device.create_semaphore(&semaphore_create_info, None)?,
                });
                image_finished_for_presentation_semaphores.push(VulkanSemaphore {
                    raw: device.create_semaphore(&semaphore_create_info, None)?,
                });
                frame_in_flight_fences.push(VulkanFence {
                    raw: device.create_fence(&fence_info, None)?,
                });
            }
        }
        Ok((
            image_available_for_render_semaphores,
            image_finished_for_presentation_semaphores,
            image_available_for_textures_copy_semaphores,
            frame_in_flight_fences,
        ))
    }

    unsafe fn create_swapchain(
        device: &ash::Device,
        instance: &ash::Instance,
        surface_loader: &khr::Surface,
        surface: vk::SurfaceKHR,
        physical_device: vk::PhysicalDevice,
        indices: QueueFamilyIndices,
        preferred_dimensions: RHIExtent2D,
        max_frames_in_flight: u8,
    ) -> Result<
        (
            vk::SwapchainKHR,
            khr::Swapchain,
            Vec<vk::Image>,
            RHIFormat,
            RHIExtent2D,
        ),
        RHIError,
    > {
        profiling::scope!("create_swapchain");

        let support =
            unsafe { SwapChainSupportDetail::new(physical_device, surface_loader, surface) }?;
        let properties = support.get_ideal_swapchain_properties(preferred_dimensions);
        let SwapchainProperties {
            surface_format,
            present_mode,
            extent,
        } = properties;

        let mut image_count = support.capabilities.min_image_count + 1;
        image_count = if support.capabilities.max_image_count > 0 {
            image_count.min(support.capabilities.max_image_count)
        } else {
            image_count
        };

        let graphics_family_queue_index = indices.graphics_family.unwrap();
        let present_family_queue_index = indices.present_family.unwrap();
        let (image_sharing_mode, queue_family_indices) =
            if graphics_family_queue_index != present_family_queue_index {
                (
                    // 图像可以在多个队列族间使用，不需要显式地改变图像所有权。
                    // 如果图形和呈现不是同一个队列族，我们使用协同模式来避免处理图像所有权问题。
                    vk::SharingMode::CONCURRENT,
                    vec![graphics_family_queue_index, present_family_queue_index],
                )
            } else {
                // 一张图像同一时间只能被一个队列族所拥有，在另一队列族使用它之前，必须显式地改变图像所有权。
                // 这一模式下性能表现最佳。
                (vk::SharingMode::EXCLUSIVE, vec![])
            };

        let create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
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
            .image_array_layers(1);

        let swapchain_loader = khr::Swapchain::new(instance, device);
        let swapchain = unsafe { swapchain_loader.create_swapchain(&create_info, None)? };
        log::debug!("Vulkan swapchain created. min_image_count: {}", image_count);

        let swapchain_images = unsafe { swapchain_loader.get_swapchain_images(swapchain.clone())? };
        let surface_format = surface_format.format.into();
        let swapchain_extent = RHIExtent2D {
            width: support.capabilities.current_extent.width.max(1),
            height: support.capabilities.current_extent.height.max(1),
        };
        Ok((
            swapchain,
            swapchain_loader,
            swapchain_images,
            surface_format,
            swapchain_extent,
        ))
    }

    unsafe fn create_swapchain_image_views(
        device: &ash::Device,
        images: &[vk::Image],
        surface_format: RHIFormat,
    ) -> Result<Vec<VulkanImageView>, RHIError> {
        let mut image_views = Vec::with_capacity(images.len());
        for image in images {
            let raw = unsafe {
                utils::create_image_view(
                    device,
                    *image,
                    surface_format.to_vk(),
                    vk::ImageAspectFlags::COLOR,
                    vk::ImageViewType::TYPE_2D,
                    1,
                    1,
                )?
            };
            image_views.push(VulkanImageView { raw })
        }
        Ok(image_views)
    }

    unsafe fn create_framebuffer_images_and_image_views(
        device: &ash::Device,
        allocator: &Mutex<Allocator>,
        depth_format: RHIFormat,
        swapchain_extent: RHIExtent2D,
    ) -> Result<(VulkanImage, Option<Allocation>, VulkanImageView), RHIError> {
        let (image, allocation) = unsafe {
            utils::create_image(
                device,
                allocator,
                swapchain_extent,
                depth_format.to_vk(),
                vk::ImageTiling::OPTIMAL,
                vk::ImageUsageFlags::INPUT_ATTACHMENT
                    | vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT
                    | vk::ImageUsageFlags::TRANSFER_SRC,
                vk::ImageCreateFlags::empty(),
                1,
                1,
            )?
        };

        let image_view = unsafe {
            utils::create_image_view(
                device,
                image,
                depth_format.to_vk(),
                vk::ImageAspectFlags::DEPTH,
                vk::ImageViewType::TYPE_2D,
                1,
                1,
            )?
        };
        let image = VulkanImage { raw: image };
        let image_view = VulkanImageView { raw: image_view };

        Ok((image, allocation, image_view))
    }
}

impl VulkanRHI {
    fn log_physical_device_information(
        physical_device: vk::PhysicalDevice,
        instance: &ash::Instance,
    ) {
        let device_properties = unsafe { instance.get_physical_device_properties(physical_device) };
        let device_features = unsafe { instance.get_physical_device_features(physical_device) };
        let device_queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let device_type = match device_properties.device_type {
            vk::PhysicalDeviceType::CPU => "Cpu",
            vk::PhysicalDeviceType::INTEGRATED_GPU => "Integrated GPU",
            vk::PhysicalDeviceType::DISCRETE_GPU => "Discrete GPU",
            vk::PhysicalDeviceType::VIRTUAL_GPU => "Virtual GPU",
            vk::PhysicalDeviceType::OTHER => "Unknown",
            _ => panic!(),
        };
        let device_name = c_char_to_string(&device_properties.device_name);
        log::debug!(
            "\tDevice Name: {}, id: {}, type: {}",
            device_name,
            device_properties.device_id,
            device_type
        );
        let major_version = vk::api_version_major(device_properties.api_version);
        let minor_version = vk::api_version_minor(device_properties.api_version);
        let patch_version = vk::api_version_patch(device_properties.api_version);
        log::debug!(
            "\tAPI Version: {}.{}.{}",
            major_version,
            minor_version,
            patch_version
        );
        log::debug!("\tSupport Queue Family: {}", device_queue_families.len());
        log::debug!(
            "\t\tQueue Count | {: ^10} | {: ^10} | {: ^10} | {: ^15}",
            "Graphics",
            "Compute",
            "Transfer",
            "Sparse Binding"
        );

        for queue_family in device_queue_families.iter() {
            let is_graphics_support = if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            {
                "support"
            } else {
                "unsupport"
            };
            let is_compute_support = if queue_family.queue_flags.contains(vk::QueueFlags::COMPUTE) {
                "support"
            } else {
                "unsupport"
            };
            let is_transfer_support = if queue_family.queue_flags.contains(vk::QueueFlags::TRANSFER)
            {
                "support"
            } else {
                "unsupport"
            };
            let is_sparse_support = if queue_family
                .queue_flags
                .contains(vk::QueueFlags::SPARSE_BINDING)
            {
                "support"
            } else {
                "unsupport"
            };
            log::debug!(
                "\t\t{}\t    | {: ^10} | {: ^10} | {: ^10} | {: ^15}",
                queue_family.queue_count,
                is_graphics_support,
                is_compute_support,
                is_transfer_support,
                is_sparse_support
            );
        }
    }

    fn get_max_msaa_samples(
        physical_device: vk::PhysicalDevice,
        instance: &ash::Instance,
    ) -> vk::SampleCountFlags {
        let properties = unsafe { instance.get_physical_device_properties(physical_device) };
        let counts = properties.limits.framebuffer_color_sample_counts
            & properties.limits.framebuffer_depth_sample_counts;
        [
            vk::SampleCountFlags::TYPE_64,
            vk::SampleCountFlags::TYPE_32,
            vk::SampleCountFlags::TYPE_16,
            vk::SampleCountFlags::TYPE_8,
            vk::SampleCountFlags::TYPE_4,
            vk::SampleCountFlags::TYPE_2,
        ]
        .iter()
        .cloned()
        .find(|c| counts.contains(*c))
        .unwrap_or(vk::SampleCountFlags::TYPE_1)
    }

    unsafe fn physical_device_meet_requirements(
        physical_device: vk::PhysicalDevice,
        properties: vk::PhysicalDeviceProperties,
        features: vk::PhysicalDeviceFeatures,
        queue_family_indices: &QueueFamilyIndices,
        extra_features: &DeviceFeatures,
        requirements: &PhysicalDeviceRequirements,
    ) -> bool {
        if requirements.discrete_gpu
            && properties.device_type != vk::PhysicalDeviceType::DISCRETE_GPU
        {
            log::error!("This Physical Device is not a discrete GPU, and one is required!");
            return false;
        }

        if !queue_family_indices.has_meet_requirement(requirements) {
            log::error!("This Physical Device is not meet queue family indices' requirement! \nindices is {:#?},\nbut requirement is {:#?}", queue_family_indices, requirements);
            return false;
        }

        if !extra_features.is_compatible_with(&requirements.extra_features) {
            log::error!("This Physical Device is not support extra features! \nsupport {:#?},\nbut requirement is {:#?}", extra_features, requirements.extra_features);
            return false;
        }
        return true;
    }

    fn get_support_physical_device_features(
        features: vk::PhysicalDeviceFeatures,
        features_12: vk::PhysicalDeviceVulkan12Features,
        features_13: vk::PhysicalDeviceVulkan13Features,
        features_ray_tracing_pipeline: vk::PhysicalDeviceRayTracingPipelineFeaturesKHR,
        features_acceleration_structure: vk::PhysicalDeviceAccelerationStructureFeaturesKHR,
    ) -> DeviceFeatures {
        DeviceFeatures {
            sampler_anisotropy: features.sampler_anisotropy == vk::TRUE,
            sample_rate_shading: features.sample_rate_shading == vk::TRUE,
            fragment_stores_and_atomics: features.fragment_stores_and_atomics == vk::TRUE,
            independent_blend: features.independent_blend == vk::TRUE,
            geometry_shader: features.geometry_shader == vk::TRUE,
            ray_tracing_pipeline: features_ray_tracing_pipeline.ray_tracing_pipeline == vk::TRUE,
            acceleration_structure: features_acceleration_structure.acceleration_structure
                == vk::TRUE,
            runtime_descriptor_array: features_12.runtime_descriptor_array == vk::TRUE,
            buffer_device_address: features_12.buffer_device_address == vk::TRUE,
            dynamic_rendering: features_13.dynamic_rendering == vk::TRUE,
            synchronization2: features_13.synchronization2 == vk::TRUE,
        }
    }

    unsafe fn get_queue_family_indices(
        adapter: vk::PhysicalDevice,
        queue_family_properties: &[vk::QueueFamilyProperties],
        surface: vk::SurfaceKHR,
        surface_loader: &khr::Surface,
    ) -> Result<QueueFamilyIndices, RHIError> {
        let mut indices = QueueFamilyIndices::default();
        for (i, queue_family) in queue_family_properties.iter().enumerate() {
            if indices.is_complete() {
                break;
            }
            let index = i as u32;
            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                indices.graphics_family = Some(index);
            };
            if queue_family.queue_flags.contains(vk::QueueFlags::COMPUTE) {
                indices.compute_family = Some(index);
            };
            if queue_family.queue_flags.contains(vk::QueueFlags::TRANSFER) {
                indices.transfer_family = Some(index);
            };
            let support_present = unsafe {
                surface_loader
                    .get_physical_device_surface_support(adapter, index, surface)
                    .map_err(RHIError::Vulkan)?
            };

            if support_present {
                indices.present_family = Some(index);
            }
        }
        Ok(indices)
    }

    unsafe fn check_device_extension_support(
        instance: &ash::Instance,
        adapter: vk::PhysicalDevice,
        extensions: &[String],
        use_swapchain: bool,
    ) -> bool {
        let extension_props = unsafe {
            instance
                .enumerate_device_extension_properties(adapter)
                .expect("Failed to enumerate device extension properties")
        };

        for required in extensions.iter() {
            let found = extension_props.iter().any(|ext| {
                let name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
                *required == name.to_str().unwrap().to_owned()
            });

            if !found {
                return false;
            }
        }

        let swapchain_check = use_swapchain
            && extensions.contains(&khr::Swapchain::name().to_str().unwrap().to_string());
        swapchain_check
    }

    unsafe fn get_depth_format(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
    ) -> Result<vk::Format, RHIError> {
        let formats = &[
            vk::Format::D32_SFLOAT,
            vk::Format::D32_SFLOAT_S8_UINT,
            vk::Format::D24_UNORM_S8_UINT,
        ];

        unsafe {
            Self::get_supported_format(
                instance,
                physical_device,
                formats,
                vk::ImageTiling::OPTIMAL,
                vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
            )
        }
    }

    unsafe fn get_supported_format(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        formats: &[vk::Format],
        tiling: vk::ImageTiling,
        features: vk::FormatFeatureFlags,
    ) -> Result<vk::Format, RHIError> {
        formats
            .iter()
            .cloned()
            .find(|f| {
                let properties =
                    unsafe { instance.get_physical_device_format_properties(physical_device, *f) };
                match tiling {
                    vk::ImageTiling::LINEAR => properties.linear_tiling_features.contains(features),
                    vk::ImageTiling::OPTIMAL => {
                        properties.optimal_tiling_features.contains(features)
                    }
                    _ => false,
                }
            })
            .ok_or(RHIError::Other("Failed to find supported format!"))
    }
}

struct SwapChainSupportDetail {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub surface_formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

struct SwapchainProperties {
    pub surface_format: vk::SurfaceFormatKHR,
    pub present_mode: vk::PresentModeKHR,
    pub extent: vk::Extent2D,
}

impl SwapChainSupportDetail {
    pub unsafe fn new(
        adapter: vk::PhysicalDevice,
        surface: &khr::Surface,
        surface_khr: vk::SurfaceKHR,
    ) -> Result<SwapChainSupportDetail, RHIError> {
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

    fn get_ideal_swapchain_properties(
        &self,
        preferred_dimensions: RHIExtent2D,
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
        preferred_dimensions: RHIExtent2D,
    ) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            use num::clamp;
            let width = preferred_dimensions.width;
            let height = preferred_dimensions.height;
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
