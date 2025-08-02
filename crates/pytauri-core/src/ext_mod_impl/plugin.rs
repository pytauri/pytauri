use pyo3::prelude::*;
use pyo3_utils::py_wrapper::{ConsumedResult, LockResult, PyWrapper, PyWrapperT2};
use tauri::plugin;

use crate::tauri_runtime::Runtime;

type BoxedPlugin = Box<dyn plugin::Plugin<Runtime>>;
type PluginFactory = Box<dyn FnOnce() -> BoxedPlugin + Send + Sync>;

/// See also: [tauri::plugin::Plugin].
#[pyclass(frozen)]
#[non_exhaustive]
pub struct Plugin(PyWrapper<PyWrapperT2<PluginFactory>>);

impl Plugin {
    pub fn new(plugin_fn: PluginFactory) -> Self {
        Self(PyWrapper::new2(plugin_fn))
    }

    /// Converts the Python plugin into a Tauri plugin.
    ///
    /// This method can only be called once.
    pub fn into_tauri(&self) -> LockResult<ConsumedResult<BoxedPlugin>> {
        self.0
            .try_take_inner()
            .map(|inner| inner.map(|plugin_fn| plugin_fn()))
    }
}
