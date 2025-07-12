pub(crate) mod app;
pub(crate) mod app_handle;
pub(crate) mod assets;
pub(crate) mod context;
pub(crate) mod emitter;
pub(crate) mod event;
pub(crate) mod listener;
pub(crate) mod manager;
pub(crate) mod rect;
pub(crate) mod run_event;
pub(crate) mod theme;
pub(crate) mod url;

use pyo3::{exceptions::PyRuntimeError, prelude::*};

pub use tauri::{RESTART_EXIT_CODE, VERSION};

/// See also: [tauri::is_dev]
pub const IS_DEV: bool = tauri::is_dev();

/// See also: [tauri::webview_version]
#[pyfunction]
pub fn webview_version() -> PyResult<String> {
    // TODO, FIXME: unify the `wry` error type.
    tauri::webview_version().map_err(|e| PyRuntimeError::new_err(e.to_string()))
}
