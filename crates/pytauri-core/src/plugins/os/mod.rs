use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
};

use pyo3::prelude::*;
use tauri_plugin_os::{self as plugin};

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
        // Since tauri_plugin_os::Error is an empty enum,
        // this match is unreachable but we keep it for consistency
        match value.0 {}
    }
}

impl From<plugin::Error> for PluginError {
    fn from(value: plugin::Error) -> Self {
        Self(value)
    }
}

/// See also: [tauri_plugin_os::init]
#[pyfunction]
pub fn init() -> Plugin {
    Plugin::new(Box::new(|| Box::new(plugin::init::<Runtime>())))
}

/// See also: [tauri_plugin_os]
#[pymodule(submodule, gil_used = false)]
pub mod os {
    #[pymodule_export]
    pub use super::init;
}
