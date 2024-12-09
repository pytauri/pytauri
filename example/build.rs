use std::{collections::HashMap, env, path::PathBuf};

use anyhow::Result;
use pytauri_bundler::download_standalone;
use serde::Serialize;
use serde_json::json;


macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}


fn main() -> Result<()> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    let python_version = match env::var("PYTAURI_PYTHON_VERSION") {
        Ok(version) => version,
        _ => "3.10.16".to_string()
    };

    let target_triple = env::var("TARGET")?;

    let mut out_path = out_dir.clone();
    out_path.push("pyembed");

    download_standalone(&python_version, &target_triple, out_path.clone())?;

    let location = out_path.canonicalize().unwrap().display().to_string();

    let value = json!({
        "bundle": {
            "active": true,
            "targets": "all",
            "resources": {
                location: "pyembed"
            }
        },
    });

    let json = value.to_string();

    p!("json: {:?}", json);

    println!("cargo:rustc-env=TAURI_CONFIG={}", json);
    env::set_var("TAURI_CONFIG", json);

    tauri_build::build();

    // panic!();

    Ok(())
}
