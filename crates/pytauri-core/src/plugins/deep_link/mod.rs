use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
};

use pyo3::{exceptions::PyRuntimeError, prelude::*};
use tauri_plugin_deep_link::{self as plugin};

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
            e @ plugin::Error::UnsupportedPlatform => PyRuntimeError::new_err(e.to_string()),
            plugin::Error::Io(e) => e.into(),
            plugin::Error::Tauri(e) => TauriError::from(e).into(),
            #[cfg(target_os = "windows")]
            e @ plugin::Error::Windows(_) => PyRuntimeError::new_err(e.to_string()),
            #[cfg(target_os = "linux")]
            e @ (plugin::Error::Ini(_) | plugin::Error::ParseIni(_)) => {
                PyRuntimeError::new_err(e.to_string())
            }
            #[cfg(any(target_os = "ios", target_os = "android"))]
            e @ plugin::Error::PluginInvoke(_) => PyRuntimeError::new_err(e.to_string()),
        }
    }
}

impl From<plugin::Error> for PluginError {
    fn from(value: plugin::Error) -> Self {
        Self(value)
    }
}

/// See also: [tauri_plugin_deep_link::init]
#[pyfunction]
pub fn init() -> Plugin {
    Plugin::new(Box::new(|| Box::new(plugin::init::<Runtime>())))
}

/// See also: [tauri_plugin_deep_link]
#[pymodule(submodule, gil_used = false)]
pub mod deep_link {
    #[pymodule_export]
    pub use super::init;
}
