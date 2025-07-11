use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
};

use pyo3::{exceptions::PyRuntimeError, prelude::*};
use tauri_plugin_opener::{self as plugin};

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
            plugin::Error::Tauri(e) => TauriError::from(e).into(),
            plugin::Error::Io(e) => e.into(),
            e @ (
                plugin::Error::Json(_)
                | plugin::Error::UnknownProgramName(_)
                | plugin::Error::ForbiddenPath { .. }
                | plugin::Error::ForbiddenUrl { .. }
                | plugin::Error::UnsupportedPlatform
                | plugin::Error::NoParent(_)
                | plugin::Error::FailedToConvertPathToFileUrl
            ) => PyRuntimeError::new_err(e.to_string()),
            #[cfg(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            ))]
            e @ plugin::Error::Zbus(_)  => PyRuntimeError::new_err(e.to_string()),
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

/// See also: [tauri_plugin_opener::init]
#[pyfunction]
pub fn init() -> Plugin {
    Plugin::new(|| Box::new(plugin::init::<Runtime>()))
}

/// See also: [tauri_plugin_opener]
#[pymodule(submodule, gil_used = false)]
pub mod opener {
    #[pymodule_export]
    pub use super::init;
}
