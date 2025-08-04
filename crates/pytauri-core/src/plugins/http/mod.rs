use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
};

use pyo3::{exceptions::PyRuntimeError, prelude::*};
use tauri_plugin_http::{self as plugin};

use crate::{ext_mod::plugin::Plugin, pytauri_plugins, tauri_runtime::Runtime, utils::TauriError};

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
            plugin::Error::Tauri(e) => TauriError::from(e).into(),
            plugin::Error::Utf8(e) => e.into(),
            plugin::Error::FsError(e) => pytauri_plugins::fs::PluginError::from(e).into(),
            e @ (plugin::Error::Json(_)
            | plugin::Error::Network(_)
            | plugin::Error::Http(_)
            | plugin::Error::HttpInvalidHeaderName(_)
            | plugin::Error::HttpInvalidHeaderValue(_)
            | plugin::Error::UrlNotAllowed(_)
            | plugin::Error::UrlParseError(_)
            | plugin::Error::HttpMethod(_)
            | plugin::Error::SchemeNotSupport(_)
            | plugin::Error::RequestCanceled
            | plugin::Error::DataUrlError
            | plugin::Error::DataUrlDecodeError
            | plugin::Error::DangerousSettings) => PyRuntimeError::new_err(e.to_string()),
        }
    }
}

impl From<plugin::Error> for PluginError {
    fn from(value: plugin::Error) -> Self {
        Self(value)
    }
}

/// See also: [tauri_plugin_http::init]
#[pyfunction]
pub fn init() -> Plugin {
    Plugin::new(Box::new(|| Box::new(plugin::init::<Runtime>())))
}

/// See also: [tauri_plugin_http]
#[pymodule(submodule, gil_used = false)]
pub mod http {
    #[pymodule_export]
    pub use super::init;
}
