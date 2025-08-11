use pyo3::prelude::*;
use tauri_plugin_positioner::{self as plugin};

use crate::{ext_mod::plugin::Plugin, tauri_runtime::Runtime};

/// See also: [tauri_plugin_positioner::init]
#[pyfunction]
pub fn init() -> Plugin {
    Plugin::new(Box::new(|| Box::new(plugin::init::<Runtime>())))
}

/// See also: [tauri_plugin_positioner]
#[pymodule(submodule, gil_used = false)]
pub mod positioner {
    #[pymodule_export]
    pub use super::init;
}
