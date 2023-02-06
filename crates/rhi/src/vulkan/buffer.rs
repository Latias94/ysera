use alloc::rc::Rc;
use std::mem::size_of;

use ash::vk;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator};
use gpu_allocator::MemoryLocation;
use parking_lot::Mutex;
use typed_builder::TypedBuilder;

use crate::vulkan::command_buffer_allocator::CommandBufferAllocator;
use crate::vulkan::device::Device;
use crate::DeviceError;

#[derive(Clone)]
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
pub struct StagingBufferDescriptor<'a, T> {
    pub label: crate::Label<'a>,
    pub device: &'a Rc<Device>,
    pub allocator: Rc<Mutex<Allocator>>,
    pub elements: &'a [T],
    pub command_buffer_allocator: &'a CommandBufferAllocator,
}

#[derive(Clone, TypedBuilder)]
pub struct UniformBufferDescriptor<'a, T> {
    pub label: crate::Label<'a>,
    pub device: &'a Rc<Device>,
    pub allocator: Rc<Mutex<Allocator>>,
    pub elements: &'a [T],
    pub buffer_type: BufferType,
    pub command_buffer_allocator: &'a CommandBufferAllocator,
}

impl Buffer {
    pub fn raw(&self) -> vk::Buffer {
        self.raw
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
                allocation_scheme: AllocationScheme::GpuAllocatorManaged,
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

    // https://developer.nvidia.com/vulkan-memory-management
    // recommend that you also store multiple buffers, like the vertex and index buffer,
    // into a single vk::Buffer and use offsets in commands like cmd_bind_vertex_buffers.
    pub fn new_staging_buffer<T>(desc: &StagingBufferDescriptor<T>) -> Result<Buffer, DeviceError> {
        let staging_buffer_desc = BufferDescriptor {
            label: Some("Staging Buffer"),
            device: desc.device,
            allocator: desc.allocator.clone(),
            element_size: size_of::<T>(),
            element_count: desc.elements.len() as u32,
            buffer_usage: vk::BufferUsageFlags::TRANSFER_SRC,
            memory_location: MemoryLocation::CpuToGpu,
        };
        let mut staging_buffer = Self::new(staging_buffer_desc)?;
        staging_buffer.copy_memory(desc.elements);
        Ok(staging_buffer)
    }

    pub fn new_buffer_copy_from_staging_buffer<T>(
        desc: &StagingBufferDescriptor<T>,
        buffer_type: BufferType,
    ) -> Result<Buffer, DeviceError> {
        let staging_buffer = Self::new_staging_buffer(desc)?;

        let buffer_desc = BufferDescriptor {
            label: desc.label,
            device: desc.device,
            allocator: desc.allocator.clone(),
            element_size: size_of::<T>(),
            element_count: desc.elements.len() as u32,
            buffer_usage: vk::BufferUsageFlags::TRANSFER_DST | buffer_type.to_buffer_usage(),
            memory_location: MemoryLocation::GpuOnly,
        };
        let buffer = Self::new(buffer_desc)?;
        staging_buffer.copy_buffer(&buffer, desc.command_buffer_allocator)?;
        Ok(buffer)
    }

    pub fn new_uniform_buffer<T>(desc: &UniformBufferDescriptor<T>) -> Result<Buffer, DeviceError> {
        let buffer_desc = BufferDescriptor {
            label: Some("Uniform Buffer"),
            device: desc.device,
            allocator: desc.allocator.clone(),
            element_size: size_of::<T>(),
            element_count: desc.elements.len() as u32,
            buffer_usage: vk::BufferUsageFlags::UNIFORM_BUFFER,
            memory_location: MemoryLocation::CpuToGpu,
        };
        let buffer = Self::new(buffer_desc)?;
        Ok(buffer)
    }

    pub fn copy_memory<T>(&mut self, data: &[T]) {
        if let Some(allocation) = &self.allocation {
            let dst = allocation.mapped_ptr().unwrap().cast().as_ptr();
            unsafe {
                use std::ptr::copy_nonoverlapping as memcpy;
                memcpy(data.as_ptr(), dst, data.len());
            }
        }
    }

    pub fn copy_buffer(
        &self,
        destination: &Buffer,
        command_buffer_allocator: &CommandBufferAllocator,
    ) -> Result<(), DeviceError> {
        command_buffer_allocator.create_single_use(|device, command_buffer| {
            let regions = [vk::BufferCopy::builder().size(self.buffer_size).build()];
            device.cmd_copy_buffer(command_buffer.raw(), self.raw, destination.raw, &regions);
        })?;
        Ok(())
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
