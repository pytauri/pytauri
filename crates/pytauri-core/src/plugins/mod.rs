#[cfg(feature = "plugin-notification")]
mod notification;

use pyo3::prelude::*;

#[pymodule(submodule, gil_used = false)]
pub mod pytauri_plugins {
    #[allow(unused_imports)] // if none of pymodule exported
    use super::*;

    #[cfg(feature = "plugin-notification")]
    #[pymodule_export]
    pub use notification::notification;
}
