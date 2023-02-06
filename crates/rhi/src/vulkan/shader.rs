use crate::vulkan::device::Device;
use crate::{Label, ShaderError};
use ash::vk;
use math::{Vec3, Vertex3D};
use spirq::ty::Type;
use spirq::{EntryPoint, ReflectConfig, Variable};
use std::borrow::Cow;
use std::ffi::CString;
use std::mem::size_of;
use std::path::Path;
use std::sync::Arc;
use typed_builder::TypedBuilder;

pub struct Shader {
    device: Arc<Device>,
    shader: vk::ShaderModule,
    entry_point: EntryPoint,
    name: CString,
    stage: vk::ShaderStageFlags,
}

#[derive(Clone, TypedBuilder)]
pub struct ShaderDescriptor<'a> {
    pub label: Label<'a>,
    pub device: &'a Arc<Device>,
    pub spv_bytes: &'a [u32],
    pub entry_name: &'a str,
}

pub trait ShaderPropertyInfo {
    fn get_binding_descriptions() -> Vec<vk::VertexInputBindingDescription>;
    fn get_attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription>;
}

impl Shader {
    pub fn shader_module(&self) -> vk::ShaderModule {
        self.shader
    }

    pub fn entry_name(&self) -> &str {
        self.entry_point.name.as_str()
    }

    pub fn name(&self) -> &CString {
        &self.name
    }

    pub fn stage(&self) -> vk::ShaderStageFlags {
        self.stage
    }

    pub fn new(desc: &ShaderDescriptor, stage: vk::ShaderStageFlags) -> Result<Self, ShaderError> {
        let shader = Self::create_shader_module(desc.label, desc.device, desc.spv_bytes)?;

        let entry_point = Self::reflect_entry_point(desc.entry_name, desc.spv_bytes);
        log::debug!("shader module created.");
        Ok(Self {
            device: desc.device.clone(),
            shader,
            entry_point,
            stage,
            name: CString::new(desc.entry_name).unwrap(),
        })
    }

    pub fn new_vert(desc: &ShaderDescriptor) -> Result<Self, ShaderError> {
        Self::new(desc, vk::ShaderStageFlags::VERTEX)
    }

    pub fn new_frag(desc: &ShaderDescriptor) -> Result<Self, ShaderError> {
        Self::new(desc, vk::ShaderStageFlags::FRAGMENT)
    }

    fn reflect_entry_point(entry_name: &str, spv: &[u32]) -> EntryPoint {
        let entry_points = ReflectConfig::new()
            // Load SPIR-V data into `[u32]` buffer `spv_words`.
            .spv(spv)
            // Set this true if you want to reflect all resources no matter it's
            // used by an entry point or not.
            .ref_all_rscs(true)
            .reflect()
            .map_err(|_| {
                log::error!("Unable to reflect spirv");
            })
            .unwrap();
        // println!("{:#?}", &entry_points);
        let entry_point = entry_points
            .into_iter()
            .find(|entry_point| entry_point.name == entry_name)
            .ok_or_else(|| {
                log::error!("Entry point not found");
            })
            .unwrap();

        entry_point
    }

    pub fn create_shader_module(
        label: Label,
        device: &Device,
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

    pub fn get_push_constant_range(&self) -> Option<vk::PushConstantRange> {
        self.entry_point
            .vars
            .iter()
            .filter_map(|var| match var {
                Variable::PushConstant {
                    ty: Type::Struct(ty),
                    ..
                } => Some(ty.members.clone()),
                _ => None,
            })
            .flatten()
            .map(|push_const| {
                push_const.offset..push_const.offset + push_const.ty.nbyte().unwrap_or_default()
            })
            .reduce(|a, b| a.start.min(b.start)..a.end.max(b.end))
            .map(|push_const| vk::PushConstantRange {
                stage_flags: self.stage,
                size: (push_const.end - push_const.start) as _,
                offset: push_const.start as _,
            })
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        self.device.destroy_shader_module(self.shader);
        log::debug!("shader module destroyed.");
    }
}

impl ShaderPropertyInfo for Vertex3D {
    // todo vertex layout
    fn get_binding_descriptions() -> Vec<vk::VertexInputBindingDescription> {
        let desc = vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<Vertex3D>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build();
        vec![desc]
    }

    fn get_attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription> {
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
        let tex_coord = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(2)
            .format(vk::Format::R32G32_SFLOAT)
            .offset((size_of::<Vec3>() + size_of::<Vec3>()) as u32)
            .build();
        vec![pos, color, tex_coord]
    }
}
