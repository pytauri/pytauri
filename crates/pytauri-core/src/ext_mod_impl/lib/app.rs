use pyo3::prelude::*;
use pyo3_utils::{
    py_wrapper::{PyWrapper, PyWrapperT2},
    ungil::UnsafeUngilExt,
};

use crate::{
    ext_mod::{
        debug_assert_app_handle_py_is_rs, AppHandle, PyAppHandleExt as _, RunEvent, TauriAppHandle,
    },
    tauri_runtime::Runtime,
    utils::{PyResultExt as _, TauriError},
};

pub(crate) type TauriApp = tauri::App<Runtime>;

#[pyclass(frozen, unsendable)]
#[non_exhaustive]
pub struct App(pub PyWrapper<PyWrapperT2<TauriApp>>);

impl App {
    #[cfg(feature = "__private")]
    pub fn try_build(py: Python<'_>, app: TauriApp) -> PyResult<Self> {
        // remember to initialize the global singleton [PyAppHandle], see it's doc
        app.get_or_init_py_app_handle(py)?;
        Ok(Self(PyWrapper::new2(app)))
    }

    fn py_cb_to_rs_cb(
        callback: PyObject,
        app_handle: Py<AppHandle>,
    ) -> impl FnMut(&TauriAppHandle, tauri::RunEvent) {
        move |_app_handle, run_event| {
            let py_app_handle: &Py<AppHandle> = &app_handle;
            debug_assert_app_handle_py_is_rs(&app_handle, _app_handle);

            Python::with_gil(|py| {
                let py_run_event: RunEvent = RunEvent::from_tauri(py, run_event)
                    // TODO: maybe we should only `write_unraisable` and log it instead of `panic` here?
                    .expect("Failed to convert rust `RunEvent` to pyobject");

                let callback = callback.bind(py);
                let result = callback.call1((py_app_handle, py_run_event));
                // `panic` allows Python to exit `app.run()`,
                // otherwise the Python main thread will be blocked by `app.run()`
                // and unable to raise an error.
                result.unwrap_unraisable_py_result(py, Some(callback), || {
                    "Python exception occurred in `App` run callback"
                });
            })
        }
    }

    fn noop_callback(_: &TauriAppHandle, _: tauri::RunEvent) {}
}

#[pymethods]
impl App {
    fn run_on_main_thread(&self, py: Python<'_>, handler: PyObject) -> PyResult<()> {
        unsafe {
            // TODO, PREF: do we really need to release the GIL here?
            // It seems that `App::run_on_main_thread` will be called immediately.
            //
            // Safety: `tauri::App` does not hold the GIL, so this is safe
            py.allow_threads_unsend(self, |slf| {
                let app = slf.0.try_lock_inner_ref()??;
                app.run_on_main_thread(move || {
                    Python::with_gil(|py| {
                        let handler = handler.bind(py);
                        let result = handler.call0();
                        result.unwrap_unraisable_py_result(py, Some(handler), || {
                            "Python exception occurred in `App::run_on_main_thread`"
                        });
                    })
                })
                .map_err(TauriError::from)
                .map_err(PyErr::from)
            })
        }
    }

    fn handle(&self, py: Python<'_>) -> PyResult<Py<AppHandle>> {
        let app = self.0.try_lock_inner_ref()??;
        // TODO, PERF: release the GIL?
        let app_handle = app.py_app_handle().clone_ref(py);
        Ok(app_handle)
    }

    #[pyo3(signature = (callback = None, /))]
    fn run(&self, py: Python<'_>, callback: Option<PyObject>) -> PyResult<()> {
        let app = self.0.try_take_inner()??;
        let py_app_handle = app.py_app_handle().clone_ref(py);
        unsafe {
            // Safety: `tauri::App` does not hold the GIL, so this is safe
            py.allow_threads_unsend(app, move |app| {
                match callback {
                    Some(callback) => app.run(Self::py_cb_to_rs_cb(callback, py_app_handle)),
                    None => app.run(Self::noop_callback),
                }
                Ok(())
            })
        }
    }

    #[pyo3(signature = (callback = None, /))]
    fn run_return(&self, py: Python<'_>, callback: Option<PyObject>) -> PyResult<i32> {
        let app = self.0.try_take_inner()??;
        let py_app_handle = app.py_app_handle().clone_ref(py);
        unsafe {
            // Safety: `tauri::App` does not hold the GIL, so this is safe
            py.allow_threads_unsend(app, move |app| {
                let exit_code = match callback {
                    Some(callback) => app.run_return(Self::py_cb_to_rs_cb(callback, py_app_handle)),
                    None => app.run_return(Self::noop_callback),
                };
                Ok(exit_code)
            })
        }
    }

    #[expect(deprecated)]
    #[pyo3(signature = (callback = None, /))]
    fn run_iteration(&self, py: Python<'_>, callback: Option<PyObject>) -> PyResult<()> {
        let app = self.0.try_lock_inner_mut()??;
        let py_app_handle = app.py_app_handle().clone_ref(py);
        unsafe {
            // Safety: `&mut tauri::App` does not hold the GIL, so this is safe
            py.allow_threads_unsend(app, |mut app| {
                match callback {
                    Some(callback) => {
                        app.run_iteration(Self::py_cb_to_rs_cb(callback, py_app_handle))
                    }
                    None => app.run_iteration(Self::noop_callback),
                }
                Ok(())
            })
        }
    }

    fn cleanup_before_exit(&self, py: Python<'_>) -> PyResult<()> {
        unsafe {
            // Safety: `self: &App` does not hold the GIL, so this is safe
            py.allow_threads_unsend(self, |slf| {
                let app = slf.0.try_lock_inner_ref()??;
                app.cleanup_before_exit();
                Ok(())
            })
        }
    }
}
