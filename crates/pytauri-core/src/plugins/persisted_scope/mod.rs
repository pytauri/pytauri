use pyo3::prelude::*;
use tauri_plugin_persisted_scope::{self as plugin};

use crate::{ext_mod::plugin::Plugin, tauri_runtime::Runtime};

/// See also: [tauri_plugin_persisted_scope::init]
#[pyfunction]
pub fn init() -> Plugin {
    Plugin::new(Box::new(|| Box::new(plugin::init::<Runtime>())))
}

/// See also: [tauri_plugin_persisted_scope]
#[pymodule(submodule, gil_used = false)]
pub mod persisted_scope {
    #[pymodule_export]
    pub use super::init;
}
