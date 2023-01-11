use alloc::rc::Rc;
use std::mem::size_of;
use std::ptr::copy_nonoverlapping as memcpy;

use ash::vk;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, Allocator};
use gpu_allocator::MemoryLocation;
use parking_lot::Mutex;
use typed_builder::TypedBuilder;

use crate::vulkan::device::Device;
use crate::DeviceError;

pub enum BufferType {
    Index = 0,
    Vertex = 1,
    Uniform = 2,
}

impl BufferType {
    pub fn to_buffer_usage(&self) -> vk::BufferUsageFlags {
        match self {
            BufferType::Index => vk::BufferUsageFlags::INDEX_BUFFER,
            BufferType::Vertex => vk::BufferUsageFlags::VERTEX_BUFFER,
            BufferType::Uniform => vk::BufferUsageFlags::UNIFORM_BUFFER,
        }
    }
}

pub struct Buffer {
    raw: vk::Buffer,
    device: Rc<Device>,
    allocator: Rc<Mutex<Allocator>>,
    allocation: Option<Allocation>,
    buffer_size: u64,
    element_size: usize,
    element_count: u32,
}

#[derive(Clone, TypedBuilder)]
pub struct BufferDescriptor<'a> {
    pub label: crate::Label<'a>,
    pub device: &'a Rc<Device>,
    pub allocator: Rc<Mutex<Allocator>>,
    pub element_size: usize,
    pub element_count: u32,
    pub buffer_usage: vk::BufferUsageFlags,
    pub memory_location: MemoryLocation,
}

#[derive(Clone, TypedBuilder)]
pub struct VertexBufferDescriptor<'a, T> {
    pub label: crate::Label<'a>,
    pub device: &'a Rc<Device>,
    pub allocator: Rc<Mutex<Allocator>>,
    pub elements: &'a [T],
}

impl Buffer {
    pub fn raw(&self) -> vk::Buffer {
        self.raw
    }

    pub fn new_vertex_buffer<T>(desc: VertexBufferDescriptor<T>) -> Result<Buffer, DeviceError> {
        let buffer_desc = BufferDescriptor {
            label: desc.label,
            device: desc.device,
            allocator: desc.allocator,
            element_size: size_of::<T>(),
            element_count: desc.elements.len() as u32,
            buffer_usage: vk::BufferUsageFlags::VERTEX_BUFFER,
            memory_location: MemoryLocation::CpuToGpu,
        };
        let vertex_buffer = Self::new(buffer_desc)?;
        vertex_buffer.copy_memory(desc.elements);
        Ok(vertex_buffer)
    }

    pub fn new(desc: BufferDescriptor) -> Result<Buffer, DeviceError> {
        let buffer_size = desc.element_count as u64 * desc.element_size as u64;
        let buffer_info = vk::BufferCreateInfo::builder()
            .size(buffer_size)
            .usage(desc.buffer_usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let device = desc.device;
        let raw = device.create_buffer(&buffer_info)?;

        let requirements = device.get_buffer_memory_requirements(raw);

        let allocator = desc.allocator.clone();
        let allocation = allocator
            .lock()
            .allocate(&AllocationCreateDesc {
                name: desc.label.unwrap(),
                requirements,
                location: desc.memory_location,
                linear: true,
            })
            .unwrap();

        unsafe { device.bind_buffer_memory(raw, allocation.memory(), allocation.offset())? }

        Ok(Self {
            raw,
            device: device.clone(),
            allocator,
            allocation: Some(allocation),
            element_size: desc.element_size,
            element_count: desc.element_count,
            buffer_size,
        })
    }

    pub fn copy_memory<T>(&self, data: &[T]) {
        if let Some(allocation) = &self.allocation {
            let dst = allocation.mapped_ptr().unwrap().cast().as_ptr();
            unsafe {
                memcpy(data.as_ptr(), dst, data.len());
            }
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        let allocation = self.allocation.take();
        if let Some(allocation) = allocation {
            self.allocator.lock().free(allocation).unwrap();
        }
        self.device.destroy_buffer(self.raw);
    }
}
