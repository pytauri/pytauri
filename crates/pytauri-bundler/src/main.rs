#![allow(dead_code)]


use pytauri_bundler::{get_cargo_toml_path, download_standalone};
use std::{env, fs::create_dir_all, error::Error};

fn main() -> Result<(), Box<dyn Error>> {
    let debug_mode = match env::var("TAURI_ENV_DEBUG") {
        Ok(val) if val == "true" => true,
        _ => false
    };

    let python_version = match env::var("PYTAURI_PYTHON_VERSION") {
        Ok(version) => version,
        _ => "3.10.16".to_string()
    };

    let target_triple = env::var("TAURI_ENV_TARGET_TRIPLE")?;
    let _arch = env::var("TAURI_ENV_ARCH")?;
    let platform = env::var("TAURI_ENV_PLATFORM")?;

    if debug_mode == true {
        panic!("Cannot bundle in debug mode")
    }

    if platform == "windows" || platform == "darwin" {
        let mut output_dir = get_cargo_toml_path()?;
        output_dir.push("resources");
        create_dir_all(&output_dir)?;
        output_dir.push(format!("cpython-{}", target_triple));

        download_standalone(&python_version, &target_triple, output_dir)?;
    }

    Ok(())
}
