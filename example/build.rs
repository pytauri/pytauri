use uv_cache::Cache;
use std::error::Error;
use uv_python::{PythonEnvironment, PythonRequest, EnvironmentPreference};
use uv_installer::SitePackages;

use directories::ProjectDirs;

pub fn uv_cache() -> Cache {
    ProjectDirs::from("", "", "uv").map_or_else(
        || Cache::from_path(".uv_cache"),
        |project_dirs| Cache::from_path(project_dirs.cache_dir()),
    )
}

/// try to find a `PythonEnvironment` based on Cache or currently active virtualenv (`VIRTUAL_ENV`).
pub fn uv_venv(maybe_cache: Option<Cache>) -> Result<(PythonEnvironment, Cache), Box<dyn Error>> {
    let cache = maybe_cache.unwrap_or_else(uv_cache);
    cache.venv_dir()?; // set up the cache

    let environ = PythonEnvironment::find(
        &PythonRequest::Any,                // just find me a python
        EnvironmentPreference::OnlyVirtual, // venv is always virtual
        &cache,
    )?;

    Ok((environ, cache))
}

fn main() -> Result<(), Box<dyn Error>> {
    let (environment, cache) = uv_venv(None)?;
    let site_packages = SitePackages::from_environment(&environment)?;
    for package in site_packages.iter() {
        match package.as_editable() {
            Some(package) => {
                println!("editable: {:?}", package.to_file_path().unwrap().display().to_string());
            },
            None => {
                println!("package: {:?}", package.path().to_string_lossy());
            },
        }
    }
    // panic!();
    tauri_build::build();

    Ok(())
}
