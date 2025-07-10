use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
    ops::Deref,
    path::PathBuf,
};

use pyo3::{
    exceptions::{PyNotImplementedError, PyRuntimeError},
    prelude::*,
    pybacked::PyBackedStr,
    types::{PyDict, PyString},
};
use pyo3_utils::from_py_dict::{derive_from_py_dict, FromPyDict as _, NotRequired};
use tauri::Manager as _;
use tauri_plugin_dialog::{self as plugin, DialogExt as _};

use crate::{
    ext_mod::{manager_method_impl, plugin::Plugin, webview::WebviewWindow, ImplManager},
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

/// See also: [tauri_plugin_dialog::init]
#[pyfunction]
pub fn init() -> Plugin {
    Plugin::new(|| Box::new(plugin::init::<Runtime>()))
}

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

// TODO: unify this type with [tauri_plugin_fs::FilePath]
/// See also: [tauri_plugin_dialog::FilePath]
pub struct FilePath(plugin::FilePath);

impl From<plugin::FilePath> for FilePath {
    fn from(value: plugin::FilePath) -> Self {
        Self(value)
    }
}

impl From<FilePath> for plugin::FilePath {
    fn from(value: FilePath) -> Self {
        value.0
    }
}

/// `pathlib.Path`
impl<'py> IntoPyObject<'py> for &FilePath {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let ret = match &self.0 {
            plugin::FilePath::Url(_) => {
                // TODO: support Android/iOS:
                //
                // let pyobj: Bound<'_, PyString> = Url::from(url).into_pyobject(py)?;
                // pyobj.into_any() // str
                return Err(PyNotImplementedError::new_err(
                    "[FilePath::Url] type is only used on Android/iOS, report this to the pytauri developers"
                ));
            }
            plugin::FilePath::Path(path) => {
                let path: &PathBuf = path;
                let pyobj: Bound<'_, PyAny> = path.into_pyobject(py)?; // pathlib.Path
                pyobj
            }
        };
        Ok(ret)
    }
}

impl<'py> IntoPyObject<'py> for FilePath {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (&self).into_pyobject(py)
    }
}

/// See also: [tauri_plugin_dialog::FileDialogBuilder]
#[non_exhaustive]
pub struct FileDialogBuilderArgs {
    // TODO, PERF: avoid `Vec`, use `PyIterable` or `smallvec` instead.
    add_filter: NotRequired<(String, Vec<PyBackedStr>)>,
    // PERF: avoid `PathBuf`, prefer `&Path` instead.
    set_directory: NotRequired<PathBuf>,
    set_file_name: NotRequired<String>,
    set_parent: NotRequired<HasWindowHandleAndHasDisplayHandle>,
    set_title: NotRequired<String>,
    set_can_create_directories: NotRequired<bool>,
}

derive_from_py_dict!(FileDialogBuilderArgs {
    #[default]
    add_filter,
    #[default]
    set_directory,
    #[default]
    set_file_name,
    #[default]
    set_parent,
    #[default]
    set_title,
    #[default]
    set_can_create_directories,
});

impl FileDialogBuilderArgs {
    fn from_kwargs(kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<Option<Self>> {
        kwargs.map(FileDialogBuilderArgs::from_py_dict).transpose()
    }

    fn apply_to_builder(
        self,
        mut builder: plugin::FileDialogBuilder<Runtime>,
    ) -> plugin::FileDialogBuilder<Runtime> {
        let Self {
            add_filter,
            set_directory,
            set_file_name,
            set_parent,
            set_title,
            set_can_create_directories,
        } = self;

        if let Some((name, extensions)) = add_filter.0 {
            // TODO, PERF: avoid alloc `Vec` (because `collect`) here
            let extensions = extensions.iter().map(|s| s.deref()).collect::<Vec<_>>();
            builder = builder.add_filter(name, &extensions);
        }
        if let Some(directory) = set_directory.0 {
            builder = builder.set_directory(directory);
        }
        if let Some(file_name) = set_file_name.0 {
            builder = builder.set_file_name(file_name);
        }
        if let Some(parent) = set_parent.0 {
            builder = builder.set_parent(&*parent.get().0.inner_ref());
        }
        if let Some(title) = set_title.0 {
            builder = builder.set_title(title);
        }
        if let Some(can_create_directories) = set_can_create_directories.0 {
            builder = builder.set_can_create_directories(can_create_directories);
        }

        builder
    }
}

/// See also: [tauri_plugin_dialog::FileDialogBuilder]
#[pyclass(frozen)]
#[non_exhaustive]
// [tauri_plugin_dialog::FileDialogBuilder] is `!Sync`,
// so we wrap [tauri::AppHandle] instead.
pub struct FileDialogBuilder {
    handle: tauri::AppHandle<Runtime>,
}

impl FileDialogBuilder {
    fn to_tauri(&self) -> plugin::FileDialogBuilder<Runtime> {
        let Self { handle } = self;
        handle.dialog().file()
    }
}

#[pymethods]
impl FileDialogBuilder {
    #[pyo3(signature = (handler, /, **kwargs))]
    fn pick_file(&self, handler: PyObject, kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<()> {
        let args = FileDialogBuilderArgs::from_kwargs(kwargs)?;

        let mut builder = self.to_tauri();
        if let Some(args) = args {
            builder = args.apply_to_builder(builder);
        }

        // PERF: it's short enough, so we don't release the GIL
        builder.pick_file(move |file_path| {
            Python::with_gil(|py| {
                let file_path = file_path.map(FilePath::from);

                let handler = handler.bind(py);
                let result = handler.call1((file_path,));
                result.unwrap_unraisable_py_result(py, Some(handler), || {
                    "Python exception occurred in `FileDialogBuilder::pick_file` handler"
                });
            })
        });

        Ok(())
    }

    #[pyo3(signature = (**kwargs))]
    fn blocking_pick_file(
        &self,
        py: Python<'_>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Option<FilePath>> {
        let args = FileDialogBuilderArgs::from_kwargs(kwargs)?;

        let mut builder = self.to_tauri();
        if let Some(args) = args {
            builder = args.apply_to_builder(builder);
        }

        let ret = py.allow_threads(|| builder.blocking_pick_file().map(Into::into));
        Ok(ret)
    }

    #[pyo3(signature = (handler, / ,**kwargs))]
    fn pick_files(&self, handler: PyObject, kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<()> {
        let args = FileDialogBuilderArgs::from_kwargs(kwargs)?;

        let mut builder = self.to_tauri();
        if let Some(args) = args {
            builder = args.apply_to_builder(builder);
        }

        // PERF: it's short enough, so we don't release the GIL
        builder.pick_files(move |file_paths| {
            Python::with_gil(|py| {
                let file_paths = file_paths
                    // TODO, PERF: avoid `Vec`, use `PyList` instead
                    .map(|files| files.into_iter().map(FilePath::from).collect::<Vec<_>>());

                let handler = handler.bind(py);
                let result = handler.call1((file_paths,));
                result.unwrap_unraisable_py_result(py, Some(handler), || {
                    "Python exception occurred in `FileDialogBuilder::pick_files` handler"
                });
            })
        });

        Ok(())
    }

    #[pyo3(signature = (**kwargs))]
    fn blocking_pick_files(
        &self,
        py: Python<'_>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Option<Vec<FilePath>>> {
        let args = FileDialogBuilderArgs::from_kwargs(kwargs)?;

        let mut builder = self.to_tauri();
        if let Some(args) = args {
            builder = args.apply_to_builder(builder);
        }

        let ret = py.allow_threads(|| {
            builder
                .blocking_pick_files()
                // TODO, PERF: avoid `Vec`, use `PyList` instead
                .map(|files| files.into_iter().map(Into::into).collect::<Vec<_>>())
        });
        Ok(ret)
    }

    #[pyo3(signature = (handler, / ,**kwargs))]
    fn pick_folder(&self, handler: PyObject, kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<()> {
        let args = FileDialogBuilderArgs::from_kwargs(kwargs)?;

        let mut builder = self.to_tauri();
        if let Some(args) = args {
            builder = args.apply_to_builder(builder);
        }

        // PERF: it's short enough, so we don't release the GIL
        builder.pick_folder(move |file_path| {
            Python::with_gil(|py| {
                let file_path = file_path.map(FilePath::from);

                let handler = handler.bind(py);
                let result = handler.call1((file_path,));
                result.unwrap_unraisable_py_result(py, Some(handler), || {
                    "Python exception occurred in `FileDialogBuilder::pick_folder` handler"
                });
            })
        });

        Ok(())
    }

    #[pyo3(signature = (**kwargs))]
    fn blocking_pick_folder(
        &self,
        py: Python<'_>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Option<FilePath>> {
        let args = FileDialogBuilderArgs::from_kwargs(kwargs)?;

        let mut builder = self.to_tauri();
        if let Some(args) = args {
            builder = args.apply_to_builder(builder);
        }

        let ret = py.allow_threads(|| builder.blocking_pick_folder().map(Into::into));
        Ok(ret)
    }

    #[pyo3(signature = (handler, / ,**kwargs))]
    fn pick_folders(&self, handler: PyObject, kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<()> {
        let args = FileDialogBuilderArgs::from_kwargs(kwargs)?;

        let mut builder = self.to_tauri();
        if let Some(args) = args {
            builder = args.apply_to_builder(builder);
        }

        // PERF: it's short enough, so we don't release the GIL
        builder.pick_folders(move |file_paths| {
            Python::with_gil(|py| {
                let file_paths = file_paths
                    // TODO, PERF: avoid `Vec`, use `PyList` instead
                    .map(|files| files.into_iter().map(FilePath::from).collect::<Vec<_>>());

                let handler = handler.bind(py);
                let result = handler.call1((file_paths,));
                result.unwrap_unraisable_py_result(py, Some(handler), || {
                    "Python exception occurred in `FileDialogBuilder::pick_folders` handler"
                });
            })
        });

        Ok(())
    }

    #[pyo3(signature = (**kwargs))]
    fn blocking_pick_folders(
        &self,
        py: Python<'_>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Option<Vec<FilePath>>> {
        let args = FileDialogBuilderArgs::from_kwargs(kwargs)?;

        let mut builder = self.to_tauri();
        if let Some(args) = args {
            builder = args.apply_to_builder(builder);
        }

        let ret = py.allow_threads(|| {
            builder
                .blocking_pick_folders()
                // TODO, PERF: avoid `Vec`, use `PyList` instead
                .map(|files| files.into_iter().map(Into::into).collect::<Vec<_>>())
        });
        Ok(ret)
    }

    #[pyo3(signature = (handler, / ,**kwargs))]
    fn save_file(&self, handler: PyObject, kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<()> {
        let args = FileDialogBuilderArgs::from_kwargs(kwargs)?;

        let mut builder = self.to_tauri();
        if let Some(args) = args {
            builder = args.apply_to_builder(builder);
        }

        // PERF: it's short enough, so we don't release the GIL
        builder.save_file(move |file_path| {
            Python::with_gil(|py| {
                let file_path = file_path.map(FilePath::from);

                let handler = handler.bind(py);
                let result = handler.call1((file_path,));
                result.unwrap_unraisable_py_result(py, Some(handler), || {
                    "Python exception occurred in `FileDialogBuilder::save_file` handler"
                });
            })
        });

        Ok(())
    }

    #[pyo3(signature = (**kwargs))]
    fn blocking_save_file(
        &self,
        py: Python<'_>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Option<FilePath>> {
        let args = FileDialogBuilderArgs::from_kwargs(kwargs)?;

        let mut builder = self.to_tauri();
        if let Some(args) = args {
            builder = args.apply_to_builder(builder);
        }

        let ret = py.allow_threads(|| builder.blocking_save_file().map(Into::into));
        Ok(ret)
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

    #[staticmethod]
    fn file(slf: ImplDialogExt, py: Python<'_>) -> PyResult<FileDialogBuilder> {
        manager_method_impl!(py, &slf, |_py, manager| {
            // PERF: it's short enough, so we don't release the GIL
            let handle = manager.app_handle().clone();
            FileDialogBuilder { handle }
        })
    }
}

/// See also: [tauri_plugin_dialog]
#[pymodule(submodule, gil_used = false)]
pub mod dialog {
    #[pymodule_export]
    pub use super::{
        init, DialogExt, FileDialogBuilder, MessageDialogBuilder, MessageDialogButtons,
        MessageDialogKind,
    };

    pub use super::{FileDialogBuilderArgs, FilePath, ImplDialogExt, MessageDialogBuilderArgs};
}
