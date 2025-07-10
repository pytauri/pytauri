use std::{
    convert::Infallible,
    error::Error,
    fmt::{Debug, Display},
};

use pyo3::{exceptions::PyRuntimeError, prelude::*, types::PyString, IntoPyObject};
use pyo3_utils::py_wrapper::{PyWrapper, PyWrapperT0};

use crate::{
    ext_mod::{
        image::Image,
        menu::{Menu, MenuEvent},
        plugin::{BoxedPluginWrapper, Plugin},
        tray::{TrayIcon, TrayIconEvent},
        Theme,
    },
    tauri_runtime::Runtime,
    utils::{delegate_inner, PyResultExt as _},
};

pub(crate) type TauriAppHandle = tauri::AppHandle<Runtime>;

// TODO: move into the `utils` module or make it a method of [AppHandle]
pub(crate) fn debug_assert_app_handle_py_is_rs(
    py_app_handle: &Py<AppHandle>,
    rs_app_handle: &TauriAppHandle,
) {
    if cfg!(debug_assertions) {
        let py_app_handle = py_app_handle.get().0.inner_ref();
        let py_app_handle_global = py_app_handle.py_app_handle();
        let rs_app_handle_global = rs_app_handle.py_app_handle();
        debug_assert!(
            py_app_handle_global.is(rs_app_handle_global),
            "AppHandle pyobject instance is not the same as the rust instance"
        );
    }
}

/// You can get the global singleton [Py]<[AppHandle]> using [PyAppHandleExt].
#[pyclass(frozen)]
#[non_exhaustive]
// NOTE: Do not use [PyWrapperT2], otherwise the global singleton [PyAppHandle]
// will be consumed and cannot be used;
// If you really need ownership of [tauri::AppHandle], you can use [tauri::AppHandle::clone].
pub struct AppHandle(pub PyWrapper<PyWrapperT0<TauriAppHandle>>);

impl AppHandle {
    /// NOTE: use [PyAppHandleExt] instead.
    fn new(app_handle: TauriAppHandle) -> Self {
        Self(PyWrapper::new0(app_handle))
    }
}

#[pymethods]
impl AppHandle {
    fn run_on_main_thread(&self, py: Python<'_>, handler: PyObject) -> PyResult<()> {
        py.allow_threads(|| {
            delegate_inner!(self, run_on_main_thread, move || {
                Python::with_gil(|py| {
                    let handler = handler.bind(py);
                    let result = handler.call0();
                    result.unwrap_unraisable_py_result(py, Some(handler), || {
                        "Python exception occurred in `AppHandle::run_on_main_thread`"
                    });
                })
            })
        })
    }

    fn plugin(&self, py: Python<'_>, plugin: Py<Plugin>) -> PyResult<()> {
        py.allow_threads(|| {
            let plugin = plugin.get().into_tauri()??;
            delegate_inner!(self, plugin, BoxedPluginWrapper(plugin))
        })
    }

    fn exit(&self, py: Python<'_>, exit_code: i32) {
        py.allow_threads(|| self.0.inner_ref().exit(exit_code))
    }

    /// NoReturn
    fn restart(&self, py: Python<'_>) {
        let _: Infallible = py.allow_threads(|| self.0.inner_ref().restart());
    }

    pub(crate) fn on_menu_event(slf: Py<Self>, py: Python<'_>, handler: PyObject) {
        let moved_slf = slf.clone_ref(py);
        py.allow_threads(|| {
            slf.get()
                .0
                .inner_ref()
                .on_menu_event(move |_app_handle, menu_event| {
                    Python::with_gil(|py| {
                        let app_handle: &Py<Self> = &moved_slf;
                        debug_assert_app_handle_py_is_rs(app_handle, _app_handle);
                        let menu_event: Bound<'_, MenuEvent> =
                            MenuEvent::intern(py, &menu_event.id.0);

                        let handler = handler.bind(py);
                        let result = handler.call1((app_handle, menu_event));
                        result.unwrap_unraisable_py_result(py, Some(handler), || {
                            "Python exception occurred in `AppHandle::on_menu_event` handler"
                        });
                    })
                })
        })
    }

    fn on_tray_icon_event(slf: Py<Self>, py: Python<'_>, handler: PyObject) {
        let moved_slf = slf.clone_ref(py);
        py.allow_threads(|| {
            slf.get()
                .0
                .inner_ref()
                .on_tray_icon_event(move |_app_handle, tray_icon_event| {
                    Python::with_gil(|py| {
                        let app_handle: &Py<Self> = &moved_slf;
                        debug_assert_app_handle_py_is_rs(app_handle, _app_handle);
                        let tray_icon_event: TrayIconEvent =
                            TrayIconEvent::from_tauri(py, &tray_icon_event)
                                // TODO: maybe we should only `write_unraisable` and log it instead of `panic` here?
                                .expect("Failed to convert rust `TrayIconEvent` to pyobject");

                        let handler = handler.bind(py);
                        let result = handler.call1((app_handle, tray_icon_event));
                        result.unwrap_unraisable_py_result(py, Some(handler), || {
                            "Python exception occurred in `AppHandle::on_tray_icon_event` handler"
                        });
                    })
                })
        })
    }

    fn tray_by_id(&self, py: Python<'_>, id: &str) -> Option<TrayIcon> {
        py.allow_threads(|| self.0.inner_ref().tray_by_id(id).map(TrayIcon::new))
    }

    fn remove_tray_by_id(&self, py: Python<'_>, id: &str) -> Option<TrayIcon> {
        py.allow_threads(|| self.0.inner_ref().remove_tray_by_id(id).map(TrayIcon::new))
    }

    fn set_theme(&self, py: Python<'_>, theme: Option<Theme>) {
        py.allow_threads(|| self.0.inner_ref().set_theme(theme.map(Into::into)))
    }

    fn default_window_icon(&self, py: Python<'_>) -> Option<Image> {
        self.0
            .inner_ref()
            // this is not a blocking operation, so we don't need to release the GIL
            .default_window_icon()
            .map(|icon| Image::from_tauri(py, icon))
    }

    fn menu(&self, py: Python<'_>) -> Option<Menu> {
        py.allow_threads(|| self.0.inner_ref().menu().map(Menu::new))
    }

    fn set_menu(&self, py: Python<'_>, menu: Py<Menu>) -> PyResult<Option<Menu>> {
        py.allow_threads(|| {
            let menu = menu.get().0.inner_ref().clone();
            let returned_menu = delegate_inner!(self, set_menu, menu)?;
            PyResult::Ok(returned_menu.map(Menu::new))
        })
    }

    fn remove_menu(&self, py: Python<'_>) -> PyResult<Option<Menu>> {
        py.allow_threads(|| {
            let returned_menu = delegate_inner!(self, remove_menu,)?;
            PyResult::Ok(returned_menu.map(Menu::new))
        })
    }

    fn hide_menu(&self, py: Python<'_>) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, hide_menu,))
    }

    fn show_menu(&self, py: Python<'_>) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, show_menu,))
    }

    fn invoke_key<'py>(&self, py: Python<'py>) -> Bound<'py, PyString> {
        // if `invoke_key` is immutable, we can intern it to save memory.
        PyString::intern(py, self.0.inner_ref().invoke_key())
    }
}

/// This error indicates that the app was not initialized using [crate::ext_mod::App::try_build],
/// i.e. it was not created by pytauri.
#[derive(Debug)]
pub struct PyAppHandleStateError;

impl Display for PyAppHandleStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to get `PyAppHandle` from state, maybe this app was not created by pytauri"
        )
    }
}

impl Error for PyAppHandleStateError {}

impl From<PyAppHandleStateError> for PyErr {
    fn from(value: PyAppHandleStateError) -> Self {
        PyRuntimeError::new_err(format!("{value}"))
    }
}

pub type PyAppHandleStateResult<T> = Result<T, PyAppHandleStateError>;

mod sealed {
    use super::*;

    pub trait SealedPyAppHandleExt {}
    impl<T: tauri::Manager<Runtime>> SealedPyAppHandleExt for T {}

    pub(super) struct PyAppHandle(pub(crate) Py<AppHandle>);
}

use sealed::{PyAppHandle, SealedPyAppHandleExt};

/// You can use this trait to get the global singleton [Py]<[AppHandle]>.
//
// NOTE: due to the unsoundness of [Manager::unmanage], do not allow to unmanage `PyAppHandle`,
// see: <https://github.com/tauri-apps/tauri/issues/12721>.
pub trait PyAppHandleExt: tauri::Manager<Runtime> + SealedPyAppHandleExt {
    /// See [PyAppHandleExt::try_py_app_handle] for details.
    ///
    /// # Panics
    ///
    /// Panics if [PyAppHandleExt::try_py_app_handle] returns an error.
    fn py_app_handle(&self) -> &Py<AppHandle> {
        self.try_py_app_handle().unwrap()
    }

    /// Get the global singleton [Py]<[AppHandle]>.
    ///
    /// If it has not been initialized, it will return an error.
    /// Use [PyAppHandleExt::get_or_init_py_app_handle] to initialize.
    fn try_py_app_handle(&self) -> PyAppHandleStateResult<&Py<AppHandle>> {
        let state = self
            .try_state::<PyAppHandle>()
            .ok_or(PyAppHandleStateError)?;
        Ok(&state.inner().0)
    }

    /// Get or initialize the global singleton [Py]<[AppHandle]>.
    ///
    /// It may return an error only during the first initialization.
    /// Once successfully called for the first time, subsequent calls will always return [Ok].
    ///
    /// [crate::ext_mod::App::try_build] will call this method,
    /// which means if you already have an [crate::ext_mod::App] instance,
    /// the [AppHandle] has also been initialized.
    fn get_or_init_py_app_handle(&self, py: Python<'_>) -> PyResult<&Py<AppHandle>> {
        match self.try_py_app_handle() {
            Ok(py_app_handle) => Ok(py_app_handle),
            Err(_) => {
                let py_app_handle = AppHandle::new(self.app_handle().to_owned());
                let py_app_handle = py_app_handle.into_pyobject(py)?.unbind();
                let not_yet_managed = self.manage::<PyAppHandle>(PyAppHandle(py_app_handle));
                debug_assert!(
                    not_yet_managed,
                    "`PyAppHandle` is private, so it is impossible for other crates to manage it, \
                    and for self crate, it should be initialized only once."
                );
                Ok(self
                    .try_py_app_handle()
                    .expect("`PyAppHandle` has already been initialized, so this never fail"))
            }
        }
    }
}

impl<T: tauri::Manager<Runtime>> PyAppHandleExt for T {}
