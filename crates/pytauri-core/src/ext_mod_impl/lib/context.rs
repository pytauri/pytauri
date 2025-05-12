use pyo3::prelude::*;
use pyo3_utils::py_wrapper::{PyWrapper, PyWrapperT2};

use crate::{ext_mod::PyAssets, tauri_runtime::Runtime};

type TauriContext = tauri::Context<Runtime>;

/// see also: [tauri::Context]
#[pyclass(frozen)]
#[non_exhaustive]
pub struct Context(pub PyWrapper<PyWrapperT2<TauriContext>>);

impl Context {
    pub fn new(context: TauriContext) -> Self {
        Self(PyWrapper::new2(context))
    }
}

#[pymethods]
impl Context {
    fn set_assets(&self, py: Python<'_>, assets: PyObject) -> PyResult<()> {
        py.allow_threads(|| {
            let mut context = self.0.try_lock_inner_mut()??;
            context.set_assets(Box::new(PyAssets(assets)));
            Ok(())
        })
    }
}
