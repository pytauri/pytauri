use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pytauri_core::ext_mod::ipc::Invoke;
use pytauri_core::tauri_runtime::Runtime as PyTauriRuntime;
use tauri::ipc;

type IpcInvoke = ipc::Invoke<PyTauriRuntime>;

use crate::gil_runtime::task_with_gil;
use crate::PyInvokeHandlerExt as _;

fn pyfunc(invoke: IpcInvoke) {
    task_with_gil(move |py| {
        let py_invoke_handler = invoke
            .message
            .webview_ref()
            .try_py_invoke_handler()
            // it's ok to `unwrap` here, because the plugin is already initialized
            .unwrap()
            .bind(py)
            .clone();

        let invoke = match Invoke::new(py, invoke) {
            Some(invoke) => invoke,
            None => return, // the ipc has already been handled and rejected
        };

        // NOTE: We require that the implementation of `py_invoke_handler`
        // does not block for a long time, so this call will not block
        // the tokio runtime.
        if let Err(e) = py_invoke_handler.call1((invoke,)) {
            let new_err = PyRuntimeError::new_err("`py_invoke_handler` raised an exception");
            new_err.set_cause(py, Some(e));
            // TODO: use [pytauri_core::utils::PyResultExt::unwrap_unraisable_py_result]
            new_err.write_unraisable(py, Some(&py_invoke_handler));
            // TODO: use `log` instead of `panic!`,
            // it's because the joinhandle will never be awaited
            panic!("`py_invoke_handler` shouldn't raise exception");
        }
    });
}

pub(crate) fn invoke_handler(invoke: IpcInvoke) -> bool {
    match invoke.message.command() {
        "pyfunc" => {
            pyfunc(invoke);
            true
        }
        _ => false,
    }
}
