use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
    ops::Deref,
};

use pyo3::{
    exceptions::PyRuntimeError,
    prelude::*,
    pybacked::PyBackedStr,
    types::{PyDict, PyString},
};
use pyo3_utils::from_py_dict::{derive_from_py_dict, FromPyDict as _, NotRequired};
use tauri::Manager as _;
use tauri_plugin_dialog::{self as plugin, DialogExt as _};

use crate::{
    ext_mod::{manager_method_impl, webview::WebviewWindow, ImplManager},
    tauri_runtime::Runtime,
    utils::{PyResultExt as _, TauriError},
};

#[derive(Debug)]
struct PluginError(plugin::Error);

impl Display for PluginError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Error for PluginError {}

impl From<PluginError> for PyErr {
    fn from(value: PluginError) -> Self {
        match value.0 {
            plugin::Error::Io(e) => e.into(),
            plugin::Error::Tauri(e) => TauriError::from(e).into(),
            // TODO: unify this error with `tauri_plugin_fs::Error`
            plugin::Error::Fs(e) => PyRuntimeError::new_err(e.to_string()),
            non_exhaustive => PyRuntimeError::new_err(format!(
                "Unimplemented plugin error, please report this to the pytauri developers: {non_exhaustive}"
            )),
        }
    }
}

impl From<plugin::Error> for PluginError {
    fn from(value: plugin::Error) -> Self {
        Self(value)
    }
}

// TODO: support `tauri::window::Window` also, something like:
//
// ```rust
// // TODO: unify/stable this in `pytauri_core::ext_mod`
// #[non_exhaustive]
// #[derive(FromPyObject)]
// enum HasWindowHandleAndHasDisplayHandle {
//     WebviewWindow(Py<WebviewWindow>),
//     Window(Py<Window>),
// }
// ```
type HasWindowHandleAndHasDisplayHandle = Py<WebviewWindow>;

/// See also: [tauri_plugin_dialog::MessageDialogButtons]
#[pyclass(frozen)]
#[non_exhaustive]
pub enum MessageDialogButtons {
    Ok(),
    OkCancel(),
    YesNo(),
    // TODO, PERF: or we can use [pyo3::pybacked::PyBackedStr],
    // so that we don't need to require GIL for `fn to_tauri`.
    OkCustom(Py<PyString>),
    OkCancelCustom(Py<PyString>, Py<PyString>),
}

impl MessageDialogButtons {
    fn to_tauri(&self, py: Python<'_>) -> PyResult<plugin::MessageDialogButtons> {
        let ret = match self {
            MessageDialogButtons::Ok() => plugin::MessageDialogButtons::Ok,
            MessageDialogButtons::OkCancel() => plugin::MessageDialogButtons::OkCancel,
            MessageDialogButtons::YesNo() => plugin::MessageDialogButtons::YesNo,
            MessageDialogButtons::OkCustom(text) => {
                // TODO, PERF: once we drop py39 support, we can use [PyStringMethods::to_str] directly.
                plugin::MessageDialogButtons::OkCustom(text.to_cow(py)?.into_owned())
            }
            MessageDialogButtons::OkCancelCustom(ok_text, cancel_text) => {
                // TODO, PERF: once we drop py39 support, we can use [PyStringMethods::to_str] directly.
                plugin::MessageDialogButtons::OkCancelCustom(
                    ok_text.to_cow(py)?.into_owned(),
                    cancel_text.to_cow(py)?.into_owned(),
                )
            }
        };
        Ok(ret)
    }
}

macro_rules! message_dialog_kind_impl {
    ($ident:ident => : $($variant:ident),*) => {
        /// See also: [tauri_plugin_dialog::MessageDialogKind]
        #[pyclass(frozen, eq, eq_int)]
        #[derive(PartialEq, Clone, Copy)]
        pub enum $ident {
            $($variant,)*
        }

        impl From<$ident> for tauri_plugin_dialog::MessageDialogKind {
            fn from(val: $ident) -> Self {
                match val {
                    $($ident::$variant => tauri_plugin_dialog::MessageDialogKind::$variant,)*
                }
            }
        }
    };
}

message_dialog_kind_impl!(
    MessageDialogKind => :
    Info,
    Warning,
    Error
);

/// See also: [tauri_plugin_dialog::MessageDialogBuilder]
#[non_exhaustive]
pub struct MessageDialogBuilderArgs {
    title: NotRequired<String>,
    parent: NotRequired<HasWindowHandleAndHasDisplayHandle>,
    buttons: NotRequired<Py<MessageDialogButtons>>,
    kind: NotRequired<Py<MessageDialogKind>>,
}

derive_from_py_dict!(MessageDialogBuilderArgs {
    #[default]
    title,
    #[default]
    parent,
    #[default]
    buttons,
    #[default]
    kind,
});

impl MessageDialogBuilderArgs {
    fn from_kwargs(kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<Option<Self>> {
        kwargs
            .map(MessageDialogBuilderArgs::from_py_dict)
            .transpose()
    }

    fn apply_to_builder(
        self,
        py: Python<'_>,
        mut builder: plugin::MessageDialogBuilder<Runtime>,
    ) -> PyResult<plugin::MessageDialogBuilder<Runtime>> {
        let Self {
            title,
            parent,
            buttons,
            kind,
        } = self;

        if let Some(title) = title.0 {
            builder = builder.title(title);
        }
        if let Some(parent) = parent.0 {
            builder = builder.parent(&*parent.get().0.inner_ref());
        }
        if let Some(buttons) = buttons.0 {
            builder = builder.buttons(buttons.get().to_tauri(py)?);
        }
        if let Some(kind) = kind.0 {
            builder = builder.kind((*kind.get()).into());
        }

        Ok(builder)
    }
}

/// See also: [tauri_plugin_dialog::MessageDialogBuilder]
#[pyclass(frozen)]
#[non_exhaustive]
// [tauri_plugin_dialog::MessageDialogBuilder] is `!Sync`,
// so we wrap [tauri::AppHandle] instead.
pub struct MessageDialogBuilder {
    handle: tauri::AppHandle<Runtime>,
    message: PyBackedStr,
}

impl MessageDialogBuilder {
    fn to_tauri(&self) -> plugin::MessageDialogBuilder<Runtime> {
        let Self { handle, message } = self;
        handle.dialog().message(message.deref())
    }
}

#[pymethods]
impl MessageDialogBuilder {
    #[pyo3(signature = (**kwargs))]
    fn blocking_show(&self, py: Python<'_>, kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<bool> {
        let args = MessageDialogBuilderArgs::from_kwargs(kwargs)?;

        let mut builder = self.to_tauri();
        if let Some(args) = args {
            builder = args.apply_to_builder(py, builder)?;
        }

        let ret = py.allow_threads(|| builder.blocking_show());
        Ok(ret)
    }

    #[pyo3(signature = (handler, /, **kwargs))]
    fn show(
        &self,
        py: Python<'_>,
        handler: PyObject,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<()> {
        let args = MessageDialogBuilderArgs::from_kwargs(kwargs)?;

        let mut builder = self.to_tauri();
        if let Some(args) = args {
            builder = args.apply_to_builder(py, builder)?;
        }

        // PERF: it's short enough, so we don't release the GIL
        builder.show(move |is_ok| {
            Python::with_gil(|py| {
                let handler = handler.bind(py);
                let result = handler.call1((is_ok,));
                result.unwrap_unraisable_py_result(py, Some(handler), || {
                    "Python exception occurred in `MessageDialogBuilder::show` handler"
                });
            })
        });

        Ok(())
    }
}

/// See also: [tauri_plugin_dialog::DialogExt]
#[pyclass(frozen)]
#[non_exhaustive]
pub struct DialogExt;

/// The Implementers of [tauri_plugin_dialog::DialogExt].
pub type ImplDialogExt = ImplManager;

#[pymethods]
impl DialogExt {
    #[staticmethod]
    fn message(
        slf: ImplDialogExt,
        py: Python<'_>,
        message: PyBackedStr,
    ) -> PyResult<MessageDialogBuilder> {
        manager_method_impl!(py, &slf, |_py, manager| {
            // PERF: it's short enough, so we don't release the GIL
            let handle = manager.app_handle().clone();
            MessageDialogBuilder { handle, message }
        })
    }
}

/// See also: [tauri_plugin_dialog]
#[pymodule(submodule, gil_used = false)]
pub mod dialog {
    #[pymodule_export]
    pub use super::{DialogExt, MessageDialogBuilder, MessageDialogButtons, MessageDialogKind};

    pub use super::{ImplDialogExt, MessageDialogBuilderArgs};
}
