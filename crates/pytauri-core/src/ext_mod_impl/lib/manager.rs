use std::{collections::HashMap, iter::Iterator};

use pyo3::{marker::Ungil, prelude::*, FromPyObject, IntoPyObject};
use pyo3_utils::ungil::UnsafeUngilExt;
use tauri::Manager as _;

use crate::{
    ext_mod::{
        path::PathResolver,
        webview::{TauriWebviewWindow, WebviewWindow},
        App, AppHandle, PyAppHandleExt, TauriApp, TauriAppHandle,
    },
    tauri_runtime::Runtime,
};

/// The Implementers of [tauri::Manager].
#[derive(FromPyObject, IntoPyObject, IntoPyObjectRef)]
#[non_exhaustive]
// TODO: more types
pub enum ImplManager {
    App(Py<App>),
    AppHandle(Py<AppHandle>),
    WebviewWindow(Py<WebviewWindow>),
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
    fn path(py: Python<'_>, slf: ImplManager) -> PyResult<PathResolver> {
        manager_method_impl!(py, &slf, [ungil], |manager| {
            let path_resolver = manager.path().clone();
            PathResolver::new(path_resolver)
        })
    }
}
