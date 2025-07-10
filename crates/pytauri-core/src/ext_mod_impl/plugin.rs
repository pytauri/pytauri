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
    pub fn new(plugin_fn: impl FnOnce() -> BoxedPlugin + Send + Sync + 'static) -> Self {
        Self(PyWrapper::new2(Box::new(plugin_fn)))
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

// TODO: remove this
pub struct BoxedPluginWrapper(pub BoxedPlugin);

impl plugin::Plugin<Runtime> for BoxedPluginWrapper {
    fn name(&self) -> &'static str {
        self.0.name()
    }
    fn initialize(
        &mut self,
        app: &tauri::AppHandle<Runtime>,
        config: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.0.initialize(app, config)
    }
    fn initialization_script(&self) -> Option<String> {
        self.0.initialization_script()
    }
    fn window_created(&mut self, window: tauri::Window<Runtime>) {
        self.0.window_created(window)
    }
    fn webview_created(&mut self, webview: tauri::Webview<Runtime>) {
        self.0.webview_created(webview)
    }
    fn on_navigation(&mut self, webview: &tauri::Webview<Runtime>, url: &tauri::Url) -> bool {
        self.0.on_navigation(webview, url)
    }
    fn on_page_load(
        &mut self,
        webview: &tauri::Webview<Runtime>,
        payload: &tauri::webview::PageLoadPayload<'_>,
    ) {
        self.0.on_page_load(webview, payload)
    }
    fn on_event(&mut self, app: &tauri::AppHandle<Runtime>, event: &tauri::RunEvent) {
        self.0.on_event(app, event)
    }
    fn extend_api(&mut self, invoke: tauri::ipc::Invoke<Runtime>) -> bool {
        self.0.extend_api(invoke)
    }
}
