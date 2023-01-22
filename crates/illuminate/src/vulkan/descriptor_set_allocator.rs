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
use crate::vulkan::uniform_buffer::UniformBufferObject;
use crate::DeviceError;

#[derive(TypedBuilder)]
pub struct DescriptorSetsCreateInfo<'a> {
    pub uniform_buffers: &'a [Buffer],
}

/// https://vulkan.gpuinfo.org/displaydevicelimit.php?name=maxBoundDescriptorSets&platform=windows
///
/// 描述符集编号 0 将用于引擎全局资源，并且每帧绑定一次。描述符集编号 1 将用于每次传递资源，并且每次传递绑定一次。
///
/// 描述符集编号 2 将用于材料资源，并且编号 3 将用于每个对象资源。这样，内部渲染循环将仅绑定描述符集 2 和 3，并且性能将很高。
pub struct DescriptorSetAllocator {
    pool: DescriptorPool,
    per_frame_layout: DescriptorSetLayout,
    device: Rc<Device>,
}

impl DescriptorSetAllocator {
    pub fn raw_per_frame_layout(&self) -> vk::DescriptorSetLayout {
        self.per_frame_layout.raw()
    }

    pub fn new(device: &Rc<Device>, swapchain_image_count: u32) -> Result<Self, DeviceError> {
        let pool_create_info = DescriptorPoolCreateInfo {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: swapchain_image_count,
            device,
            max_sets: swapchain_image_count,
        };
        let pool = DescriptorPool::new(pool_create_info)?;

        let descriptor_set_layout_binding_0 = DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            shader_stage_flags: vk::ShaderStageFlags::VERTEX,
        };
        let per_frame_layout_desc = DescriptorSetLayoutCreateInfo {
            device,
            bindings: &[descriptor_set_layout_binding_0],
        };
        let per_frame_layout = DescriptorSetLayout::new(per_frame_layout_desc)?;
        log::debug!("Descriptor Set Allocator created.");
        Ok(Self {
            device: device.clone(),
            pool,
            per_frame_layout,
        })
    }

    pub fn allocate_per_frame_descriptor_sets(
        &self,
        desc: &DescriptorSetsCreateInfo,
    ) -> Result<Vec<vk::DescriptorSet>, DeviceError> {
        log::debug!("Allocating per frame descriptor sets!");

        let count = desc.uniform_buffers.len();
        let layouts = vec![self.per_frame_layout.raw(); count];
        let info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.pool.raw())
            .set_layouts(&layouts);
        let descriptor_sets = self.device.allocate_descriptor_sets(&info)?;

        for i in 0..count {
            let info = vk::DescriptorBufferInfo::builder()
                .buffer(desc.uniform_buffers[i].raw())
                .offset(0)
                .range(size_of::<UniformBufferObject>() as u64)
                .build();
            let buffer_info = [info];
            let write_descriptor_set = vk::WriteDescriptorSet::builder()
                .dst_set(descriptor_sets[i])
                .dst_binding(0)
                // 描述符可以是数组，因此我们还需要指定要更新的数组中的第一个索引。我们没有使用数组，因此索引只是 0。
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                // buffer_info 字段用于引用缓冲区数据的描述符
                .buffer_info(&buffer_info)
                // image_info 用于引用图像数据的描述符，texel_buffer_view 用于引用缓冲区视图的描述符。
                .build();
            self.device
                .update_descriptor_sets(&[write_descriptor_set], &[] as &[vk::CopyDescriptorSet]);
        }
        log::debug!("Per frame descriptor sets Allocated.");

        Ok(descriptor_sets)
    }
}

impl Drop for DescriptorSetAllocator {
    fn drop(&mut self) {
        log::debug!("Descriptor Set Allocator destroyed.");
    }
}
