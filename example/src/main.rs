// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::env::var;

use pyo3::prelude::*;
use pyo3::wrap_pymodule;
use pytauri::standalone::{
    append_ext_mod, prepare_freethreaded_python, prepare_freethreaded_python_venv,
};

use _ext_mod::_ext_mod;

fn main() -> Result<(), PyErr> {
    if let Ok(venv_path) = var("VIRTUAL_ENV") {
        prepare_freethreaded_python_venv(venv_path).expect("failed to initialize python from venv");
    } else {
        prepare_freethreaded_python()
    }

    let settings = Settings::default();

    let interpreter = rustpython::InterpreterConfig::new()
        .init_hook(Box::new(|vm| {
            let python_libs = vm::py_freeze!(file = "../python/codelldb/src/codelldb/__init__.py", module_name = "codelldb");
            vm.add_frozen(python_libs);
        }))
        .init_stdlib()
        .settings(settings)
        .interpreter();

    let module = vm::py_compile!(file = "python/pytauri_demo/__main__.py");

    interpreter.run(|vm| {
        let scope = vm.new_scope_with_builtins();
        vm.run_code_obj(vm.ctx.new_code(module), scope)?;
        Ok(())
    });

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
