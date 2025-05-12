use std::borrow::Cow;

use pyo3::{prelude::*, types::PyString};
use tauri::Listener as _;

use crate::{
    ext_mod::{manager_method_impl, Event, EventId, ImplManager},
    utils::PyResultExt as _,
};

/// The Implementers of [tauri::Listener].
pub type ImplListener = ImplManager;

/// See also: [tauri::Listener].
#[pyclass(frozen)]
#[non_exhaustive]
pub struct Listener;

impl Listener {
    fn pyobj_to_handler(pyobj: PyObject) -> impl Fn(tauri::Event) + Send + 'static {
        move |event| {
            Python::with_gil(|py| {
                let event = Event {
                    id: event.id(),
                    payload: PyString::new(py, event.payload()).unbind(),
                };
                let pyobj = pyobj.bind(py);
                let result = pyobj.call1((event,));
                result.unwrap_unraisable_py_result(py, Some(pyobj), || {
                    "Python exception occurred in `Listener` handler"
                });
            })
        }
    }
}

#[pymethods]
impl Listener {
    #[staticmethod]
    fn listen(
        py: Python<'_>,
        slf: ImplListener,
        event: Cow<'_, str>,
        handler: PyObject,
    ) -> PyResult<EventId> {
        manager_method_impl!(py, &slf, [ungil], |manager| manager
            .listen(event, Self::pyobj_to_handler(handler)))
    }

    #[staticmethod]
    fn once(
        py: Python<'_>,
        slf: ImplListener,
        event: Cow<'_, str>,
        handler: PyObject,
    ) -> PyResult<EventId> {
        manager_method_impl!(py, &slf, [ungil], |manager| manager
            .once(event, Self::pyobj_to_handler(handler)))
    }

    #[staticmethod]
    fn unlisten(py: Python<'_>, slf: ImplListener, id: EventId) -> PyResult<()> {
        manager_method_impl!(py, &slf, [ungil], |manager| manager.unlisten(id))
    }

    #[staticmethod]
    fn listen_any(
        py: Python<'_>,
        slf: ImplListener,
        event: Cow<'_, str>,
        handler: PyObject,
    ) -> PyResult<EventId> {
        manager_method_impl!(py, &slf, [ungil], |manager| manager
            .listen_any(event, Self::pyobj_to_handler(handler)))
    }

    #[staticmethod]
    fn once_any(
        py: Python<'_>,
        slf: ImplListener,
        event: Cow<'_, str>,
        handler: PyObject,
    ) -> PyResult<EventId> {
        manager_method_impl!(py, &slf, [ungil], |manager| manager
            .once_any(event, Self::pyobj_to_handler(handler)))
    }
}
