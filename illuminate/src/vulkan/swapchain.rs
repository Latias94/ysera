use crate::QueueFamilyIndices;
use ash::extensions::khr;
use std::ffi::c_char;

pub struct Swapchain {
    family_index: QueueFamilyIndices,
    queue_index: u32,
    enabled_extensions: Vec<*const c_char>,
}

impl Swapchain {
    pub fn create() -> Self {
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
        todo!();
    }
}
