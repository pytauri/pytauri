use pyo3::prelude::*;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

pub fn tauri_generate_context() -> tauri::Context {
    tauri::generate_context!()
}

// 👉 Set `#[pymodule(gil_used = false)]` so that your extension module supports free-threading (no GIL) python,
// see: <https://pyo3.rs/v0.23.4/free-threading.html#supporting-free-threaded-python-with-pyo3>.
#[pymodule(gil_used = false)]
#[pyo3(name = "ext_mod")]
pub mod ext_mod {
    use super::*;

    #[pymodule_export]
    use pytauri_plugin_notification::notification;

    #[pymodule_init]
    fn init(module: &Bound<'_, PyModule>) -> PyResult<()> {
        pytauri::pymodule_export(
            module,
            // i.e., `context_factory` function of python binding
            |_args, _kwargs| Ok(tauri_generate_context()),
            // i.e., `builder_factory` function of python binding
            |_args, _kwargs| {
                let builder = tauri::Builder::default()
                    .plugin(tauri_plugin_opener::init())
                    .plugin(tauri_plugin_notification::init())
                    .invoke_handler(tauri::generate_handler![greet]);
                Ok(builder)
            },
        )
    }
}
