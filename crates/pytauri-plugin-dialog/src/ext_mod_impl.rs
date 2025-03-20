use std::error::Error;
use std::fmt::{Debug, Display};

use pyo3::prelude::*;
use pyo3_utils::py_wrapper::{PyWrapper, PyWrapperSemverExt as _, PyWrapperT2};
use pytauri_core::{ext_mod::ImplManager, tauri_runtime::Runtime};
use tauri::AppHandle;
use tauri_plugin_dialog::{self as plugin, MessageDialogButtons, MessageDialogKind};

#[derive(Debug)]
struct PluginError(plugin::Error);

impl Display for PluginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

impl Error for PluginError {}

impl From<PluginError> for PyErr {
    fn from(value: PluginError) -> Self {
        match value.0 {
            plugin::Error::Io(e) => e.into(),
            _ => panic!("unexpected plugin error"),
        }
    }
}

impl From<plugin::Error> for PluginError {
    fn from(value: plugin::Error) -> Self {
        Self(value)
    }
}
#[pyclass(frozen, eq, eq_int)]
#[derive(PartialEq, Clone)]
#[non_exhaustive]
enum PyMessageDialogKind {
    Info,
    Warning,
    Error,
}

impl From<PyMessageDialogKind> for plugin::MessageDialogKind {
    fn from(value: PyMessageDialogKind) -> Self {
        todo!()
    }
}

use pyo3::types::PyString;

#[pyclass(frozen)]
enum PyMessageDialogButtons {
    Ok(),
    OkCancel(),
    YesNo(),
    OkCustom(Py<PyString>),
    OkCancelCustom(Py<PyString>, Py<PyString>),
}

impl PyMessageDialogButtons {
    fn to_tauri(py: Python<'_>) -> plugin::MessageDialogButtons {
        todo!()
    }
}

#[pyclass(frozen)]
#[non_exhaustive]
pub struct MessageDialogBuilder(pub PyWrapper<PyWrapperT2<plugin::MessageDialogBuilder<Runtime>>>);

impl MessageDialogBuilder {
    fn new(builder: plugin::MessageDialogBuilder<Runtime>) -> Self {
        Self(PyWrapper::new2(builder))
    }
}

#[pymethods]
impl MessageDialogBuilder {
    fn blocking_show(&self, py: Python<'_>) -> PyResult<bool> {
        py.allow_threads(|| {
            let builder = self.0.try_take_inner()??;
            builder.blocking_show()
        })
    }

    /// callback: Callable[[bool], object]
    fn show(&self, py: Python<'_>, callback: PyObject) -> PyResult<()> {
        py.allow_threads(|| {
            let builder = self.0.try_take_inner()??;
            builder.show(|ok_or_no| todo!("callback(ok_or_no)"))
        })
    }
}

#[pyclass(frozen)]
#[non_exhaustive]
pub struct DialogExt;

pub type ImplDialogExt = ImplManager;

#[pymethods]
impl DialogExt {
    #[staticmethod]
    #[pyo3(signature = (
        slf,
        message,
        title=None,
        buttons=None,
        kind=None
    ))]
    fn message(
        slf: ImplDialogExt,
        message: String,
        title: Option<String>,
        buttons: Option<Py<PyMessageDialogButtons>>,
        kind: Option<PyMessageDialogKind>,
    ) -> PyResult<MessageDialogBuilder> {
        todo!()
    }
}
