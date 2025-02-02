//! Pay attention to this module's:
//!
//! - [pymodule_export]
//! - [standalone]

// See: <https://doc.rust-lang.org/rustdoc/unstable-features.html#extensions-to-the-doc-attribute>
#![cfg_attr(
    docsrs,
    feature(doc_cfg, doc_auto_cfg, doc_cfg_hide),
    doc(cfg_hide(doc))
)]

#[cfg(feature = "standalone")]
mod pyembed;
#[cfg(feature = "standalone")]
pub mod standalone;

use pyo3::prelude::*;
use pyo3::types::{PyCFunction, PyDict, PyModule, PyTuple};
use pyo3::wrap_pymodule;
use pyo3_utils::py_wrapper::{PyWrapper, PyWrapperT2};
use pytauri_core::tauri_runtime::Runtime;
use pytauri_core::utils::TauriError;
use tauri::Context;

/// Use [pymodule_export] instead of this `ext_mod` directly.
pub use pytauri_core::ext_mod;

/// Please refer to the Python-side documentation
#[pyclass(frozen)]
#[non_exhaustive]
#[derive(Debug)]
// Currently, we do not need to store large memory data,
// so we do not need `PyWrapperT2` to take ownership;
// Since we only store `pyobject`, `clone_ref` will be more efficient.
// If we need to store large memory data in the future,
// we can consider `PyWrapperT2` again.
pub struct BuilderArgs {
    context: Py<ext_mod::Context>,
    /// see [`tauri_plugin_pytauri::init`] for `invoke_handler`
    invoke_handler: Option<PyObject>,
}

#[pymethods]
impl BuilderArgs {
    #[new]
    #[pyo3(signature = (*, context, invoke_handler = None))]
    fn new(context: Py<ext_mod::Context>, invoke_handler: Option<PyObject>) -> Self {
        Self {
            context,
            invoke_handler,
        }
    }
}

// TODO, FIXME, PERF, XXX: `tauri::Builder` is `!Sync`,
// we need wait pyo3 `pyclass(unsync)` feature,
// maybe we should file a issue to pyo3.
#[pyclass(frozen, unsendable)]
#[non_exhaustive]
pub struct Builder(pub PyWrapper<PyWrapperT2<tauri::Builder<Runtime>>>);

impl Builder {
    fn new(builder: tauri::Builder<Runtime>) -> Self {
        Self(PyWrapper::new2(builder))
    }
}

#[pymethods]
impl Builder {
    fn build(&self, args: Bound<BuilderArgs>) -> PyResult<ext_mod::App> {
        let py = args.py();

        let mut builder = self.0.try_take_inner()??;

        let BuilderArgs {
            context,
            invoke_handler,
        } = args.get();

        if let Some(invoke_handler) = invoke_handler {
            builder = builder.plugin(tauri_plugin_pytauri::init(invoke_handler.clone_ref(py)));
        }

        let context = context.get().0.try_take_inner()??;
        let app = builder.build(context).map_err(Into::<TauriError>::into)?;
        ext_mod::App::try_build(py, app)
    }
}

/// Exports the [ext_mod] module to the `parent_module`.
///
/// `context_factory` and `builder_factory` will be exported as the
/// `pytauri.context_factory` and `pytauri.builder_factory` functions on the
/// Python side.
///
/// # Example
/**
```ignore
use pyo3::prelude::*;

#[pymodule(gil_used = false)]
#[pyo3(name = "_ext_mod")]
pub mod _ext_mod {
    use super::*;

    #[pymodule_init]
    fn init(module: &Bound<'_, PyModule>) -> PyResult<()> {
        pytauri::pymodule_export(
            module,
            |_args, _kwargs| Ok(tauri::generate_context!()),
            |_args, _kwargs| {
                let builder = tauri::Builder::default();
                // do whatever you want with the builder
                Ok(builder)
            },
        )
    }
}
```
*/
pub fn pymodule_export(
    parent_module: &Bound<'_, PyModule>,
    // TODO: make `context_factory` optional and get `Context` from python side
    context_factory: impl Fn(&Bound<'_, PyTuple>, Option<&Bound<'_, PyDict>>) -> PyResult<Context<Runtime>>
        + Send
        + 'static,
    builder_factory: impl Fn(&Bound<'_, PyTuple>, Option<&Bound<'_, PyDict>>) -> PyResult<tauri::Builder<Runtime>>
        + Send
        + 'static,
) -> PyResult<()> {
    let py = parent_module.py();

    let builder_factory =
        PyCFunction::new_closure(py, Some(c"builder_factory"), None, move |args, kwargs| {
            builder_factory(args, kwargs).map(Builder::new)
        })?;

    let context_factory =
        PyCFunction::new_closure(py, Some(c"context_factory"), None, move |args, kwargs| {
            context_factory(args, kwargs).map(ext_mod::Context::new)
        })?;

    // TODO, FIXME: The return type of `wrap_pymodule` is a private detail.
    // We should file an issue with pyo3 to inquire about this matter.
    let self_module: Py<PyModule> = wrap_pymodule!(ext_mod)(py);
    let self_module = self_module.bind(py);

    self_module.add_function(builder_factory)?;
    self_module.add_function(context_factory)?;
    self_module.add_class::<BuilderArgs>()?;
    self_module.add_class::<Builder>()?;

    parent_module.add_submodule(self_module)?;
    Ok(())
}
