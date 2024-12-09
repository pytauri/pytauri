use std::{env, ffi::OsStr, fs::{self}, io::Cursor, path::{Path, PathBuf}, process::Command};
use anyhow::{bail, Result};
use flate2::read::GzDecoder;

use serde::Deserialize;
use tar::Archive;

#[derive(Clone, Debug, Deserialize)]
pub struct DownloadUrlResponse {
    pub version: u32,
    pub tag: String,
    pub release_url: String,
    pub asset_url_prefix: String
}

pub fn get_cargo_toml_path() -> Result<PathBuf> {
    let mut current_dir = env::current_dir()?;
    while if let Some(_) = current_dir.parent() { true } else { false } {
        for entry in current_dir.read_dir()? {
            match entry {
                Ok(entry) if entry.path().file_name() == Some(OsStr::new("Cargo.toml")) => return Ok(entry.path()),
                _ => (),
            }
        }

        current_dir.pop();
    }

    bail!("Could not find Cargo.toml");
}

pub fn download_standalone(python_version: &str, target_triple: &str, output_dir: &impl AsRef<Path>) -> Result<()> {
    let body = async {
        if output_dir.as_ref().exists() { return Ok(()) }

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

        fs::rename(out_dir, output_dir)?;

        Ok(())
    };

    return tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed building the Runtime")
        .block_on(body);
}

pub fn install_venv(output_dir: &impl AsRef<Path>) -> Result<()> {
    Command::new("uv")
        .arg("venv")
        .arg("--relocatable")
        .arg(output_dir.as_ref())
        .status()?;

    Ok(())
}
