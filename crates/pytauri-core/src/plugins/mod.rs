#[cfg(feature = "plugin-autostart")]
mod autostart;
#[cfg(feature = "plugin-clipboard-manager")]
mod clipboard_manager;
#[cfg(feature = "plugin-deep-link")]
mod deep_link;
#[cfg(feature = "plugin-dialog")]
mod dialog;
#[cfg(feature = "plugin-fs")]
mod fs;
#[cfg(feature = "plugin-http")]
mod http;
#[cfg(feature = "plugin-notification")]
mod notification;
#[cfg(feature = "plugin-opener")]
mod opener;
#[cfg(feature = "plugin-os")]
mod os;

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

    /// Whether the `plugin-clipboard-manager` feature is enabled.
    #[pymodule_export]
    pub const PLUGIN_CLIPBOARD_MANAGER: bool = cfg!(feature = "plugin-clipboard-manager");

    /// Whether the `plugin-fs` feature is enabled.
    #[pymodule_export]
    pub const PLUGIN_FS: bool = cfg!(feature = "plugin-fs");

    /// Whether the `plugin-opener` feature is enabled.
    #[pymodule_export]
    pub const PLUGIN_OPENER: bool = cfg!(feature = "plugin-opener");

    /// Whether the `plugin-autostart` feature is enabled.
    #[pymodule_export]
    pub const PLUGIN_AUTOSTART: bool = cfg!(feature = "plugin-autostart");

    /// Whether the `plugin-deep-link` feature is enabled.
    #[pymodule_export]
    pub const PLUGIN_DEEP_LINK: bool = cfg!(feature = "plugin-deep-link");

    /// Whether the `plugin-http` feature is enabled.
    #[pymodule_export]
    pub const PLUGIN_HTTP: bool = cfg!(feature = "plugin-http");

    /// Whether the `plugin-os` feature is enabled.
    #[pymodule_export]
    pub const PLUGIN_OS: bool = cfg!(feature = "plugin-os");

    #[cfg(feature = "plugin-notification")]
    #[pymodule_export]
    pub use notification::notification;

    #[cfg(feature = "plugin-dialog")]
    #[pymodule_export]
    pub use dialog::dialog;

    #[cfg(feature = "plugin-clipboard-manager")]
    #[pymodule_export]
    pub use clipboard_manager::clipboard_manager;

    #[cfg(feature = "plugin-fs")]
    #[pymodule_export]
    pub use fs::fs;

    #[cfg(feature = "plugin-opener")]
    #[pymodule_export]
    pub use opener::opener;

    #[cfg(feature = "plugin-autostart")]
    #[pymodule_export]
    pub use autostart::autostart;

    #[cfg(feature = "plugin-deep-link")]
    #[pymodule_export]
    pub use deep_link::deep_link;

    #[cfg(feature = "plugin-http")]
    #[pymodule_export]
    pub use http::http;

    #[cfg(feature = "plugin-os")]
    #[pymodule_export]
    pub use os::os;
}
