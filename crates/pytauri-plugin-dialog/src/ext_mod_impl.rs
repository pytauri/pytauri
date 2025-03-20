use std::error::Error;
use std::fmt::{Debug, Display};

use pyo3::prelude::*;
use pyo3_utils::py_wrapper::{PyWrapper, PyWrapperSemverExt, PyWrapperT2};
use pytauri_core::{ext_mod::ImplManager, tauri_runtime::Runtime};
use tauri_plugin_dialog::{self as plugin, MessageDialogButtons};

//region errors
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
//endregion

//region message dialog kind
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
        match value {
            PyMessageDialogKind::Info => Self::Info,
            PyMessageDialogKind::Warning => Self::Warning,
            PyMessageDialogKind::Error => Self::Error,
        }
    }
}
//endregion

//region message dialog buttons
use pyo3::types::PyString;

// Q: Can we have args for python enums?
#[pyclass(frozen)]
enum PyMessageDialogButtons {
    Ok(),
    OkCancel(),
    YesNo(),
    OkCustom(Py<PyString>),
    OkCancelCustom(Py<PyString>, Py<PyString>),
}

impl PyMessageDialogButtons {
    fn to_tauri(&self, py: Python<'_>) -> MessageDialogButtons {
        match self {
            PyMessageDialogButtons::Ok() => MessageDialogButtons::Ok,
            PyMessageDialogButtons::OkCancel() => MessageDialogButtons::OkCancel,
            PyMessageDialogButtons::YesNo() => MessageDialogButtons::YesNo,
            PyMessageDialogButtons::OkCustom(custom_ok) => {
                MessageDialogButtons::OkCustom(custom_ok.to_string())
            }
            PyMessageDialogButtons::OkCancelCustom(custom_ok_py, custom_cancel_py) => {
                MessageDialogButtons::OkCancelCustom(
                    custom_ok_py.to_string(),
                    custom_cancel_py.to_string(),
                )
            }
        }
    }
}
//endregion

#[pyclass(frozen)]
#[non_exhaustive]
pub struct MessageDialogBuilder(pub PyWrapper<PyWrapperT2<plugin::MessageDialogBuilder<Runtime>>>); // Q: Why is this wrapped twice?

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

// Macro to call a correct macro based on the ImplManager type.
macro_rules! dialog_ext_method_impl {
    ($slf:expr, $macro:ident) => {
        match $slf {
            ImplDialogExt::App(v) => $macro!(v),
            ImplDialogExt::AppHandle(v) => $macro!(v),
            ImplDialogExt::WebviewWindow(v) => $macro!(v),
            _ => unimplemented!("please create an feature request to pytauri"),
        }
    };
}

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
        py: Python<'_>,
        message: String,
        title: Option<String>,
        buttons: Option<PyMessageDialogButtons>, // Q: This used to be  Option<Py<PyMessageDialogButtons>>, why do we need to wrap Py around it if it's already a python class?
        kind: Option<PyMessageDialogKind>,
    ) -> PyResult<MessageDialogBuilder> {
        
        let dialog_builder = plugin::Dialog::message(message) // Q: Wrong, we need a dialog instance, for which we need an apphandle.
        let dialog_title = title; // TODO: Remove.
        let dialog_buttons = buttons.unwrap_or(PyMessageDialogButtons::Ok()).to_tauri(py); // Q: Can we supply defaults in the python signature that are not none to make it more readable for python users?
        let dialog_kind =
            plugin::MessageDialogKind::from(kind.unwrap_or(PyMessageDialogKind::Info));


        // Macro to create a new message dialog builder based on which enum ImplDialogExt is.
        macro_rules! builder_impl {
            ($wrapper:expr) => {{
                let py_ref = $wrapper.borrow(py);
                let guard = py_ref.0.inner_ref_semver()??;
                let builder: plugin::MessageDialogBuilder = guard.dialog().builder(); // it's short enough, so we don't release the GIL
                // Q: How do I know above line works for dialog as well?
                if let Some(title) = title {
                    builder.title(title);
                }
                builder.buttons(dialog_buttons);
                builder.kind(dialog_kind);

                Ok(MessageDialogBuilder::new(builder))
            }};
        }
        dialog_ext_method_impl!(slf, builder_impl)
    }
}
