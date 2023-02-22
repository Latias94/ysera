use ash::vk;
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator};
use gpu_allocator::MemoryLocation;
use parking_lot::Mutex;

use rhi_types::RHIExtent2D;

use crate::RHIError;

pub unsafe fn create_image_view(
    device: &ash::Device,
    image: vk::Image,
    format: vk::Format,
    aspect_mask: vk::ImageAspectFlags,
    view_type: vk::ImageViewType,
    layer_count: u32,
    mip_levels: u32,
) -> Result<vk::ImageView, RHIError> {
    let range = vk::ImageSubresourceRange::builder()
        .aspect_mask(aspect_mask)
        .base_array_layer(0)
        .layer_count(layer_count)
        .base_mip_level(0)
        .level_count(mip_levels)
        .build();
    let info = vk::ImageViewCreateInfo::builder()
        .flags(vk::ImageViewCreateFlags::empty())
        .image(image)
        // 用于指定图像被看作是一维纹理、二维纹理、三维纹理还是立方体贴图
        .view_type(view_type)
        .format(format)
        // 指定图像的用途和图像的哪一部分可以被访问。
        // 如果编写 VR 一类的应用程序，可能会使用支持多个层次的交换链。这时，应该为每个图像创建多个图像视图，
        // 分别用来访问左眼和右眼两个不同的图层。
        .subresource_range(range)
        // 用于进行图像颜色通道的映射。比如，对于单色纹理，我们可以将所有颜色通道映射到红色通道。
        // 我们也可以直接将颜色通道的值映射为常数 0 或 1。在这里，我们只使用默认的映射。
        .components(vk::ComponentMapping {
            r: vk::ComponentSwizzle::IDENTITY,
            g: vk::ComponentSwizzle::IDENTITY,
            b: vk::ComponentSwizzle::IDENTITY,
            a: vk::ComponentSwizzle::IDENTITY,
        })
        .build();
    let raw = unsafe { device.create_image_view(&info, None)? };

    Ok(raw)
}

pub unsafe fn create_image(
    device: &ash::Device,
    allocator: &Mutex<Allocator>,
    extent: RHIExtent2D,
    format: vk::Format,
    image_tiling: vk::ImageTiling,
    image_usage_flags: vk::ImageUsageFlags,
    image_create_flags: vk::ImageCreateFlags,
    array_layers: u32,
    mip_levels: u32,
) -> Result<(vk::Image, Option<Allocation>), RHIError> {
    let create_info = vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::TYPE_2D)
        .extent(vk::Extent3D {
            width: extent.width,
            height: extent.height,
            depth: 1,
        })
        .flags(image_create_flags)
        .mip_levels(mip_levels)
        .array_layers(array_layers)
        .format(format)
        // vk::ImageTiling::LINEAR - Texels 是以行为主的顺序排列的，就像我们的像素阵列。平铺模式不能在以后的时间里改变。
        // 如果你想能够直接访问图像内存中的 texels，那么你必须使用 vk::ImageTiling::LINEAR。
        // vk::ImageTiling::OPTIMAL - Texels 是以执行已定义的顺序排列的，以达到最佳的访问效果。
        .tiling(image_tiling)
        // vk::ImageLayout::UNDEFINED - 不能被 GPU 使用，而且第一次转换会丢弃纹理。
        // vk::ImageLayout::PREINITIALIZED - 不能被 GPU 使用，但第一次转换将保留 texels。
        // 很少有情况下需要在第一次转换时保留 texels。然而，有一个例子是，如果你想把一个图像和 vk::ImageTiling::LINEAR
        // 布局结合起来作为一个暂存图像(staging image)。在这种情况下，你想把文本数据上传到它那里，
        // 然后在不丢失数据的情况下把图像转换为传输源。然而，在我们的例子中，我们首先要把图像转换为传输目的地，
        // 然后从一个缓冲区对象中复制 texel 数据到它，所以我们不需要这个属性，可以安全地使用 vk::ImageLayout::UNDEFINED。
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(image_usage_flags)
        // 稀疏图像是只有某些区域实际上由内存支持的图像。例如，如果您对体素地形使用3D纹理，那么您可以使用它来避免分配内存来存储大量“空气”值。
        .samples(vk::SampleCountFlags::TYPE_1)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);
    let raw = unsafe { device.create_image(&create_info, None)? };

    // 为图像分配内存的方式与为缓冲区分配内存的方式完全相同。只不过这里使用 get_image_memory_requirements 而不是
    // get_buffer_memory_requirements，使用 bind_image_memory 而不是 bind_buffer_memory。
    let requirements = unsafe { device.get_image_memory_requirements(raw) };

    let allocation = allocator.lock().allocate(&AllocationCreateDesc {
        name: "Image",
        requirements,
        location: MemoryLocation::GpuOnly,
        linear: true,
        allocation_scheme: AllocationScheme::GpuAllocatorManaged,
    })?;

    unsafe {
        device.bind_image_memory(raw, allocation.memory(), allocation.offset())?;
    }

    Ok((raw, Some(allocation)))
}
