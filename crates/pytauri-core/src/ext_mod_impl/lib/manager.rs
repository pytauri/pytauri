use std::{collections::HashMap, iter::Iterator};

use pyo3::{
    exceptions::PyValueError,
    marker::Ungil,
    prelude::*,
    types::{PyDict, PyType},
    FromPyObject, IntoPyObject,
};
use pyo3_utils::ungil::UnsafeUngilExt;
use tauri::{Manager as TauriManager, State};

use crate::{
    ext_mod::{
        path::PathResolver,
        webview::{TauriWebviewWindow, WebviewWindow},
        App, AppHandle, PyAppHandleExt, TauriApp, TauriAppHandle,
    },
    tauri_runtime::Runtime,
};

pub(crate) struct StateManager(Py<PyDict>);

impl StateManager {
    pub(crate) fn get_or_init<'a>(
        py: Python<'_>,
        manager: &'a impl TauriManager<Runtime>,
    ) -> State<'a, Self> {
        if let Some(state) = manager.try_state::<Self>() {
            return state;
        }
        manager.manage(Self(PyDict::new(py).into()));
        manager.state::<Self>()
    }

    pub(crate) fn manage(&self, py: Python<'_>, state: &Bound<PyAny>) -> PyResult<bool> {
        let this = self.0.bind(py);
        let py_type = state.get_type();
        // If the state for the T type has previously been set, the state is unchanged and false is returned.
        // Otherwise true is returned.
        if this.contains(&py_type)? {
            return Ok(false);
        }
        this.set_item(py_type, state)?;
        Ok(true)
    }

    pub(crate) fn state<'py>(
        &self,
        py: Python<'py>,
        state_type: &Bound<'py, PyType>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let state = self.try_state(py, state_type)?;
        state.ok_or_else(|| {
            PyValueError::new_err(format!("state() called before manage() for {state_type}"))
        })
    }

    pub(crate) fn try_state<'py>(
        &self,
        py: Python<'py>,
        state_type: &Bound<'py, PyType>,
    ) -> PyResult<Option<Bound<'py, PyAny>>> {
        let this = self.0.bind(py);
        PyDictMethods::get_item(this, state_type)
    }
}

/// The Implementers of [tauri::Manager].
#[derive(FromPyObject, IntoPyObject, IntoPyObjectRef)]
#[non_exhaustive]
// TODO: more types
pub enum ImplManager {
    AppHandle(Py<AppHandle>),
    WebviewWindow(Py<WebviewWindow>),
    // NOTE: Put `App` at the end because it is the least likely to appear
    App(Py<App>),
}

impl ImplManager {
    #[inline]
    pub(crate) fn _delegate_app<'py, R>(
        py: Python<'py>,
        app: &Py<App>,
        f: impl FnOnce(Python<'py>, &TauriApp) -> R,
    ) -> PyResult<R> {
        let py_app = app.borrow(py);
        let rs_app = py_app.0.try_lock_inner_ref()??;
        Ok(f(py, &rs_app))
    }

    #[inline]
    pub(crate) fn _delegate_app_handle<'py, R>(
        py: Python<'py>,
        app_handle: &Py<AppHandle>,
        f: impl FnOnce(Python<'py>, &TauriAppHandle) -> R,
    ) -> PyResult<R> {
        let app_handle = app_handle.get().0.inner_ref();
        Ok(f(py, &app_handle))
    }

    #[inline]
    pub(crate) fn _delegate_webview_window<'py, R>(
        py: Python<'py>,
        webview_window: &Py<WebviewWindow>,
        f: impl FnOnce(Python<'py>, &TauriWebviewWindow) -> R,
    ) -> PyResult<R> {
        let webview_window = webview_window.get().0.inner_ref();
        Ok(f(py, &webview_window))
    }

    #[inline]
    pub(crate) fn _delegate_manager_ungil<M, F, R>(py: Python<'_>, manager: &M, f: F) -> R
    where
        M: tauri::Manager<Runtime>,
        F: FnOnce(&M) -> R + Ungil + Send,
        R: Ungil,
    {
        unsafe {
            // safety: `tauri::Manager` does not hold the GIL, so this is safe
            py.allow_threads_unsend(manager, f)
        }
    }
}

/**
```ignore
fn manager_method_impl(py: Python<'_>, manager: &ImplManager) -> PyResult<()> {
    manager_method_impl!(py, manager, |_py, manager| {
        // here the `manager` is equivalent to `&impl tauri::Manager<Runtime>`
        manager.get_webview_window("main")
    })?;

    manager_method_impl!(py, manager, [ungil], |manager| {
        // here equivalent to `Python::allow_threads_unsend`
        manager.get_webview_window("main")
    })?;

    Ok(())
}
```
*/
macro_rules! manager_method_impl {
    // impl
    ($py:expr, $manager:expr, $f0:expr, $f1:expr, $f2:expr) => {{
        use $crate::ext_mod::ImplManager;

        let manager: &ImplManager = $manager;
        match manager {
            ImplManager::App(v) => {
                ImplManager::_delegate_app($py, v, $f0)
            }
            ImplManager::AppHandle(v) => {
                ImplManager::_delegate_app_handle($py, v, $f1)
            }
            ImplManager::WebviewWindow(v) => {
                ImplManager::_delegate_webview_window($py, v, $f2)
            }
        }
    }};

    // entry1 -> entry0
    ($py:expr, $manager:expr, [ungil], $($f:tt)*) => {{
        manager_method_impl!($py, $manager, |py, manager| {
            $crate::ext_mod::ImplManager::_delegate_manager_ungil(py, manager, $($f)*)
        })
    }};
    // entry0
    ($py:expr, $manager:expr, $($f:tt)*) => {
        manager_method_impl!($py, $manager, $($f)*, $($f)*, $($f)*)
    };
}

// if export this macro, remember to enable doctest 👆
pub(crate) use manager_method_impl;

/// See also: [tauri::Manager].
#[pyclass(frozen)]
#[non_exhaustive]
pub struct Manager;

#[pymethods]
impl Manager {
    #[staticmethod]
    fn app_handle(py: Python<'_>, slf: ImplManager) -> PyResult<Py<AppHandle>> {
        manager_method_impl!(py, &slf, |py, manager| manager
            // TODO, PERF: release the GIL?
            .py_app_handle()
            .clone_ref(py))
    }

    #[staticmethod]
    fn get_webview_window(
        py: Python<'_>,
        slf: ImplManager,
        label: &str,
    ) -> PyResult<Option<WebviewWindow>> {
        manager_method_impl!(py, &slf, [ungil], |manager| {
            manager.get_webview_window(label).map(WebviewWindow::new)
        })
    }

    #[staticmethod]
    fn webview_windows(
        py: Python<'_>,
        slf: ImplManager,
    ) -> PyResult<HashMap<String, WebviewWindow>> {
        manager_method_impl!(py, &slf, [ungil], |manager| {
            manager
                .webview_windows()
                .into_iter()
                .map(|(label, window)| (label, WebviewWindow::new(window)))
                .collect::<_>()
        })
    }

    #[staticmethod]
    fn manage(py: Python<'_>, slf: ImplManager, state: &Bound<PyAny>) -> PyResult<bool> {
        manager_method_impl!(py, &slf, |py, manager| {
            let state_manager = StateManager::get_or_init(py, manager);
            state_manager.manage(py, state)
        })?
    }

    #[staticmethod]
    fn state<'py>(
        py: Python<'py>,
        slf: ImplManager,
        state_type: &Bound<'py, PyType>,
    ) -> PyResult<Bound<'py, PyAny>> {
        manager_method_impl!(py, &slf, |py, manager| {
            let state_manager = StateManager::get_or_init(py, manager);
            state_manager.state(py, state_type)
        })?
    }

    #[staticmethod]
    fn try_state<'py>(
        py: Python<'py>,
        slf: ImplManager,
        state_type: &Bound<'py, PyType>,
    ) -> PyResult<Option<Bound<'py, PyAny>>> {
        manager_method_impl!(py, &slf, |py, manager| {
            let state_manager = StateManager::get_or_init(py, manager);
            state_manager.try_state(py, state_type)
        })?
    }

    #[staticmethod]
    fn path(py: Python<'_>, slf: ImplManager) -> PyResult<PathResolver> {
        manager_method_impl!(py, &slf, [ungil], |manager| {
            let path_resolver = manager.path().clone();
            PathResolver::new(path_resolver)
        })
    }
}
