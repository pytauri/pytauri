use std::{env, fs, path::{Path, PathBuf}};

use anyhow::Result;
use pytauri_bundler::download_standalone;
use tauri_utils::resources::ResourcePaths;

/// Copies resources to a path.
fn copy_resources(resources: ResourcePaths<'_>, path: &Path) -> Result<()> {
  let path = path.canonicalize()?;
  for resource in resources.iter() {
    let resource = resource?;

    println!("cargo:rerun-if-changed={}", resource.path().display());

    // avoid copying the resource if target is the same as source
    let src = resource.path().canonicalize()?;
    let target = path.join(resource.target());
    if src != target {
        copy_file(src, target)?;
    }
  }
  Ok(())
}

fn copy_file(from: impl AsRef<Path> + std::fmt::Debug, to: impl AsRef<Path> + std::fmt::Debug) -> Result<()> {
    println!("copying file from: {:?}", from);
    println!("copying file to: {:?}", to);
    let from = from.as_ref();
    let to = to.as_ref();
    if !from.exists() {
        return Err(anyhow::anyhow!("{:?} does not exist", from));
    }
    if !from.is_file() {
        return Err(anyhow::anyhow!("{:?} is not a file", from));
    }
    let dest_dir = to.parent().expect("No data in parent");
    fs::create_dir_all(dest_dir)?;
    fs::copy(from, to)?;
    Ok(())
}

fn main() -> Result<()> {
    // for var in env::vars() {
    //     println!("var: {:?}", var);
    // }

    // let manifest_directory = env::var("CARGO_MANIFEST_DIR")?;
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    let target_dir = out_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let python_version = match env::var("PYTAURI_PYTHON_VERSION") {
        Ok(version) => version,
        _ => "3.10.16".to_string()
    };

    let target_triple = env::var("TARGET")?;

    let mut out_path = out_dir.clone();
    out_path.push("pyembed");

    download_standalone(&python_version, &target_triple, out_path.clone())?;

    let binding = std::collections::HashMap::from_iter([
        (out_path.to_string_lossy().into_owned(), "pyembed".into()),
    ]);

    let resources = ResourcePaths::from_map(
      &binding,
      true,
    );

    copy_resources(resources, target_dir)?;

    tauri_build::build();

    // panic!();

    Ok(())
}
