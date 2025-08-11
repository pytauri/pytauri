// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{convert::Infallible, error::Error, path::PathBuf};

use pyo3::wrap_pymodule;
use pytauri::standalone::{
    dunce::simplified, PythonInterpreterBuilder, PythonInterpreterEnv, PythonScript,
};
use tauri::utils::platform::resource_dir;

use tauri_app_lib::{ext_mod, tauri_generate_context};

fn main() -> Result<Infallible, Box<dyn Error>> {
    let py_env = if cfg!(dev) {
        // `cfg(dev)` is set by `tauri-build` in `build.rs`, which means running with `tauri dev`,
        // see: <https://github.com/tauri-apps/tauri/pull/8937>.

        // NOTE: We cannot rely on the `VIRTUAL_ENV` environment variable,
        // because when launched by `tauri-plugin-deep-link`, this env var is not set.
        //
        // `{repo_root}/examples/tauri-app/src-tauri`
        let mut venv_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        venv_dir.pop();
        venv_dir.pop();
        venv_dir.pop();
        // `{repo_root}/.venv`
        venv_dir.push(".venv");
        assert!(
            venv_dir.is_dir(),
            "Python virtual environment not found at: {}",
            venv_dir.display()
        );

        PythonInterpreterEnv::Venv(venv_dir.into())
    } else {
        // embedded Python, i.e., bundle mode with `tauri build`.

        let context = tauri_generate_context();
        let resource_dir = resource_dir(context.package_info(), &tauri::Env::default())
            .map_err(|err| format!("failed to get resource dir: {err}"))?;
        // 👉 Remove the UNC prefix `\\?\`, Python ecosystems don't like it.
        let resource_dir = simplified(&resource_dir).to_owned();

        // 👉 When bundled as a standalone App, we will put python in the resource directory
        PythonInterpreterEnv::Standalone(resource_dir.into())
    };

    // 👉 Equivalent to `python -m tauri_app`,
    // i.e, run the `src-tauri/python/tauri_app/__main__.py`
    let py_script = PythonScript::Module("tauri_app".into());

    // 👉 `ext_mod` is your extension module, we export it from memory,
    // so you don't need to compile it into a binary file (.pyd/.so).
    let builder =
        PythonInterpreterBuilder::new(py_env, py_script, |py| wrap_pymodule!(ext_mod)(py));
    let interpreter = builder.build()?;

    let exit_code = interpreter.run();
    std::process::exit(exit_code);
}
