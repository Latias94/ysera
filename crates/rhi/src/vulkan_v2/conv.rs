use crate::{ImageFormat, PresentMode, VertexFormat};
use ash::vk;

pub fn map_texture_format(format: ImageFormat) -> vk::Format {
    match format {
        ImageFormat::Bgra8UnormSrgb => vk::Format::B8G8R8A8_SRGB,
    }
}

pub fn map_vk_surface_formats(sf: vk::SurfaceFormatKHR) -> Option<ImageFormat> {
    use ash::vk::Format as F;
    use ImageFormat as Tf;
    // https://vulkan.gpuinfo.org/listsurfaceformats.php
    Some(match sf.format {
        F::B8G8R8A8_SRGB => Tf::Bgra8UnormSrgb,
        _ => return None,
    })
}

pub fn map_vk_present_mode(mode: vk::PresentModeKHR) -> Option<PresentMode> {
    if mode == vk::PresentModeKHR::IMMEDIATE {
        Some(PresentMode::Immediate)
    } else if mode == vk::PresentModeKHR::MAILBOX {
        Some(PresentMode::Mailbox)
    } else if mode == vk::PresentModeKHR::FIFO {
        Some(PresentMode::Fifo)
    } else if mode == vk::PresentModeKHR::FIFO_RELAXED {
        //Some(PresentMode::Relaxed)
        None
    } else {
        log::warn!("Unrecognized present mode {:?}", mode);
        None
    }
}

pub fn map_vertex_format(format: VertexFormat) -> vk::Format {
    match format {
        VertexFormat::Float32x2 => vk::Format::R32G32_SFLOAT,
        VertexFormat::Float32x3 => vk::Format::R32G32B32_SFLOAT,
        VertexFormat::Depth32Float => vk::Format::D32_SFLOAT,
        VertexFormat::Depth32FloatStencil8 => vk::Format::D32_SFLOAT_S8_UINT,
        VertexFormat::Depth24UnormStencil8 => vk::Format::D24_UNORM_S8_UINT,
    }
}

pub fn map_vk_image_usage(usage: vk::ImageUsageFlags) -> crate::ImageUses {
    let mut bits = crate::ImageUses::empty();
    if usage.contains(vk::ImageUsageFlags::TRANSFER_SRC) {
        bits |= crate::ImageUses::COPY_SRC;
    }
    if usage.contains(vk::ImageUsageFlags::TRANSFER_DST) {
        bits |= crate::ImageUses::COPY_DST;
    }
    if usage.contains(vk::ImageUsageFlags::SAMPLED) {
        bits |= crate::ImageUses::RESOURCE;
    }
    if usage.contains(vk::ImageUsageFlags::COLOR_ATTACHMENT) {
        bits |= crate::ImageUses::COLOR_TARGET;
    }
    if usage.contains(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT) {
        bits |= crate::ImageUses::DEPTH_STENCIL_READ | crate::ImageUses::DEPTH_STENCIL_WRITE;
    }
    bits
}

pub fn map_image_usage(usage: crate::ImageUses) -> vk::ImageUsageFlags {
    let mut flags = vk::ImageUsageFlags::empty();
    if usage.contains(crate::ImageUses::COPY_SRC) {
        flags |= vk::ImageUsageFlags::TRANSFER_SRC;
    }
    if usage.contains(crate::ImageUses::COPY_DST) {
        flags |= vk::ImageUsageFlags::TRANSFER_DST;
    }
    if usage.contains(crate::ImageUses::RESOURCE) {
        flags |= vk::ImageUsageFlags::SAMPLED;
    }
    if usage.contains(crate::ImageUses::COLOR_TARGET) {
        flags |= vk::ImageUsageFlags::COLOR_ATTACHMENT;
    }
    if usage
        .intersects(crate::ImageUses::DEPTH_STENCIL_READ | crate::ImageUses::DEPTH_STENCIL_WRITE)
    {
        flags |= vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT;
    }

    flags
}

pub fn map_buffer_usage(usage: crate::BufferUses) -> vk::BufferUsageFlags {
    let mut flags = vk::BufferUsageFlags::empty();
    if usage.contains(crate::BufferUses::COPY_SRC) {
        flags |= vk::BufferUsageFlags::TRANSFER_SRC;
    }
    if usage.contains(crate::BufferUses::COPY_DST) {
        flags |= vk::BufferUsageFlags::TRANSFER_DST;
    }
    if usage.contains(crate::BufferUses::UNIFORM) {
        flags |= vk::BufferUsageFlags::UNIFORM_BUFFER;
    }
    if usage.contains(crate::BufferUses::INDEX) {
        flags |= vk::BufferUsageFlags::INDEX_BUFFER;
    }
    if usage.contains(crate::BufferUses::VERTEX) {
        flags |= vk::BufferUsageFlags::VERTEX_BUFFER;
    }
    flags
}

pub fn map_present_mode(mode: PresentMode) -> vk::PresentModeKHR {
    match mode {
        PresentMode::Immediate => vk::PresentModeKHR::IMMEDIATE,
        PresentMode::Mailbox => vk::PresentModeKHR::MAILBOX,
        PresentMode::Fifo => vk::PresentModeKHR::FIFO,
        PresentMode::FifoRelaxed => vk::PresentModeKHR::FIFO_RELAXED,
        PresentMode::AutoNoVsync | PresentMode::AutoVsync => {
            unreachable!("Cannot create swapchain with Auto PresentationMode")
        }
    }
}
