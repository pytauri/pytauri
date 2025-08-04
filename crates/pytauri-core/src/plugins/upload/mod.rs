use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
};

use pyo3::{exceptions::PyRuntimeError, prelude::*};
use tauri_plugin_upload::{self as plugin};

use crate::{ext_mod::plugin::Plugin, tauri_runtime::Runtime};

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
            plugin::Error::Io(e) => e.into(),
            e @ (plugin::Error::Request(_)
            | plugin::Error::ContentLength(_)
            | plugin::Error::HttpErrorCode(_, _)) => PyRuntimeError::new_err(e.to_string()),
        }
    }
}

impl From<plugin::Error> for PluginError {
    fn from(value: plugin::Error) -> Self {
        Self(value)
    }
}

/// See also: [tauri_plugin_upload::init]
#[pyfunction]
pub fn init() -> Plugin {
    Plugin::new(Box::new(|| Box::new(plugin::init::<Runtime>())))
}

/// See also: [tauri_plugin_upload]
#[pymodule(submodule, gil_used = false)]
pub mod upload {
    #[pymodule_export]
    pub use super::init;
}
