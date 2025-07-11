use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
};

use pyo3::{exceptions::PyRuntimeError, prelude::*};
use tauri_plugin_fs::{self as plugin};

use crate::{ext_mod::plugin::Plugin, tauri_runtime::Runtime, utils::TauriError};

#[derive(Debug)]
struct PluginError(plugin::Error);

impl Display for PluginError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Error for PluginError {}

impl From<PluginError> for PyErr {
    fn from(value: PluginError) -> Self {
        match value.0 {
            plugin::Error::Json(e) => PyRuntimeError::new_err(e.to_string()),
            plugin::Error::Tauri(e) => TauriError::from(e).into(),
            plugin::Error::Io(e) => e.into(),
            plugin::Error::PathForbidden(path) => PyRuntimeError::new_err(("PathForbidden", path)),
            plugin::Error::GlobPattern(e) => PyRuntimeError::new_err(e.to_string()),
            plugin::Error::InvalidPathUrl => PyRuntimeError::new_err("InvalidPathUrl"),
            plugin::Error::UnsafePathBuf(msg) => PyRuntimeError::new_err(("UnsafePathBuf", msg)),
            non_exhaustive => PyRuntimeError::new_err(format!(
                "Unimplemented plugin error, please report this to the pytauri developers: {non_exhaustive}"
            )),
        }
    }
}

impl From<plugin::Error> for PluginError {
    fn from(value: plugin::Error) -> Self {
        Self(value)
    }
}

/// See also: [tauri_plugin_fs::init]
#[pyfunction]
pub fn init() -> Plugin {
    Plugin::new(|| Box::new(plugin::init::<Runtime>()))
}

/// See also: [tauri_plugin_fs]
#[pymodule(submodule, gil_used = false)]
pub mod fs {
    #[pymodule_export]
    pub use super::init;
}
