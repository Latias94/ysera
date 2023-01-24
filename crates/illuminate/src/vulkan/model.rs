use crate::vulkan::command_buffer_allocator::CommandBufferAllocator;
use crate::vulkan::device::Device;
use crate::vulkan::texture::{VulkanTexture, VulkanTextureDescriptor};
use gpu_allocator::vulkan::Allocator;
use math::{vec2, vec3, Vertex3D};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

use crate::vulkan::adapter::Adapter;
use crate::vulkan::instance::Instance;
use ash::vk;
use std::rc::Rc;
use typed_builder::TypedBuilder;

pub struct Model {
    vertices: Vec<Vertex3D>,
    indices: Vec<u32>,
    texture: VulkanTexture,
}

#[derive(Clone, TypedBuilder)]
pub struct ModelDescriptor<'a> {
    pub file_name: &'a str,
    pub device: &'a Rc<Device>,
    pub allocator: Rc<Mutex<Allocator>>,
    pub command_buffer_allocator: &'a CommandBufferAllocator,
    pub adapter: Rc<Adapter>, // check mipmap format support
    pub instance: Rc<Instance>,
}

impl Model {
    pub fn vertices(&self) -> &[Vertex3D] {
        &self.vertices
    }

    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    pub fn texture(&self) -> &VulkanTexture {
        &self.texture
    }

    pub fn load_obj(desc: &ModelDescriptor) -> anyhow::Result<Self> {
        let format = vk::Format::R8G8B8A8_SRGB;

        let mut texture_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        texture_path.push(format!("../../resources/textures/{}.png", desc.file_name));

        let texture_desc = VulkanTextureDescriptor {
            adapter: &desc.adapter,
            instance: &desc.instance,
            device: desc.device,
            allocator: desc.allocator.clone(),
            command_buffer_allocator: desc.command_buffer_allocator,
            path: &texture_path,
            format,
            enable_mip_levels: true,
        };

        let texture = VulkanTexture::new(&texture_desc)?;

        let mut model_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        model_path.push(format!("../../resources/objs/{}.obj", desc.file_name));
        let mut reader = BufReader::new(File::open(model_path)?);

        let (models, _) = tobj::load_obj_buf(
            &mut reader,
            &tobj::LoadOptions {
                triangulate: true,
                ..Default::default()
            },
            |_| Ok(Default::default()),
        )?;

        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut unique_vertices = HashMap::new();
        for model in &models {
            for index in &model.mesh.indices {
                let pos_offset = (3 * index) as usize;
                let tex_coord_offset = (2 * index) as usize;

                let vertex = Vertex3D {
                    position: vec3(
                        model.mesh.positions[pos_offset],
                        model.mesh.positions[pos_offset + 1],
                        model.mesh.positions[pos_offset + 2],
                    ),
                    color: vec3(1.0, 1.0, 1.0),
                    // OBJ 格式假设一个坐标系，其中垂直坐标 0 表示图像的底部，但是我们以从上到下的方向将图像上传到 Vulkan，
                    // 其中 0 表示图像的顶部。通过翻转纹理坐标的垂直分量来解决这个问题：
                    tex_coord: vec2(
                        model.mesh.texcoords[tex_coord_offset],
                        1.0 - model.mesh.texcoords[tex_coord_offset + 1],
                    ),
                };

                if let Some(index) = unique_vertices.get(&vertex) {
                    indices.push(*index as u32);
                } else {
                    let index = vertices.len();
                    unique_vertices.insert(vertex, index);
                    vertices.push(vertex);
                    indices.push(index as u32);
                }
            }
        }

        log::debug!("ObjModel created.");
        Ok(Self {
            vertices,
            indices,
            texture,
        })
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        log::debug!("model destroyed.");
    }
}
