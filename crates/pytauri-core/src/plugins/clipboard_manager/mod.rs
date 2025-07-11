use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
};

use pyo3::{exceptions::PyRuntimeError, prelude::*};
use tauri_plugin_clipboard_manager::{self as plugin};

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
            plugin::Error::Clipboard(e) => PyRuntimeError::new_err(e),
            plugin::Error::Tauri(e) => TauriError::from(e).into(),
        }
    }
}

impl From<plugin::Error> for PluginError {
    fn from(value: plugin::Error) -> Self {
        Self(value)
    }
}

/// See also: [tauri_plugin_clipboard_manager::init]
#[pyfunction]
pub fn init() -> Plugin {
    Plugin::new(|| Box::new(plugin::init::<Runtime>()))
}

/// See also: [tauri_plugin_clipboard_manager]
#[pymodule(submodule, gil_used = false)]
pub mod clipboard_manager {
    #[pymodule_export]
    pub use super::init;
}
