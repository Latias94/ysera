use std::ffi::CStr;

use ash::vk;

use crate::vulkan::debug::DebugUtils;
use crate::DeviceError;

pub struct Device {
    /// Loads device local functions.
    raw: ash::Device,
    debug_utils: Option<DebugUtils>,
}

impl Device {
    pub fn raw(&self) -> &ash::Device {
        &self.raw
    }

    pub fn new(raw: ash::Device, debug_utils: Option<DebugUtils>) -> Self {
        Self { raw, debug_utils }
    }

    pub fn wait_idle(&self) {
        unsafe { self.raw.device_wait_idle().unwrap() }
    }

    pub fn get_image_memory_requirements(&self, image: vk::Image) -> vk::MemoryRequirements {
        unsafe { self.raw.get_image_memory_requirements(image) }
    }

    pub fn get_buffer_memory_requirements(&self, buffer: vk::Buffer) -> vk::MemoryRequirements {
        unsafe { self.raw.get_buffer_memory_requirements(buffer) }
    }

    pub unsafe fn bind_buffer_memory(
        &self,
        buffer: vk::Buffer,
        device_memory: vk::DeviceMemory,
        offset: vk::DeviceSize,
    ) -> Result<(), DeviceError> {
        unsafe { self.raw.bind_buffer_memory(buffer, device_memory, offset)? };
        Ok(())
    }

    pub unsafe fn bind_image_memory(
        &self,
        image: vk::Image,
        device_memory: vk::DeviceMemory,
        offset: vk::DeviceSize,
    ) -> Result<(), DeviceError> {
        unsafe { self.raw.bind_image_memory(image, device_memory, offset)? };
        Ok(())
    }

    pub fn create_image(
        &self,
        create_info: &vk::ImageCreateInfo,
    ) -> Result<vk::Image, DeviceError> {
        Ok(unsafe { self.raw.create_image(create_info, None)? })
    }

    pub fn destroy_image(&self, image: vk::Image) {
        unsafe {
            self.raw.destroy_image(image, None);
        }
    }

    pub fn create_image_view(
        &self,
        create_info: &vk::ImageViewCreateInfo,
    ) -> Result<vk::ImageView, DeviceError> {
        Ok(unsafe { self.raw.create_image_view(create_info, None)? })
    }

    pub fn destroy_image_view(&self, image_view: vk::ImageView) {
        unsafe {
            self.raw.destroy_image_view(image_view, None);
        }
    }

    pub fn create_shader_module(
        &self,
        create_info: &vk::ShaderModuleCreateInfo,
    ) -> Result<vk::ShaderModule, DeviceError> {
        Ok(unsafe { self.raw.create_shader_module(create_info, None)? })
    }

    pub fn destroy_shader_module(&self, shader_module: vk::ShaderModule) {
        unsafe {
            self.raw.destroy_shader_module(shader_module, None);
        }
    }

    pub fn get_device_queue(&self, queue_family_index: u32, queue_index: u32) -> vk::Queue {
        unsafe { self.raw.get_device_queue(queue_family_index, queue_index) }
    }

    pub fn create_render_pass(
        &self,
        create_info: &vk::RenderPassCreateInfo,
    ) -> Result<vk::RenderPass, DeviceError> {
        Ok(unsafe { self.raw.create_render_pass(create_info, None)? })
    }

    pub fn destroy_render_pass(&self, render_pass: vk::RenderPass) {
        unsafe { self.raw.destroy_render_pass(render_pass, None) }
    }

    pub fn create_framebuffer(
        &self,
        create_info: &vk::FramebufferCreateInfo,
    ) -> Result<vk::Framebuffer, DeviceError> {
        Ok(unsafe { self.raw.create_framebuffer(create_info, None)? })
    }

    pub fn destroy_framebuffer(&self, framebuffer: vk::Framebuffer) {
        unsafe { self.raw.destroy_framebuffer(framebuffer, None) }
    }

    pub fn create_sampler(
        &self,
        create_info: &vk::SamplerCreateInfo,
    ) -> Result<vk::Sampler, DeviceError> {
        Ok(unsafe { self.raw.create_sampler(create_info, None)? })
    }

    pub fn destroy_sampler(&self, sampler: vk::Sampler) {
        unsafe { self.raw.destroy_sampler(sampler, None) }
    }

    pub fn create_pipeline_layout(
        &self,
        create_info: &vk::PipelineLayoutCreateInfo,
    ) -> Result<vk::PipelineLayout, DeviceError> {
        Ok(unsafe { self.raw.create_pipeline_layout(create_info, None)? })
    }

    pub fn destroy_pipeline_layout(&self, pipeline_layout: vk::PipelineLayout) {
        unsafe { self.raw.destroy_pipeline_layout(pipeline_layout, None) }
    }

    pub fn create_graphics_pipelines(
        &self,
        create_infos: &[vk::GraphicsPipelineCreateInfo],
    ) -> Result<Vec<vk::Pipeline>, DeviceError> {
        Ok(unsafe {
            self.raw
                .create_graphics_pipelines(vk::PipelineCache::default(), create_infos, None)
                .map_err(|e| e.1)?
        })
    }

    pub fn destroy_pipeline(&self, pipeline: vk::Pipeline) {
        unsafe { self.raw.destroy_pipeline(pipeline, None) }
    }

    pub fn create_command_pool(
        &self,
        create_info: &vk::CommandPoolCreateInfo,
    ) -> Result<vk::CommandPool, DeviceError> {
        Ok(unsafe { self.raw.create_command_pool(create_info, None)? })
    }

    pub fn destroy_command_pool(&self, command_pool: vk::CommandPool) {
        unsafe { self.raw.destroy_command_pool(command_pool, None) }
    }

    pub fn allocate_command_buffers(
        &self,
        allocate_info: &vk::CommandBufferAllocateInfo,
    ) -> Result<Vec<vk::CommandBuffer>, DeviceError> {
        Ok(unsafe { self.raw.allocate_command_buffers(allocate_info)? })
    }

    pub fn begin_command_buffer(
        &self,
        command_buffer: vk::CommandBuffer,
        begin_info: &vk::CommandBufferBeginInfo,
    ) -> Result<(), DeviceError> {
        unsafe { self.raw.begin_command_buffer(command_buffer, begin_info)? };
        Ok(())
    }

    pub fn end_command_buffer(&self, command_buffer: vk::CommandBuffer) -> Result<(), DeviceError> {
        unsafe { self.raw.end_command_buffer(command_buffer)? };
        Ok(())
    }

    pub fn reset_command_buffer(
        &self,
        command_buffer: vk::CommandBuffer,
        flags: vk::CommandBufferResetFlags,
    ) -> Result<(), DeviceError> {
        unsafe { self.raw.reset_command_buffer(command_buffer, flags)? };
        Ok(())
    }

    pub fn queue_submit(
        &self,
        queue: vk::Queue,
        submits: &[vk::SubmitInfo],
        fence: vk::Fence,
    ) -> Result<(), DeviceError> {
        unsafe { self.raw.queue_submit(queue, submits, fence)? };
        Ok(())
    }

    pub fn queue_wait_idle(&self, queue: vk::Queue) -> Result<(), DeviceError> {
        unsafe { self.raw.queue_wait_idle(queue)? };
        Ok(())
    }

    pub fn free_command_buffers(
        &self,
        command_pool: vk::CommandPool,
        command_buffers: &[vk::CommandBuffer],
    ) {
        unsafe { self.raw.free_command_buffers(command_pool, command_buffers) }
    }

    pub fn create_buffer(
        &self,
        create_info: &vk::BufferCreateInfo,
    ) -> Result<vk::Buffer, DeviceError> {
        Ok(unsafe { self.raw.create_buffer(create_info, None)? })
    }

    pub fn destroy_buffer(&self, buffer: vk::Buffer) {
        unsafe { self.raw.destroy_buffer(buffer, None) }
    }

    pub fn map_memory(
        &self,
        memory: vk::DeviceMemory,
        offset: vk::DeviceSize,
        size: vk::DeviceSize,
        flags: vk::MemoryMapFlags,
    ) -> Result<*mut std::ffi::c_void, DeviceError> {
        Ok(unsafe { self.raw.map_memory(memory, offset, size, flags)? })
    }

    pub fn unmap_memory(&self, memory: vk::DeviceMemory) {
        unsafe { self.raw.unmap_memory(memory) }
    }

    pub fn create_descriptor_set_layout(
        &self,
        create_info: &vk::DescriptorSetLayoutCreateInfo,
    ) -> Result<vk::DescriptorSetLayout, DeviceError> {
        Ok(unsafe { self.raw.create_descriptor_set_layout(create_info, None)? })
    }

    pub fn destroy_descriptor_set_layout(&self, layout: vk::DescriptorSetLayout) {
        unsafe { self.raw.destroy_descriptor_set_layout(layout, None) }
    }

    pub fn create_descriptor_pool(
        &self,
        create_info: &vk::DescriptorPoolCreateInfo,
    ) -> Result<vk::DescriptorPool, DeviceError> {
        Ok(unsafe { self.raw.create_descriptor_pool(create_info, None)? })
    }

    pub fn destroy_descriptor_pool(&self, pool: vk::DescriptorPool) {
        unsafe { self.raw.destroy_descriptor_pool(pool, None) }
    }

    pub fn allocate_descriptor_sets(
        &self,
        create_info: &vk::DescriptorSetAllocateInfo,
    ) -> Result<Vec<vk::DescriptorSet>, DeviceError> {
        Ok(unsafe { self.raw.allocate_descriptor_sets(create_info)? })
    }

    pub fn update_descriptor_sets(
        &self,
        descriptor_writes: &[vk::WriteDescriptorSet],
        descriptor_copies: &[vk::CopyDescriptorSet],
    ) {
        unsafe {
            self.raw
                .update_descriptor_sets(descriptor_writes, descriptor_copies)
        }
    }

    pub fn cmd_begin_render_pass(
        &self,
        command_buffer: vk::CommandBuffer,
        begin_info: &vk::RenderPassBeginInfo,
        contents: vk::SubpassContents,
    ) {
        unsafe {
            self.raw
                .cmd_begin_render_pass(command_buffer, begin_info, contents);
        }
    }

    pub fn cmd_end_render_pass(&self, command_buffer: vk::CommandBuffer) {
        unsafe { self.raw.cmd_end_render_pass(command_buffer) }
    }

    pub fn cmd_set_viewport(&self, command_buffer: vk::CommandBuffer, viewport: math::Rect2D) {
        unsafe {
            let vp = vk::Viewport::builder()
                .x(viewport.x)
                .y(viewport.y)
                .width(viewport.width)
                .height(viewport.height)
                .min_depth(0f32)
                .max_depth(1f32)
                .build();
            self.raw.cmd_set_viewport(command_buffer, 0, &[vp])
        }
    }
    pub fn cmd_set_scissor(
        &self,
        command_buffer: vk::CommandBuffer,
        first_scissor: u32,
        scissors: &[vk::Rect2D],
    ) {
        unsafe {
            self.raw
                .cmd_set_scissor(command_buffer, first_scissor, scissors)
        }
    }

    pub fn cmd_bind_pipeline(
        &self,
        command_buffer: vk::CommandBuffer,
        pipeline_bind_point: vk::PipelineBindPoint,
        pipeline: vk::Pipeline,
    ) {
        unsafe {
            self.raw
                .cmd_bind_pipeline(command_buffer, pipeline_bind_point, pipeline);
        }
    }

    pub fn cmd_draw(
        &self,
        command_buffer: vk::CommandBuffer,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) {
        unsafe {
            self.raw.cmd_draw(
                command_buffer,
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            );
        }
    }

    pub fn cmd_draw_indexed(
        &self,
        command_buffer: vk::CommandBuffer,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) {
        unsafe {
            self.raw.cmd_draw_indexed(
                command_buffer,
                index_count,
                instance_count,
                first_index,
                vertex_offset,
                first_instance,
            );
        }
    }

    pub fn cmd_bind_vertex_buffers(
        &self,
        command_buffer: vk::CommandBuffer,
        first_binding: u32,
        buffers: &[vk::Buffer],
        offsets: &[vk::DeviceSize],
    ) {
        unsafe {
            self.raw
                .cmd_bind_vertex_buffers(command_buffer, first_binding, buffers, offsets);
        }
    }

    pub fn cmd_bind_index_buffer(
        &self,
        command_buffer: vk::CommandBuffer,
        buffer: vk::Buffer,
        offset: vk::DeviceSize,
        index_type: vk::IndexType,
    ) {
        unsafe {
            self.raw
                .cmd_bind_index_buffer(command_buffer, buffer, offset, index_type);
        }
    }

    pub fn cmd_bind_descriptor_sets(
        &self,
        command_buffer: vk::CommandBuffer,
        pipeline_bind_point: vk::PipelineBindPoint,
        layout: vk::PipelineLayout,
        first_set: u32,
        descriptor_sets: &[vk::DescriptorSet],
        dynamic_offsets: &[u32],
    ) {
        unsafe {
            self.raw.cmd_bind_descriptor_sets(
                command_buffer,
                pipeline_bind_point,
                layout,
                first_set,
                descriptor_sets,
                dynamic_offsets,
            );
        }
    }

    pub fn cmd_copy_buffer(
        &self,
        command_buffer: vk::CommandBuffer,
        src_buffer: vk::Buffer,
        dst_buffer: vk::Buffer,
        regions: &[vk::BufferCopy],
    ) {
        unsafe {
            self.raw
                .cmd_copy_buffer(command_buffer, src_buffer, dst_buffer, regions);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn cmd_pipeline_barrier(
        &self,
        command_buffer: vk::CommandBuffer,
        src_stage_mask: vk::PipelineStageFlags,
        dst_stage_mask: vk::PipelineStageFlags,
        dependency_flags: vk::DependencyFlags,
        memory_barriers: &[vk::MemoryBarrier],
        buffer_memory_barriers: &[vk::BufferMemoryBarrier],
        image_memory_barriers: &[vk::ImageMemoryBarrier],
    ) {
        unsafe {
            self.raw.cmd_pipeline_barrier(
                command_buffer,
                src_stage_mask,
                dst_stage_mask,
                dependency_flags,
                memory_barriers,
                buffer_memory_barriers,
                image_memory_barriers,
            );
        }
    }

    pub fn cmd_copy_buffer_to_image(
        &self,
        command_buffer: vk::CommandBuffer,
        src_buffer: vk::Buffer,
        dst_image: vk::Image,
        dst_image_layout: vk::ImageLayout,
        regions: &[vk::BufferImageCopy],
    ) {
        unsafe {
            self.raw.cmd_copy_buffer_to_image(
                command_buffer,
                src_buffer,
                dst_image,
                dst_image_layout,
                regions,
            );
        }
    }

    pub fn create_semaphore(
        &self,
        create_info: &vk::SemaphoreCreateInfo,
    ) -> Result<vk::Semaphore, DeviceError> {
        Ok(unsafe { self.raw.create_semaphore(create_info, None)? })
    }

    pub fn destroy_semaphore(&self, semaphore: vk::Semaphore) {
        unsafe { self.raw.destroy_semaphore(semaphore, None) }
    }

    pub fn create_fence(
        &self,
        create_info: &vk::FenceCreateInfo,
    ) -> Result<vk::Fence, DeviceError> {
        Ok(unsafe { self.raw.create_fence(create_info, None)? })
    }

    pub fn destroy_fence(&self, fence: vk::Fence) {
        unsafe { self.raw.destroy_fence(fence, None) }
    }

    pub fn wait_for_fence(
        &self,
        fences: &[vk::Fence],
        wait_all: bool,
        timeout: u64,
    ) -> Result<(), DeviceError> {
        unsafe { self.raw.wait_for_fences(fences, wait_all, timeout)? };
        Ok(())
    }
    pub fn reset_fence(&self, fences: &[vk::Fence]) -> Result<(), DeviceError> {
        unsafe { self.raw.reset_fences(fences)? };
        Ok(())
    }

    pub unsafe fn set_object_name(
        &self,
        object_type: vk::ObjectType,
        object: impl vk::Handle,
        name: &str,
    ) {
        let debug_utils = match &self.debug_utils {
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
