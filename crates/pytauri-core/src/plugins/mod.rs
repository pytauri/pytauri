#[cfg(feature = "plugin-dialog")]
mod dialog;
#[cfg(feature = "plugin-notification")]
mod notification;

use pyo3::prelude::*;

/// See also: [tauri-apps/plugins-workspace](https://github.com/tauri-apps/plugins-workspace)
///
/// You can access this module in Python via `pytuari.EXT_MOD.pytauri_plugins`.
#[pymodule(submodule, gil_used = false)]
pub mod pytauri_plugins {
    #[allow(unused_imports)] // if none of pymodule exported
    use super::*;

    /// Whether the `plugin-notification` feature is enabled.
    #[pymodule_export]
    pub const PLUGIN_NOTIFICATION: bool = cfg!(feature = "plugin-notification");

    /// Whether the `plugin-dialog` feature is enabled.
    #[pymodule_export]
    pub const PLUGIN_DIALOG: bool = cfg!(feature = "plugin-dialog");

    #[cfg(feature = "plugin-notification")]
    #[pymodule_export]
    pub use notification::notification;

    #[cfg(feature = "plugin-dialog")]
    #[pymodule_export]
    pub use dialog::dialog;
}
