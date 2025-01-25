use pyo3::prelude::*;

pub fn tauri_generate_context() -> tauri::Context {
    tauri::generate_context!()
}

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
                    .plugin(tauri_plugin_notification::init());
                Ok(builder)
            },
        )
    }
}
