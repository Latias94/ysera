use std::borrow::Cow;
use std::mem::size_of;
use std::path::Path;
use std::rc::Rc;

use ash::vk;
use typed_builder::TypedBuilder;

use math::{Vec3, Vertex3D};

use crate::vulkan::device::Device;
use crate::{Label, ShaderError};

pub struct Shader {
    device: Rc<Device>,
    vert_shader: vk::ShaderModule,
    vert_entry_name: String,
    frag_shader: vk::ShaderModule,
    frag_entry_name: String,
}

#[derive(Clone, TypedBuilder)]
pub struct ShaderDescriptor<'a> {
    pub label: Label<'a>,
    pub device: &'a Rc<Device>,
    pub vert_bytes: &'a [u32],
    pub vert_entry_name: &'a str,
    pub frag_bytes: &'a [u32],
    pub frag_entry_name: &'a str,
}

impl Shader {
    pub fn vert_shader_module(&self) -> vk::ShaderModule {
        self.vert_shader
    }

    pub fn frag_shader_module(&self) -> vk::ShaderModule {
        self.frag_shader
    }

    pub fn vert_entry_name(&self) -> &str {
        self.vert_entry_name.as_str()
    }

    pub fn frag_entry_name(&self) -> &str {
        self.frag_entry_name.as_str()
    }

    pub fn new(desc: &ShaderDescriptor) -> Result<Self, ShaderError> {
        let vert_shader = Self::create_shader_module(desc.label, desc.device, desc.vert_bytes)?;
        let frag_shader = Self::create_shader_module(desc.label, desc.device, desc.frag_bytes)?;
        log::debug!("shader module created.");

        Ok(Self {
            device: desc.device.clone(),
            vert_shader,
            frag_shader,
            vert_entry_name: desc.vert_entry_name.to_string(),
            frag_entry_name: desc.frag_entry_name.to_string(),
        })
    }

    pub fn create_shader_module(
        label: Label,
        device: &Rc<Device>,
        bytes: &[u32],
    ) -> Result<vk::ShaderModule, ShaderError> {
        let spv = Cow::Borrowed(bytes);
        let vk_info = vk::ShaderModuleCreateInfo::builder()
            .flags(vk::ShaderModuleCreateFlags::empty())
            .code(&spv);
        let raw = device.create_shader_module(&vk_info)?;
        if let Some(label) = label {
            unsafe { device.set_object_name(vk::ObjectType::SHADER_MODULE, raw, label) };
        }
        Ok(raw)
    }

    pub fn get_binding_description(&self) -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<Vertex3D>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()
    }

    // todo: reflect shader
    pub fn get_attribute_descriptions(&self) -> [vk::VertexInputAttributeDescription; 2] {
        let pos = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(0)
            .build();
        let color = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(1)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(size_of::<Vec3>() as u32)
            .build();
        [pos, color]
    }

    pub fn load_pre_compiled_spv_bytes_from_name(shader_file_name: &str) -> Vec<u32> {
        let path = format!("{}/{}.spv", env!("OUT_DIR"), shader_file_name);
        log::debug!("load shader spv file from: {}", path);
        Self::load_pre_compiled_spv_bytes_from_path(Path::new(&path))
    }

    pub fn load_pre_compiled_spv_bytes_from_path<P: AsRef<Path>>(path: P) -> Vec<u32> {
        use std::fs::File;
        use std::io::Read;
        let spv_file = File::open(path).unwrap();
        let bytes_code: Vec<u8> = spv_file.bytes().filter_map(|byte| byte.ok()).collect();
        let (_prefix, bytes, _suffix) = unsafe { bytes_code.align_to::<u32>() };
        bytes.into()
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        self.device.destroy_shader_module(self.vert_shader);
        self.device.destroy_shader_module(self.frag_shader);
        log::debug!("shader module destroyed.");
    }
}
