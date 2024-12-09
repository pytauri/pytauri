use std::{collections::HashMap, fs, process::Command};
use anyhow::Result;
use serde::Serialize;
use pytauri_bundler::download_standalone;
use std::env;

fn main() -> Result<()> {
    let out_dir = env::temp_dir();

    let python_version = match env::var("PYTAURI_PYTHON_VERSION") {
        Ok(version) => version,
        _ => "3.10.16".to_string()
    };

    let target_triple = env!("TARGET");

    let mut python_out_path = out_dir.clone();
    python_out_path.push("pyembed");

    download_standalone(&python_version, &target_triple, python_out_path.clone())?;

    let location = python_out_path.canonicalize().unwrap().display().to_string();

    #[derive(Serialize)]
    struct BundleConfig {
       active: bool,
       targets: String,
       resources: HashMap<String, String>
    }

    #[derive(Serialize)]
    struct Config {
       bundle: BundleConfig
    }

    let mut config = Config {
        bundle: BundleConfig {
            active: true,
            targets: "all".to_string(),
            resources: HashMap::new()
        }
    };

    config.bundle.resources.insert(location, "pyembed".to_string());
    config.bundle.resources.insert("python".into(), "python".into());

    let json_config = serde_json::to_string(&config)?;

    Command::new("cargo-tauri")
        .args(["build", "--config", &json_config])
        .status()?;

    Ok(())
}
