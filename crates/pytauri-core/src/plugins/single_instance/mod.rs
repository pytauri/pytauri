use pyo3::prelude::*;
use tauri_plugin_single_instance::{self as plugin};

use crate::{
    ext_mod::{plugin::Plugin, PyAppHandleExt as _},
    tauri_runtime::Runtime,
    utils::PyResultExt as _,
};

/// See also: [tauri_plugin_single_instance::init]
#[pyfunction]
#[pyo3(signature = (callback, /))]
pub fn init(callback: Option<PyObject>) -> PyResult<Plugin> {
    let plugin = Plugin::new(Box::new(move || {
        Box::new(match callback {
            Some(callback) => plugin::init::<Runtime, _>(move |app_handle, args, cwd| {
                Python::with_gil(|py| {
                    let callback = callback.bind(py);
                    let result = callback.call1((app_handle.py_app_handle(), args, cwd));
                    result.unwrap_unraisable_py_result(py, Some(callback), || {
                        "Python exception occurred in `tauri_plugin_single_instance::init` callback"
                    });
                })
            }),
            None => plugin::init::<Runtime, _>(|_, _, _| {}),
        })
    }));

    Ok(plugin)
}

/// See also: [tauri_plugin_single_instance]
#[pymodule(submodule, gil_used = false)]
pub mod single_instance {
    #[pymodule_export]
    pub use super::init;
}
