use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
};

use pyo3::{exceptions::PyRuntimeError, prelude::*, types::PyDict};
use pyo3_utils::from_py_dict::{derive_from_py_dict, FromPyDict as _};
use tauri_plugin_window_state::{self as plugin};

use crate::{ext_mod::plugin::Plugin, utils::TauriError};

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
            e @ plugin::Error::SerdeJson(_) => PyRuntimeError::new_err(e.to_string()),
        }
    }
}

impl From<plugin::Error> for PluginError {
    fn from(value: plugin::Error) -> Self {
        Self(value)
    }
}

/// See also: [tauri_plugin_window_state::Builder]
#[non_exhaustive]
pub struct BuilderArgs {}

derive_from_py_dict!(BuilderArgs {});

impl BuilderArgs {
    fn from_kwargs(kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<Option<Self>> {
        kwargs.map(Self::from_py_dict).transpose()
    }

    fn apply_to_builder(self, builder: plugin::Builder) -> plugin::Builder {
        let Self {} = self;
        builder
    }
}

/// See also: [tauri_plugin_window_state::Builder]
#[pyclass(frozen)]
#[non_exhaustive]
pub struct Builder;

#[pymethods]
impl Builder {
    #[staticmethod]
    #[pyo3(signature = (**kwargs))]
    fn build(kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<Plugin> {
        let args = BuilderArgs::from_kwargs(kwargs)?;

        let mut builder = plugin::Builder::new();
        if let Some(args) = args {
            builder = args.apply_to_builder(builder);
        }

        let plugin = Plugin::new(Box::new(move || Box::new(builder.build())));
        Ok(plugin)
    }
}

/// See also: [tauri_plugin_window_state]
#[pymodule(submodule, gil_used = false)]
pub mod window_state {
    #[pymodule_export]
    pub use super::Builder;

    pub use super::BuilderArgs;
}
