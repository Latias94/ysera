use crate::DownlevelFlags;
use ash::extensions::khr;
use ash::vk;
use physical_device::PhysicalDeviceCapabilities;
use std::ffi::CStr;
use std::sync::Arc;

pub mod buffer;
pub mod conv;
pub mod debug;
pub mod device;
pub mod image;
pub mod instance;
pub mod physical_device;
pub mod pipeline;
pub mod platforms;
pub mod queue;
pub mod sampler;
pub mod semaphore;
pub mod shader;
pub mod swapchain;
pub mod utils;

#[derive(Clone)]
pub struct Api;

impl crate::GraphicsApi for Api {
    type Instance = Instance;
    type Surface = Surface;
    type PhysicalDevice = PhysicalDevice;
    type Device = Device;
    type Swapchain = Swapchain;
    type Semaphore = Semaphore;
    type Queue = Queue;
    type Buffer = Buffer;
    type Image = Image;
    type Sampler = Sampler;
    type Pipeline = Pipeline;
    type Shader = Shader;
}

pub struct InstanceShared {
    /// Loads instance level functions. Needs to outlive the Devices it has created.
    raw: ash::Instance,
    /// Loads the Vulkan library. Needs to outlive Instance and Device.
    entry: ash::Entry,
    extensions: Vec<&'static CStr>,
    debug_utils: Option<debug::DebugUtils>,
    flags: crate::InstanceFlags,
}

pub struct Instance {
    shared: Arc<InstanceShared>,
}

pub struct Surface {
    shared: Arc<SurfaceShared>,
    instance: Arc<InstanceShared>,
}

pub struct SurfaceShared {
    raw: vk::SurfaceKHR,
    fp: khr::Surface,
}

pub struct Swapchain {
    raw: vk::SwapchainKHR,
    loader: khr::Swapchain,
    surface: Arc<SurfaceShared>,
    device: Arc<DeviceShared>,
    fence: vk::Fence,
    images: Vec<vk::Image>,
    image_index: u32,
    config: crate::SurfaceConfiguration,
}

pub struct PhysicalDevice {
    raw: vk::PhysicalDevice,
    instance: Arc<InstanceShared>,
    surface: Option<Arc<SurfaceShared>>,
    known_memory_flags: vk::MemoryPropertyFlags,
    phd_capabilities: PhysicalDeviceCapabilities,
    downlevel_flags: DownlevelFlags,
}

pub struct DeviceShared {
    /// Loads device local functions.
    raw: ash::Device,
    family_index: u32,
    queue_index: u32,
    raw_queue: vk::Queue,
    instance: Arc<InstanceShared>,
    physical_device: vk::PhysicalDevice,
    vendor_id: u32,
    timestamp_period: f32,
    enabled_extensions: Vec<&'static CStr>,
    render_passes: Vec<vk::RenderPass>,
    framebuffers: Vec<vk::Framebuffer>,
}

pub struct Device {
    shared: Arc<DeviceShared>,
    valid_memory_types: u32,
    naga_options: naga::back::spv::Options,
}

pub struct Queue {
    raw: vk::Queue,
    swapchain_loader: khr::Swapchain,
    device: Arc<DeviceShared>,
    family_index: u32,
}

#[derive(Debug)]
pub struct Buffer {}

#[derive(Debug)]
pub struct Image {}

#[derive(Debug)]
pub struct Sampler {}

pub struct Semaphore {
    raw: vk::Semaphore,
    is_timeline: bool,
    device: Arc<DeviceShared>,
}

pub struct Pipeline {}

#[derive(Debug)]
pub struct Shader {}
