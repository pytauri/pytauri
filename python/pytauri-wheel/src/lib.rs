use std::{
    borrow::Cow,
    fs,
    path::{Path, PathBuf},
};

use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyDict, PyTuple},
};
use pytauri_core::{tauri_runtime::Runtime, utils::TauriError};
use tauri::{
    image::Image,
    ipc::RuntimeCapability,
    utils::{
        self as tauri_utils,
        acl::{
            build::parse_capabilities,
            capability::{Capability, CapabilityFile},
        },
        assets::{AssetKey, AssetsIter, CspHash},
        config::{CapabilityEntry, FrontendDist},
        platform::Target,
    },
    Assets, Config,
};

const CAPABILITIES_FOLDER: &str = "capabilities";

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

/// [CapabilityFile] does not implement [RuntimeCapability], so we need to wrap it.
struct RuntimeCapabilityFile(CapabilityFile);

impl RuntimeCapability for RuntimeCapabilityFile {
    fn build(self) -> CapabilityFile {
        self.0
    }
}

/// ref: <https://github.com/tauri-apps/tauri/blob/339a075e33292dab67766d56a8b988e46640f490/crates/tauri-codegen/src/context.rs#L508-L522>
fn find_icon(
    config: &Config,
    config_parent: &Path,
    predicate: impl Fn(&&String) -> bool,
    default: &str,
) -> Option<tauri::Result<Image<'static>>> {
    let icon_path = config.bundle.icon.iter().find(predicate);

    if let Some(icon_path) = icon_path {
        return Some(Image::from_path(config_parent.join(icon_path)));
    }

    let icon_path = config_parent.join(default);
    if icon_path.exists() {
        return Some(Image::from_path(icon_path));
    }

    None
}

/// ref: <https://github.com/tauri-apps/tauri/blob/339a075e33292dab67766d56a8b988e46640f490/crates/tauri-codegen/src/context.rs#L211-L244>
fn load_default_window_icon(
    config: &Config,
    config_parent: &Path,
    target: Target,
) -> Option<PyResult<Image<'static>>> {
    let icon = match target {
        Target::Windows => {
            // handle default window icons for Windows targets
            find_icon(
                config,
                config_parent,
                |i| i.ends_with(".ico"),
                "icons/icon.ico",
            )
            .or_else(|| {
                find_icon(
                    config,
                    config_parent,
                    |i| i.ends_with(".png"),
                    "icons/icon.png",
                )
            })
        }
        _ => {
            // handle default window icons for Unix targets
            find_icon(
                config,
                config_parent,
                |i| i.ends_with(".png"),
                "icons/icon.png",
            )
        }
    };

    icon.map(|icon| icon.map_err(TauriError::from).map_err(Into::into))
}

/// `def context_factory(src_tauri_dir: Path, /, *) -> tauri.Context`:
///
/// - `src_tauri_dir` should be absolute path.
//
// TODO: better error handling
pub fn context_factory(
    args: &Bound<'_, PyTuple>,
    _kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<tauri::Context> {
    let mut ctx = tauri_generate_context();
    let target = Target::current();

    // TODO, PERF: avoid cloning the `PathBuf` data.
    let (src_tauri_dir,): (PathBuf,) = args.extract()?;

    // Load config from file dynamically.
    // TODO: unify the error type
    // ref: <https://github.com/tauri-apps/tauri/blob/339a075e33292dab67766d56a8b988e46640f490/crates/tauri-codegen/src/lib.rs#L57-L99>
    let config = tauri_utils::config::parse::read_from(target, src_tauri_dir.clone())
        .map_err(|e| PyValueError::new_err(e.to_string()))?
        .0;
    let config: Config =
        serde_json::from_value(config).map_err(|e| PyValueError::new_err(e.to_string()))?;
    // NOTE: modify the `config` field first, because following code will use it.
    *ctx.config_mut() = config;

    // Patch `package_info` from `config`.
    // ref: <https://github.com/tauri-apps/tauri/blob/339a075e33292dab67766d56a8b988e46640f490/crates/tauri-codegen/src/context.rs#L268-L287>
    if let Some(product_name) = &ctx.config().product_name {
        ctx.package_info_mut().name = product_name.clone();
    }
    if let Some(version) = &ctx.config().version {
        ctx.package_info_mut().version = version.parse().unwrap();
    }

    // Supply custom Assets from disk dynamically.
    // ref: <https://github.com/tauri-apps/tauri/blob/339a075e33292dab67766d56a8b988e46640f490/crates/tauri-codegen/src/context.rs#L176-L207>
    if let Some(frontend_dist) = &ctx.config().build.frontend_dist {
        match frontend_dist {
            FrontendDist::Url(_) => {
                // do nothing, we don't need supply custom Assets for URL frontend_dist,
                // because tauri will fetch the frontend from the URL.
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

    // Load capabilities from disk dynamically.
    // ref: <https://github.com/tauri-apps/tauri/blob/339a075e33292dab67766d56a8b988e46640f490/crates/tauri-build/src/acl.rs#L402-L407>
    let capabilities_pattern_path = src_tauri_dir
        // i.e., `cpabilities/**/*`
        .join(format!("{}/**/*", CAPABILITIES_FOLDER));
    let capabilities_pattern = capabilities_pattern_path.to_str().ok_or_else(|| {
        PyValueError::new_err(format!(
            "`{}` is not is valid unicode",
            capabilities_pattern_path.display()
        ))
    })?;
    let mut capabilities_from_files = parse_capabilities(capabilities_pattern)
        // TODO: unify the error type
        .map_err(|e| PyValueError::new_err(e.to_string()))?;

    // Patch `capabilities` from `config`.
    // ref: <https://github.com/tauri-apps/tauri/blob/339a075e33292dab67766d56a8b988e46640f490/crates/tauri-codegen/src/context.rs#L388-L416>
    //      <https://tauri.app/security/capabilities/>
    let capabilities: Vec<Capability> = if ctx.config().app.security.capabilities.is_empty() {
        capabilities_from_files.into_values().collect()
    } else {
        let mut capabilities = Vec::new();
        for capability_entry in &ctx.config().app.security.capabilities {
            match capability_entry {
                CapabilityEntry::Inlined(capability) => {
                    capabilities.push(capability.clone());
                }
                CapabilityEntry::Reference(id) => {
                    let capability = capabilities_from_files.remove(id).ok_or_else(|| {
                        PyValueError::new_err(format!("capability with identifier {id} not found"))
                    })?;
                    capabilities.push(capability);
                }
            }
        }
        capabilities
    };

    // Add capabilities to `ctx`.
    // TODO, FIXME: `runtime_authority_mut` currently is not public API,
    // see: <https://github.com/tauri-apps/tauri/issues/12968>
    ctx.runtime_authority_mut()
        .add_capability(RuntimeCapabilityFile(CapabilityFile::List(capabilities)))
        .map_err(TauriError::from)?;

    // Set default window icon.
    let default_window_icon = load_default_window_icon(ctx.config(), &src_tauri_dir, target);
    // NOTE: Even if `default_window_icon` is `None`, we should not call `set_default_window_icon(default_window_icon)`,
    // because we have bundled the `tauri-app` icon by default, and setting it to `None` will remove it.
    if let Some(icon) = default_window_icon {
        ctx.set_default_window_icon(Some(icon?));
    }

    // Set tray icon.
    // ref: <https://github.com/tauri-apps/tauri/blob/339a075e33292dab67766d56a8b988e46640f490/crates/tauri-codegen/src/context.rs#L289-L299>
    if target.is_desktop() {
        if let Some(tray) = &ctx.config().app.tray_icon {
            let tray_icon_icon_path = src_tauri_dir.join(&tray.icon_path);
            let icon = Image::from_path(tray_icon_icon_path).map_err(TauriError::from)?;
            ctx.set_tray_icon(Some(icon));
        }
    }

    // TODO: `Context::app_icon`

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
