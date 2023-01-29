use alloc::rc::Rc;
use std::mem::size_of;

use ash::vk;
use typed_builder::TypedBuilder;

use crate::vulkan::buffer::Buffer;
use crate::vulkan::descriptor_pool::{DescriptorPool, DescriptorPoolCreateInfo};
use crate::vulkan::descriptor_set_layout::{
    DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateInfo,
};
use crate::vulkan::device::Device;
use crate::vulkan::texture::VulkanTexture;
use crate::vulkan::uniform_buffer::UniformBufferObject;
use crate::DeviceError;

#[derive(TypedBuilder)]
pub struct PerFrameDescriptorSetsCreateInfo<'a> {
    pub uniform_buffers: &'a [Buffer],
    pub texture_image_view: vk::ImageView,
    pub texture_sampler: vk::Sampler,
}

/// https://vulkan.gpuinfo.org/displaydevicelimit.php?name=maxBoundDescriptorSets&platform=windows
///
/// 描述符集编号 0 将用于引擎全局资源，并且每帧绑定一次。描述符集编号 1 将用于每次传递资源，并且每次传递绑定一次。
///
/// 描述符集编号 2 将用于材料资源，并且编号 3 将用于每个对象资源。这样，内部渲染循环将仅绑定描述符集 2 和 3，并且性能将很高。
pub struct DescriptorSetAllocator {
    device: Rc<Device>,
    per_frame_pool: DescriptorPool,
    texture_pool: DescriptorPool,
    per_frame_layout: DescriptorSetLayout,
    texture_layout: DescriptorSetLayout,
}

impl DescriptorSetAllocator {
    pub fn raw_per_frame_layout(&self) -> vk::DescriptorSetLayout {
        self.per_frame_layout.raw()
    }

    pub fn raw_texture_layout(&self) -> vk::DescriptorSetLayout {
        self.texture_layout.raw()
    }

    pub fn new(device: &Rc<Device>, swapchain_image_count: u32) -> Result<Self, DeviceError> {
        let per_frame_pool_create_info = DescriptorPoolCreateInfo {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: swapchain_image_count,
            device,
            max_sets: swapchain_image_count,
        };
        let per_frame_pool = DescriptorPool::new(per_frame_pool_create_info)?;

        let texture_pool = DescriptorPool::create_texture_descriptor_pool(device)?;

        let ubo_binding = DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            shader_stage_flags: vk::ShaderStageFlags::VERTEX,
        };

        let image_binding = DescriptorSetLayoutBinding {
            binding: 1,
            descriptor_type: vk::DescriptorType::SAMPLED_IMAGE,
            descriptor_count: 1,
            shader_stage_flags: vk::ShaderStageFlags::FRAGMENT,
        };

        let sampler_binding = DescriptorSetLayoutBinding {
            binding: 2,
            descriptor_type: vk::DescriptorType::SAMPLER,
            descriptor_count: 1,
            shader_stage_flags: vk::ShaderStageFlags::FRAGMENT,
        };

        let per_frame_layout_desc = DescriptorSetLayoutCreateInfo {
            device,
            bindings: &[ubo_binding, image_binding, sampler_binding],
        };

        let per_frame_layout = DescriptorSetLayout::new(per_frame_layout_desc)?;

        let texture_pool_ubo_binding = DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: 1,
            shader_stage_flags: vk::ShaderStageFlags::FRAGMENT,
        };
        let texture_layout_desc = DescriptorSetLayoutCreateInfo {
            device,
            bindings: &[texture_pool_ubo_binding],
        };
        let texture_layout = DescriptorSetLayout::new(texture_layout_desc)?;

        log::debug!("Descriptor Set Allocator created.");
        Ok(Self {
            device: device.clone(),
            per_frame_pool,
            texture_pool,
            per_frame_layout,
            texture_layout,
        })
    }

    pub fn allocate_per_frame_descriptor_sets(
        &self,
        desc: &PerFrameDescriptorSetsCreateInfo,
    ) -> Result<Vec<vk::DescriptorSet>, DeviceError> {
        log::debug!("Allocating per frame descriptor sets!");

        let count = desc.uniform_buffers.len();
        let layouts = vec![self.per_frame_layout.raw(); count];
        let info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.per_frame_pool.raw())
            .set_layouts(&layouts);
        let descriptor_sets = self.device.allocate_descriptor_sets(&info)?;

        for i in 0..count {
            // 将实际图像和采样器资源绑定到描述符集中的描述符
            let buffer_info = vk::DescriptorBufferInfo::builder()
                .buffer(desc.uniform_buffers[i].raw())
                .offset(0)
                .range(size_of::<UniformBufferObject>() as u64)
                .build();
            let buffer_infos = [buffer_info];
            let ubo_write = vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_sets[i])
                .dst_binding(0)
                // 描述符可以是数组，因此我们还需要指定要更新的数组中的第一个索引。我们没有使用数组，因此索引只是 0。
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                // buffer_info 字段用于引用缓冲区数据的描述符
                .buffer_info(&buffer_infos)
                // image_info 用于引用图像数据的描述符，texel_buffer_view 用于引用缓冲区视图的描述符。
                .build();

            // here use image+sampler cause naga not support sampler2D, or we can use only SAMPLED_IMAGE
            let image_info = vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(desc.texture_image_view)
                .build();

            let image_infos = &[image_info];
            let image_write = vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_sets[i])
                .dst_binding(1)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
                .image_info(image_infos)
                .build();

            let sampler_info = vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .sampler(desc.texture_sampler)
                .build();

            let sampler_infos = &[sampler_info];
            let sampler_write = vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_sets[i])
                .dst_binding(2)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::SAMPLER)
                .image_info(sampler_infos)
                .build();
            self.device
                .update_descriptor_sets(&[ubo_write, image_write, sampler_write], &[]);
        }
        log::debug!("Per frame descriptor sets Allocated.");

        Ok(descriptor_sets)
    }

    pub fn allocate_texture_descriptor_set(
        &self,
        texture: &VulkanTexture,
        image_layout: vk::ImageLayout,
    ) -> Result<vk::DescriptorSet, DeviceError> {
        let descriptor_set = {
            let layouts = [self.texture_layout.raw()];
            let allocate_info = vk::DescriptorSetAllocateInfo::builder()
                .descriptor_pool(self.texture_pool.raw())
                .set_layouts(&layouts);

            self.device.allocate_descriptor_sets(&allocate_info)?[0]
        };

        let image_info = vk::DescriptorImageInfo::builder()
            .image_layout(image_layout)
            .sampler(texture.raw_sampler())
            .image_view(texture.raw_image_view())
            .build();

        // here use image+sampler cause naga not support sampler2D, or we can use only SAMPLED_IMAGE
        let image_infos = &[image_info];
        let image_write = vk::WriteDescriptorSet::builder()
            .dst_set(descriptor_set)
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(image_infos)
            .build();
        self.device
            .update_descriptor_sets(&[image_write], &[]);
        Ok(descriptor_set)
    }

    pub fn free_texture_descriptor_set(
        &self,
        descriptor_set: vk::DescriptorSet,
    ) -> Result<(), DeviceError> {
        self.device
            .free_descriptor_sets(self.texture_pool.raw(), &[descriptor_set])
    }
}

impl Drop for DescriptorSetAllocator {
    fn drop(&mut self) {
        log::debug!("Descriptor Set Allocator destroyed.");
    }
}
