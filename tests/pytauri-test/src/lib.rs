#[cfg(feature = "test")]
pub mod test {
    use pyo3::prelude::*;
    use tauri::test::mock_builder;

    pub use pytauri_core::tauri_runtime::Runtime;

    pub fn tauri_generate_context() -> tauri::Context<Runtime> {
        tauri::generate_context!()
    }

    #[pymodule(gil_used = false)]
    #[pyo3(name = "ext_mod")]
    pub mod ext_mod {
        use super::*;

        #[pymodule_init]
        fn init(module: &Bound<'_, PyModule>) -> PyResult<()> {
            pytauri::pymodule_export(
                module,
                |_args, _kwargs| Ok(tauri_generate_context()),
                |_args, _kwargs| {
                    let builder = mock_builder();
                    Ok(builder)
                },
            )
        }
    }
}
