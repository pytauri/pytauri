use pyo3::prelude::*;
use tauri::Emitter as _;

use crate::{
    ext_mod::{manager_method_impl, EventTarget, ImplManager},
    utils::{PyResultExt as _, TauriError},
};

/// The Implementers of [tauri::Emitter].
pub type ImplEmitter = ImplManager;

/// See also: [tauri::Emitter].
//
// `subclass` for implementing `emit`, `emit_to`, etc. methods from the Python side.
#[pyclass(frozen, subclass)]
#[non_exhaustive]
pub struct Emitter;

#[pymethods]
impl Emitter {
    #[staticmethod]
    fn emit_str(py: Python<'_>, slf: ImplEmitter, event: &str, payload: String) -> PyResult<()> {
        manager_method_impl!(py, &slf, [ungil], |manager| {
            manager.emit_str(event, payload).map_err(TauriError::from)
        })??;
        Ok(())
    }

    #[staticmethod]
    fn emit_str_to(
        py: Python<'_>,
        slf: ImplEmitter,
        target: Py<EventTarget>,
        event: &str,
        payload: String,
    ) -> PyResult<()> {
        let target = target.get().to_tauri(py)?;

        manager_method_impl!(py, &slf, [ungil], |manager| {
            manager
                .emit_str_to(target, event, payload)
                .map_err(TauriError::from)
        })??;
        Ok(())
    }

    #[staticmethod]
    fn emit_str_filter(
        py: Python<'_>,
        slf: ImplEmitter,
        event: &str,
        payload: String,
        filter: Bound<PyAny>,
    ) -> PyResult<()> {
        // We can't release the GIL here, because `rs_filter` will be used as `iter.filter(|..| rs_filter(..))`;
        // if we frequently release and acquire the GIL, maybe it will cause performance problems.
        // TODO, PERF: only tauri itself can release GIL in `emit_str_filter`.
        let rs_filter = |target: &tauri::EventTarget| -> bool {
            let target = EventTarget::from_tauri(py, target);
            let filter_result = filter.call1((target,));
            let filter_ret = filter_result.unwrap_unraisable_py_result(py, Some(&filter), || {
                "Python exception occurred in emitter filter"
            });
            let extract_result = filter_ret.extract::<bool>();
            extract_result.unwrap_unraisable_py_result(py, Some(&filter_ret), || {
                "emitter filter return non-bool value"
            })
        };

        manager_method_impl!(py, &slf, |_py, manager| {
            manager
                .emit_str_filter(event, payload, rs_filter)
                .map_err(TauriError::from)
        })??;
        Ok(())
    }
}
