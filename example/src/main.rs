// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env::var;

use pyo3::prelude::*;
use pyo3::wrap_pymodule;
use pytauri::standalone::{
    append_ext_mod, prepare_freethreaded_python, prepare_freethreaded_python_venv,
};

use std::{error::Error, path::PathBuf, process::Command};
use tauri::{Env, Wry};

use _ext_mod::_ext_mod;

fn main() -> Result<(), PyErr> {
    if let Ok(venv_path) = var("VIRTUAL_ENV") {
        prepare_freethreaded_python_venv(venv_path).expect("failed to initialize python from venv");
    } else {
        prepare_freethreaded_python()
    }

    // let settings = Settings::default();

    // // This is kinda risky to be honest, not garunteed to be supported 
    // let context: tauri::Context<Wry> = tauri::generate_context!();
    // let package_info = context.package_info();
    // println!("package_info: {:?}", package_info);
    // let resource_dir = tauri_utils::platform::resource_dir(package_info, &Env::default())?;

    // let mut python_path = PathBuf::from(resource_dir);
    // python_path.push("front");
    // python_path.push("resources");
    // python_path.push("cpython-aarch64-apple-darwin");
    // python_path.push("bin");
    // python_path.push("python");

    // println!("python_path: {:?}", python_path.display());

    // let _ = Command::new(python_path)
    //     .arg("--help")
    //     .spawn()?;

    Python::with_gil(|py| {
        let script = || {
            append_ext_mod(wrap_pymodule!(_ext_mod)(py).into_bound(py))?;

            // Run your python code here
            Python::run(
                py,
                // equal to `python -m pytauri_demo`
                c"from runpy import run_module; run_module('pytauri_demo')",
                None,
                None,
            )?;

            Ok::<_, PyErr>(())
        };

        script().inspect_err(|e| {
            #[cfg(all(not(debug_assertions), windows))]
            {
                // In this case, there is no console to print the error, so we write the error to a file
                use pytauri::standalone::write_py_err_to_file;
                write_py_err_to_file(py, &e, "error.log").expect("failed to write error to file");
            }
            #[cfg(not(all(not(debug_assertions), windows)))]
            {
                e.print_and_set_sys_last_vars(py);
            }
        })
    })
}
