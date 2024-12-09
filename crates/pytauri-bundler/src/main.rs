use std::{collections::HashMap, process::Command};
use anyhow::Result;
use run_script::ScriptOptions;
use serde::Serialize;
use pytauri_bundler::{download_standalone, install_venv};
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

    download_standalone(&python_version, &target_triple, &python_out_path)?;

    let mut venv_path = out_dir.clone();
    venv_path.push("venv");

    if !venv_path.exists() {
        install_venv(&venv_path).unwrap();
    }


    let mut options = ScriptOptions::new();
    options.working_directory = Some(env::current_dir()?);

    let (code, output, error) = run_script::run_script!(format!(r#"
        {}/bin/python {}/venv/bin/activate_this.py
        uv pip install setuptools setuptools-rust setuptools-scm
        uv sync
    "#, python_out_path.display(), out_dir.display()), options).unwrap();

    println!("Exit Code: {}", code);
    println!("Output: {}", output);
    println!("Error: {}", error);

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

    let pyembed_location = python_out_path.canonicalize().unwrap().display().to_string();
    let venv_location = venv_path.canonicalize().unwrap().display().to_string();

    config.bundle.resources.insert(pyembed_location, "pyembed".to_string());
    config.bundle.resources.insert("python".into(), "python".into());
    config.bundle.resources.insert(venv_location, "venv".into());

    let json_config = serde_json::to_string(&config)?;

    Command::new("cargo-tauri")
        .args(["build", "--config", &json_config])
        .status()?;

    Ok(())
}
