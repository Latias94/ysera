use anyhow::{bail, Context, Result};
use naga::front::glsl::Options;
use naga::front::glsl::Parser;

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

    if name == "triangle_push_constant.frag" {
        // ban this shader temporarily
        return Ok(());
    }

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

// naga 对 glsl 部分的的语法不太支持，可能考虑到和其他着色器语言语法兼容的原因，还有部分是工作量太大。
// 因此这里 Windows 暂时用回 glslangValidator 来编译。
// 对于 naga 踩坑的的实践可以参考 bevy。
// https://github.com/gfx-rs/naga/issues/1012 不支持 sample2D 属性
// glsl fragment shader 里的 push constant 好像也不支持，可以用回 uniform buffer

// not window use naga
#[cfg(not(target_os = "windows"))]
fn compile_shaders() -> Result<()> {
    use fs_extra::copy_items;
    use fs_extra::dir::CopyOptions;
    use glob::glob;
    use rayon::prelude::*;

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

// window use glslangValidator.exe
#[cfg(target_os = "windows")]
fn compile_shaders() -> Result<()> {
    println!("Compiling shaders");
    // recompile shader if shader code changes
    let shader_dir_path = get_shader_source_dir_path();
    println!(
        "cargo:rerun-if-changed={}",
        shader_dir_path.to_str().unwrap()
    );
    let path = get_glslang_exe_path();
    if let Some(glslang_validator_exe_path) = path {
        fs::read_dir(shader_dir_path.clone())?
            .map(Result::unwrap)
            .filter(|dir| dir.file_type().unwrap().is_file())
            .filter(|dir| dir.path().extension() != Some(std::ffi::OsStr::new("spv")))
            .for_each(|dir| {
                let path = dir.path();
                let name = path.file_name().unwrap().to_str().unwrap();
                let output_name = format!("{}/{}.spv", env::var("OUT_DIR").unwrap(), &name);
                println!("Found file {:?}.\nCompiling...", path.as_os_str());

                let result = dbg!(std::process::Command::new(
                    glslang_validator_exe_path.as_os_str()
                )
                .current_dir(&shader_dir_path)
                .arg("-V")
                .arg(&path)
                .arg("-o")
                .arg(output_name))
                .output();

                handle_program_result(result);
            })
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn get_glslang_exe_path() -> Option<PathBuf> {
    let vulkan_sdk_dir = env!("VULKAN_SDK");

    let mut path = Path::new(vulkan_sdk_dir)
        .join("Bin")
        .join("glslangValidator.exe");
    if !path.exists() {
        path = Path::new(vulkan_sdk_dir)
            .join("Bin32")
            .join("glslangValidator.exe");
    }
    if !path.exists() {
        println!("Can not find Glslang executable!");
        None
    } else {
        println!("Glslang executable path: {:?}", path.as_os_str());
        Some(path)
    }
}

#[cfg(target_os = "windows")]
fn handle_program_result(result: std::io::Result<std::process::Output>) {
    match result {
        Ok(output) => {
            if output.status.success() {
                println!("Shader compilation succeeded.");
                print!(
                    "stdout: {}",
                    String::from_utf8(output.stdout)
                        .unwrap_or_else(|_| "Failed to print program stdout".to_string())
                );
            } else {
                eprintln!("Shader compilation failed. Status: {}", output.status);
                eprint!(
                    "stdout: {}",
                    String::from_utf8(output.stdout)
                        .unwrap_or_else(|_| "Failed to print program stdout".to_string())
                );
                eprint!(
                    "stderr: {}",
                    String::from_utf8(output.stderr)
                        .unwrap_or_else(|_| "Failed to print program stderr".to_string())
                );
                panic!("Shader compilation failed. Status: {}", output.status);
            }
        }
        Err(error) => {
            panic!("Failed to compile shader. Cause: {}", error);
        }
    }
}
