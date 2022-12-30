use anyhow::{bail, Context, Result};
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use glob::glob;
use naga::front::glsl::Options;
use naga::front::glsl::Parser;
use rayon::prelude::*;

use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() -> Result<()> {
    compile_shaders()
}

// macro_rules! p {
//     ($($tokens: tt)*) => {
//         println!("cargo:warning={}", format!($($tokens)*))
//     }
// }

pub fn load_shader(src_path: PathBuf) -> Result<()> {
    let name = src_path.file_name().unwrap().to_str().unwrap();
    let extension = src_path
        .extension()
        .context("File has no extension")?
        .to_str()
        .context("Extension cannot be converted to &str")?;
    let kind = match extension {
        "vert" => naga::ShaderStage::Vertex,
        "frag" => naga::ShaderStage::Fragment,
        "comp" => naga::ShaderStage::Compute,
        _ => bail!("Unsupported shader: {}", src_path.display()),
    };

    let src = fs::read_to_string(src_path.clone())?;

    let output_name = format!("{}/{}", env::var("OUT_DIR")?, &name);
    let output_name_ext = format!("{}.spv", &output_name);
    let spv_path = Path::new(&output_name_ext);
    // let wgsl_path = src_path.with_extension(format!("{}.wgsl", extension));

    let mut parser = Parser::default();
    let options = Options::from(kind);
    let module = match parser.parse(&options, &src) {
        Ok(it) => it,
        Err(errors) => {
            bail!(
                "Failed to compile shader: {}\nErrors:\n{:#?}",
                src_path.display(),
                errors
            );
        }
    };

    let flags = naga::valid::ValidationFlags::all();
    let info =
        naga::valid::Validator::new(flags, naga::valid::Capabilities::empty()).validate(&module)?;
    // std::fs::write(
    //     wgsl_path,
    //     wgsl::write_string(&module, &info, wgsl::WriterFlags::all())?,
    // )?;
    let spv = naga::back::spv::write_vec(
        &module,
        &info,
        &naga::back::spv::Options {
            flags: naga::back::spv::WriterFlags::empty(),
            ..naga::back::spv::Options::default()
        },
        None,
    )?;
    // from naga-cli
    let bytes = spv
        .iter()
        .fold(Vec::with_capacity(spv.len() * 4), |mut v, w| {
            v.extend_from_slice(&w.to_le_bytes());
            v
        });

    // p!("output spv to {:?}", &spv_path);
    fs::write(spv_path, bytes)?;

    Ok(())
}

fn compile_shaders() -> Result<()> {
    println!("Compiling shaders");
    // recompile shader if shader code changes
    let shader_dir_path = get_shader_source_dir_path();
    println!(
        "cargo:rerun-if-changed={}",
        shader_dir_path.to_str().unwrap()
    );
    let shader_paths = {
        let mut data = Vec::new();
        data.extend(glob("../../resources/shaders/**/*.vert")?);
        data.extend(glob("../../resources/shaders/**/*.frag")?);
        data.extend(glob("../../resources/shaders/**/*.comp")?);
        data
    };
    shader_paths
        .into_par_iter()
        .map(|glob_result| load_shader(glob_result?))
        .collect::<Vec<Result<_>>>()
        .into_iter()
        .collect::<Result<Vec<_>>>()?;

    let out_dir = env::var("OUT_DIR")?;
    let mut copy_options = CopyOptions::new();
    let mut paths_to_copy = Vec::new();
    copy_options.overwrite = true;
    paths_to_copy.push("../../resources/shaders/");
    copy_items(&paths_to_copy, out_dir, &copy_options)?;

    Ok(())
}

fn get_shader_source_dir_path() -> PathBuf {
    let path = get_root_path()
        .join("../..")
        .join("resources")
        .join("shaders");
    println!("Shader source directory: {:?}", path.as_os_str());
    path
}

fn get_root_path() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}
