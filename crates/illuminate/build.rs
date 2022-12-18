use std::{
    env,
    ffi::OsStr,
    fs,
    io::Result,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn main() {
    compile_shaders();
}

fn compile_shaders() {
    println!("Compiling shaders");
    // recompile shader if shader code changes
    let shader_dir_path = get_shader_source_dir_path();
    println!(
        "cargo:rerun-if-changed={}",
        shader_dir_path.to_str().unwrap()
    );
    let path = get_glslang_exe_path();
    if let Some(glslang_validator_exe_path) = path {
        fs::read_dir(shader_dir_path.clone())
            .unwrap()
            .map(Result::unwrap)
            .filter(|dir| dir.file_type().unwrap().is_file())
            .filter(|dir| dir.path().extension() != Some(OsStr::new("spv")))
            .for_each(|dir| {
                let path = dir.path();
                let name = path.file_name().unwrap().to_str().unwrap();
                let output_name = format!("{}/{}.spv", env::var("OUT_DIR").unwrap(), &name);
                println!("Found file {:?}.\nCompiling...", path.as_os_str());

                let result = dbg!(Command::new(glslang_validator_exe_path.as_os_str())
                    .current_dir(&shader_dir_path)
                    .arg("-V")
                    .arg(&path)
                    .arg("-o")
                    .arg(output_name))
                .output();

                handle_program_result(result);
            })
    }
}

fn get_shader_source_dir_path() -> PathBuf {
    let path = get_root_path().join("../..").join("resources").join("shaders");
    println!("Shader source directory: {:?}", path.as_os_str());
    path
}

fn get_root_path() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

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

fn handle_program_result(result: Result<Output>) {
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
