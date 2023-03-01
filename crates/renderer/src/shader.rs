use std::ffi::CString;
use std::path::Path;

use spirq::ty::Type;
use spirq::{EntryPoint, ExecutionModel, ReflectConfig, Variable};
use typed_builder::TypedBuilder;

use rhi::types::Label;
use rhi::types_v2::{RHIPushConstantRange, RHIShaderCreateInfo};
use rhi::RHI;
use rhi_types::RHIShaderStageFlags;

use crate::RendererError;

pub struct Shader<R: RHI> {
    pub shader: R::Shader,
    pub entry_point: EntryPoint,
    pub stage: RHIShaderStageFlags,
    pub name: CString,
}

#[derive(Clone, TypedBuilder)]
pub struct ShaderDescriptor<'a> {
    pub label: Label<'a>,
    pub spv_bytes: &'a [u32],
    pub entry_name: &'a str,
}

impl<R: RHI> Shader<R> {
    pub fn new(rhi: &R, desc: &ShaderDescriptor) -> Result<Self, RendererError> {
        let create_info = RHIShaderCreateInfo {
            spv: desc.spv_bytes,
        };

        let shader = unsafe { rhi.create_shader_module(&create_info)? };
        let entry_point = ShaderUtil::reflect_entry_point(desc.entry_name, desc.spv_bytes)?;
        let stage = match entry_point.exec_model {
            ExecutionModel::Vertex => RHIShaderStageFlags::VERTEX,
            ExecutionModel::TessellationControl => RHIShaderStageFlags::TESSELLATION_CONTROL,
            ExecutionModel::TessellationEvaluation => RHIShaderStageFlags::TESSELLATION_EVALUATION,
            ExecutionModel::Geometry => RHIShaderStageFlags::GEOMETRY,
            ExecutionModel::Fragment => RHIShaderStageFlags::FRAGMENT,
            ExecutionModel::GLCompute => RHIShaderStageFlags::COMPUTE,
            _ => RHIShaderStageFlags::empty(),
        };
        log::debug!("shader module created.");
        Ok(Self {
            shader,
            entry_point,
            stage,
            name: CString::new(desc.entry_name).unwrap(),
        })
    }

    pub fn destroy(self, rhi: &R) {
        unsafe { rhi.destroy_shader_module(self.shader) }
    }

    pub fn get_push_constant_range(&self) -> Option<RHIPushConstantRange> {
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
            .map(|push_const| RHIPushConstantRange {
                stage_flags: self.stage,
                size: (push_const.end - push_const.start) as _,
                offset: push_const.start as _,
            })
    }
}

pub struct ShaderUtil;

impl ShaderUtil {
    pub unsafe fn load_pre_compiled_spv_bytes_from_name(shader_file_name: &str) -> Vec<u32> {
        let path = format!("{}/{}.spv", env!("OUT_DIR"), shader_file_name);
        log::debug!("load shader spv file from: {}", path);
        unsafe { Self::load_pre_compiled_spv_bytes_from_path(Path::new(&path)) }
    }

    pub unsafe fn load_pre_compiled_spv_bytes_from_path<P: AsRef<Path>>(path: P) -> Vec<u32> {
        use std::fs::File;
        use std::io::Read;
        let spv_file = File::open(path).unwrap();
        let bytes_code: Vec<u8> = spv_file.bytes().filter_map(|byte| byte.ok()).collect();
        let (_prefix, bytes, _suffix) = unsafe { bytes_code.align_to::<u32>() };
        bytes.into()
    }

    fn reflect_entry_point(entry_name: &str, spv: &[u32]) -> Result<EntryPoint, RendererError> {
        let entry_points = ReflectConfig::new()
            // Load SPIR-V data into `[u32]` buffer `spv_words`.
            .spv(spv)
            // Set this true if you want to reflect all resources no matter it's
            // used by an entry point or not.
            .ref_all_rscs(true)
            .reflect()?;
        // println!("{:#?}", &entry_points);
        let entry_point = entry_points
            .into_iter()
            .find(|entry_point| entry_point.name == entry_name)
            .ok_or_else(|| {
                log::error!("Entry point not found");
            })
            .unwrap();

        Ok(entry_point)
    }
}
