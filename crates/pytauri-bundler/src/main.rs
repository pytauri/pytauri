#![allow(dead_code)]
use std::{env, error::Error, fs::{self, create_dir_all}, io::Cursor};
use flate2::read::GzDecoder;

use serde::Deserialize;
use tar::Archive;

#[derive(Clone, Debug, Deserialize)]
struct DownloadUrlResponse {
    version: u32,
    tag: String,
    release_url: String,
    asset_url_prefix: String
}

async fn download_standalone(python_version: String, target_triple: String) -> Result<(), Box<dyn Error>> {
    let mut output_dir = env::current_dir()?;
    output_dir.push("resources");
    create_dir_all(&output_dir)?;
    output_dir.push(format!("cpython-{}", target_triple));

    if output_dir.exists() { return Ok(()) }

    let mut out_dir = env::temp_dir();
    out_dir.push("python-build-standalone");

    let download_url = "https://raw.githubusercontent.com/indygreg/python-build-standalone/latest-release/latest-release.json";
    let response: DownloadUrlResponse = reqwest::get(download_url).await?.json().await?;

    let file_name = format!("cpython-{}+{}-{}-install_only_stripped.tar.gz", python_version, response.tag, target_triple);

    let response = reqwest::get(format!("{}/{}", response.asset_url_prefix, file_name)).await?;
    let bytes = response.bytes().await?;

    let tar = GzDecoder::new(Cursor::new(bytes));
    let mut archive = Archive::new(tar);

    archive.unpack(&out_dir)?;

    out_dir.push("python");
    println!("out_dir: {:?}", out_dir);
    println!("output_dir: {:?}", output_dir);

    fs::rename(out_dir, output_dir)?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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
        download_standalone(python_version, target_triple).await?;
    }

    Ok(())
}
