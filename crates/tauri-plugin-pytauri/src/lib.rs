//! This crate is currently only used internally by pytauri to
//! implement IPC communication between the frontend and Python.

mod commands;
mod gil_runtime;

use std::error::Error;
use std::fmt::Display;
use std::ops::Deref;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pytauri_core::tauri_runtime::Runtime as PyTauriRuntime;
use tauri::plugin::{Builder, TauriPlugin};
use tauri::{Manager, Runtime};

use crate::commands::invoke_handler;

const PLUGIN_NAME: &str = "pytauri";

type PyInvokeHandlerType = PyObject;

struct PyInvokeHandler(PyInvokeHandlerType);

impl Deref for PyInvokeHandler {
    type Target = PyInvokeHandlerType;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PyInvokeHandler {
    fn new(handler: PyInvokeHandlerType) -> Self {
        Self(handler)
    }
}

/// Initializes the plugin.
///
/// The `py_invoke_handler` should have the following signature:
///
/// > def py_invoke_handler(invoke: [Invoke][pytauri_core::ext_mod::ipc::Invoke], /) -> None:
/// >     ...
///
/// It will be stored in the tauri app state and used to handle ipc requests from the frontend.
/// You can get its reference through [PyInvokeHandlerExt].
///
/// # NOTE:
///
/// - `py_invoke_handler` will be called in a tokio runtime, so it must not block for a long time.
///     - `tokio runtime` means it is running on an external thread.
/// - `py_invoke_handler` must not raise exceptions, otherwise it will result in logical undefined behavior.
pub fn init(py_invoke_handler: PyInvokeHandlerType) -> TauriPlugin<PyTauriRuntime> {
    Builder::<PyTauriRuntime>::new(PLUGIN_NAME)
        .invoke_handler(invoke_handler)
        .setup(|app_handle, _plugin_api| {
            // if false, there has already state set for the app instance.
            if !app_handle.manage(PyInvokeHandler::new(py_invoke_handler)) {
                unreachable!(
                    "`PyInvokeHandler` is private, so it is impossible for other crates to manage it"
                )
            }
            Ok(())
        })
        .build()
}

mod sealed {
    use super::*;

    pub trait SealedTrait<R> {}

    impl<R: Runtime, T: Manager<R>> SealedTrait<R> for T {}
}

/// This error indicates that the extension is not initialized
#[derive(Debug)]
pub struct PyInvokeHandlerStateError;

impl Display for PyInvokeHandlerStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to get `PyInvokeHandler` from state, maybe `{}` is not initialized",
            env!("CARGO_PKG_NAME")
        )
    }
}

impl Error for PyInvokeHandlerStateError {}

impl From<PyInvokeHandlerStateError> for PyErr {
    fn from(value: PyInvokeHandlerStateError) -> Self {
        PyRuntimeError::new_err(value.to_string())
    }
}

pub type PyInvokeHandlerStateResult<T> = Result<T, PyInvokeHandlerStateError>;

/// Gets the `py_invoke_handler` passed in when initializing the plugin.
pub trait PyInvokeHandlerExt<R: Runtime>: Manager<R> + sealed::SealedTrait<R> {
    fn try_py_invoke_handler(
        &self,
    ) -> PyInvokeHandlerStateResult<impl Deref<Target = PyInvokeHandlerType>> {
        self.try_state::<PyInvokeHandler>()
            .map(|state| state.inner().deref())
            .ok_or(PyInvokeHandlerStateError)
    }

    /// # Panic
    ///
    /// If [PyInvokeHandlerExt::try_py_invoke_handler] returns an error.
    fn py_invoke_handler(&self) -> impl Deref<Target = PyInvokeHandlerType> {
        self.try_py_invoke_handler().unwrap()
    }
}

impl<R: Runtime, T: Manager<R>> PyInvokeHandlerExt<R> for T {}
