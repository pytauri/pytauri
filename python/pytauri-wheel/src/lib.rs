use std::{borrow::Cow, fs, path::PathBuf};

use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyDict, PyTuple},
};
use pytauri_core::tauri_runtime::Runtime;
use tauri::utils::{
    self as tauri_utils,
    assets::{AssetKey, AssetsIter, CspHash},
    config::FrontendDist,
    platform::Target,
};
use tauri::Assets;

pub fn tauri_generate_context() -> tauri::Context {
    tauri::generate_context!()
}

/// A simple `Assets` implementation that reads files from disk directory.
struct DirAssets(PathBuf);

impl Assets<Runtime> for DirAssets {
    fn get(&self, key: &AssetKey) -> Option<Cow<'_, [u8]>> {
        // > refer to [tauri_utils::assets::AssetKey]
        // >
        // > - Has a root directory
        //
        // So we need to skip the first character (i.e., `/`) of the key.
        let path = self.0.join(&key.as_ref()[1..]);

        // TODO: return `None` only when not found, log::error!() in other cases
        fs::read(&path).ok().map(Cow::Owned)
    }

    fn csp_hashes(&self, _html_path: &AssetKey) -> Box<dyn Iterator<Item = CspHash<'_>> + '_> {
        unimplemented!()
    }

    fn iter(&self) -> Box<AssetsIter<'_>> {
        todo!("use `walkdir` crate to implement this")
    }
}

/// `def context_factory(src_tauri_dir: Path, /, *) -> tauri.Context`:
///
/// - `src_tauri_dir` should be absolute path.
pub fn context_factory(
    args: &Bound<'_, PyTuple>,
    _kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<tauri::Context> {
    let mut ctx = tauri_generate_context();

    // TODO, PERF: avoid cloning the `PathBuf` data.
    let (src_tauri_dir,): (PathBuf,) = args.extract()?;

    // Load config from file dynamically.
    // TODO: unify the error type
    // 👇 ref: <https://github.com/tauri-apps/tauri/blob/339a075e33292dab67766d56a8b988e46640f490/crates/tauri-codegen/src/lib.rs#L57-L99>
    let config = tauri_utils::config::parse::read_from(Target::current(), src_tauri_dir.clone())
        .map_err(|e| PyValueError::new_err(e.to_string()))?
        .0;
    let config: tauri::Config =
        serde_json::from_value(config).map_err(|e| PyValueError::new_err(e.to_string()))?;
    // 👆

    // Patch `package_info` from `config`.
    // 👇 ref: <https://github.com/tauri-apps/tauri/blob/339a075e33292dab67766d56a8b988e46640f490/crates/tauri-codegen/src/context.rs#L268-L287>
    if let Some(product_name) = &config.product_name {
        ctx.package_info_mut().name = product_name.clone();
    }
    if let Some(version) = &config.version {
        ctx.package_info_mut().version = version.parse().unwrap();
    }
    // 👆

    // Supply custom Assets from disk dynamically.
    // 👇 ref: <https://github.com/tauri-apps/tauri/blob/339a075e33292dab67766d56a8b988e46640f490/crates/tauri-codegen/src/context.rs#L176-L207>
    if let Some(frontend_dist) = &config.build.frontend_dist {
        match frontend_dist {
            FrontendDist::Url(_) => {
                // do nothing, we don't need supply custom Assets for URL frontend_dist
            }
            FrontendDist::Directory(dir) => {
                let abs_assert_dir = if dir.is_relative() {
                    src_tauri_dir.join(dir)
                } else {
                    dir.clone()
                };
                ctx.set_assets(Box::new(DirAssets(abs_assert_dir)));
            }
            FrontendDist::Files(_) => {
                return Err(PyValueError::new_err(
                    "frontend_dist: Files is not supported yet",
                ));
            }
            unknown => unimplemented!("unimplemented frontend_dist type: {:?}", unknown),
        }
    }
    // 👆

    *ctx.config_mut() = config;

    Ok(ctx)
}

pub fn builder_factory(
    _args: &Bound<'_, PyTuple>,
    _kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<tauri::Builder<Runtime>> {
    // TODO: more plugins
    let builder = tauri::Builder::default().plugin(tauri_plugin_opener::init());
    Ok(builder)
}

#[pymodule(gil_used = false)]
#[pyo3(name = "ext_mod")]
pub mod ext_mod {
    use super::*;

    #[pymodule_init]
    fn init(module: &Bound<'_, PyModule>) -> PyResult<()> {
        pytauri::pymodule_export(
            module,
            // i.e., `context_factory` function of python binding
            context_factory,
            // i.e., `builder_factory` function of python binding
            builder_factory,
        )
    }
}
