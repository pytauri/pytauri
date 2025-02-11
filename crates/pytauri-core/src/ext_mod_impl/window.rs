use pyo3::prelude::*;
use pyo3_utils::py_wrapper::{PyWrapper, PyWrapperT0};
use tauri::window;

use crate::tauri_runtime::Runtime;

type TauriWindow = window::Window<Runtime>;

/// see also: [tauri::window::Window]
#[pyclass(frozen)]
#[non_exhaustive]
pub struct Window(pub PyWrapper<PyWrapperT0<TauriWindow>>);

impl Window {
    pub(crate) fn new(window: TauriWindow) -> Self {
        Self(PyWrapper::new0(window))
    }
}
