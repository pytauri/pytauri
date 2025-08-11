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
pub mod standalone;

use pyo3::{
    exceptions::PyTypeError,
    prelude::*,
    types::{PyCFunction, PyDict, PyModule, PyTuple},
    wrap_pymodule,
};
use pyo3_utils::{
    from_py_dict::{derive_from_py_dict, FromPyDict as _, NotRequired},
    py_wrapper::{PyWrapper, PyWrapperT2},
};
use pytauri_core::{ext_mod::PyAppHandleExt as _, tauri_runtime::Runtime, utils::TauriError};

/// Use [pymodule_export] instead of this [ext_mod] and [pytauri_plugins] directly.
pub use pytauri_core::{ext_mod, pytauri_plugins};

type TauriBuilder = tauri::Builder<Runtime>;
type TauriContext = tauri::Context<Runtime>;

/// See also: [tauri::Builder]. And please refer to the Python-side documentation.
#[non_exhaustive]
pub struct BuilderArgs {
    /// see [`tauri_plugin_pytauri::init`] for `invoke_handler`.
    invoke_handler: Option<PyObject>,
    /// see [tauri::Builder::setup] and python side type hint.
    setup: NotRequired<PyObject>,
    /// see [tauri::Builder::plugin]
    plugins: NotRequired<Vec<Py<ext_mod::plugin::Plugin>>>,
}

derive_from_py_dict!(BuilderArgs {
    // We require `invoke_handler` as a required parameter,
    // see: <https://github.com/pytauri/pytauri/pull/133>.
    invoke_handler,
    #[default]
    setup,
    #[default]
    plugins,
});

impl BuilderArgs {
    fn from_kwargs(kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
        match kwargs {
            Some(kwargs) => Ok(BuilderArgs::from_py_dict(kwargs)?),
            // because [BuilderArgs::invoke_handler] is required,
            // so we return [PyErr] if `kwargs` is [None].
            None => Err(PyTypeError::new_err(
                "Missing required `**kwargs` arguments `BuilderArgs`",
            )),
        }
    }

    fn apply_to_builder(self, py: Python<'_>, mut builder: TauriBuilder) -> PyResult<TauriBuilder> {
        let Self {
            invoke_handler,
            setup,
            plugins,
        } = self;

        if let Some(invoke_handler) = invoke_handler {
            builder = builder.plugin(tauri_plugin_pytauri::init(invoke_handler.clone_ref(py)));
        }
        if let Some(setup) = setup.0 {
            builder = builder.setup(move |app| {
                Python::with_gil(|py| {
                    // we haven't called [ext_mod::App::try_build], so we need init the [PyAppHandle] before get it.
                    let app_handle = app.get_or_init_py_app_handle(py)?;
                    setup.call1(py, (app_handle,))?;
                    Ok(())
                })
            });
        }
        if let Some(plugins) = plugins.0 {
            for plugin in plugins {
                let plugin = plugin.get().into_tauri()??;
                builder = builder.plugin_boxed(plugin);
            }
        }

        Ok(builder)
    }
}

// TODO, FIXME, PERF, XXX: `tauri::Builder` is `!Sync`,
// we need wait pyo3 `pyclass(unsync)` feature,
// maybe we should file a issue to pyo3.
/// See also: [tauri::Builder]
#[pyclass(frozen, unsendable)]
#[non_exhaustive]
pub struct Builder(pub PyWrapper<PyWrapperT2<TauriBuilder>>);

impl Builder {
    fn new(builder: TauriBuilder) -> Self {
        Self(PyWrapper::new2(builder))
    }
}

#[pymethods]
impl Builder {
    #[pyo3(signature = (context, **kwargs))]
    fn build(
        &self,
        py: Python<'_>,
        context: Py<ext_mod::Context>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<ext_mod::App> {
        let context = context.get().0.try_take_inner()??;
        let args = BuilderArgs::from_kwargs(kwargs)?;

        let mut builder = self.0.try_take_inner()??;
        builder = args.apply_to_builder(py, builder)?;

        let app = builder.build(context).map_err(TauriError::from)?;
        ext_mod::App::try_build(py, app)
    }
}

/// Exports the [ext_mod] and [pytauri_plugins] module to the `parent_module`.
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
    context_factory: impl Fn(&Bound<'_, PyTuple>, Option<&Bound<'_, PyDict>>) -> PyResult<TauriContext>
        + Send
        + 'static,
    builder_factory: impl Fn(&Bound<'_, PyTuple>, Option<&Bound<'_, PyDict>>) -> PyResult<TauriBuilder>
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
    let pytauri_module: Py<PyModule> = wrap_pymodule!(ext_mod)(py);
    let pytauri_module = pytauri_module.bind(py);

    pytauri_module.add_function(builder_factory)?;
    pytauri_module.add_function(context_factory)?;
    pytauri_module.add_class::<Builder>()?;

    let pytauri_plugins_module = wrap_pymodule!(pytauri_plugins)(py);
    let pytauri_plugins_module = pytauri_plugins_module.bind(py);

    parent_module.add_submodule(pytauri_module)?;
    parent_module.add_submodule(pytauri_plugins_module)?;
    Ok(())
}
