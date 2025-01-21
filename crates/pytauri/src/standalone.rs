//! Used for embedding Python in a standalone pytauri application.
//!
//! In most cases you will use:
//!
//! - [prepare_freethreaded_python_with_executable]
//! - [append_ext_mod]
//!
//! # NOTE
//!
//! This module is licensed under the `MPL-2.0` license from [pyembed].
//!
//! [pyembed]: https://crates.io/crates/pyembed

use std::{
    env::{args_os, current_exe},
    ffi::OsString,
    fs, io,
    path::{absolute, Path, PathBuf},
};

use pyo3::{
    ffi::{self as pyffi, c_str},
    prelude::*,
    prepare_freethreaded_python,
    types::{PyDict, PyModule},
};

use crate::pyembed::{utils, NewInterpreterError, NewInterpreterResult};

#[non_exhaustive]
pub enum PyConfigProfile {
    Python,
    Isolated,
}

/// see: <https://docs.python.org/3/c-api/init_config.html#c.PyConfig>
pub struct PyConfig(pyffi::PyConfig);

/// NOTE: You can use [PyConfig] only before calling [pyo3::prepare_freethreaded_python],
/// otherwise it is an invalid operation
// ref: <https://github.com/indygreg/PyOxidizer/blob/1ceca8664c71f39e849ce4873e00d821504b32bd/pyembed/src/interpreter_config.rs#L252-L619>
impl PyConfig {
    pub fn new(profile: PyConfigProfile) -> Self {
        let mut config: pyffi::PyConfig = unsafe { std::mem::zeroed() };

        unsafe {
            match profile {
                PyConfigProfile::Isolated => pyffi::PyConfig_InitIsolatedConfig(&mut config),
                PyConfigProfile::Python => pyffi::PyConfig_InitPythonConfig(&mut config),
            }
        }

        Self(config)
    }

    pub fn set_prefix(&mut self, prefix: &Path) -> NewInterpreterResult<()> {
        unsafe {
            utils::set_config_string_from_path(&self.0, &self.0.prefix, prefix, "setting prefix")
        }
    }

    pub fn set_base_prefix(&mut self, base_prefix: &Path) -> NewInterpreterResult<()> {
        unsafe {
            utils::set_config_string_from_path(
                &self.0,
                &self.0.base_prefix,
                base_prefix,
                "setting base_prefix",
            )
        }
    }

    pub fn set_exec_prefix(&mut self, exec_prefix: &Path) -> NewInterpreterResult<()> {
        unsafe {
            utils::set_config_string_from_path(
                &self.0,
                &self.0.exec_prefix,
                exec_prefix,
                "setting exec_prefix",
            )
        }
    }

    pub fn set_base_exec_prefix(&mut self, base_exec_prefix: &Path) -> NewInterpreterResult<()> {
        unsafe {
            utils::set_config_string_from_path(
                &self.0,
                &self.0.base_exec_prefix,
                base_exec_prefix,
                "setting base_exec_prefix",
            )
        }
    }

    pub fn set_program_name(&mut self, program_name: &Path) -> NewInterpreterResult<()> {
        unsafe {
            utils::set_config_string_from_path(
                &self.0,
                &self.0.program_name,
                program_name,
                "setting program_name",
            )
        }
    }

    pub fn set_executable(&mut self, executable: &Path) -> NewInterpreterResult<()> {
        unsafe {
            utils::set_config_string_from_path(
                &self.0,
                &self.0.executable,
                executable,
                "setting executable",
            )
        }
    }

    pub fn set_argv(&mut self, args: &[OsString]) -> NewInterpreterResult<()> {
        utils::set_argv(&mut self.0, args)
    }

    pub fn set_parse_argv(&mut self, parse_argv: bool) {
        self.0.parse_argv = if parse_argv { 1 } else { 0 };
    }

    /// After calling this method, the Python interpreter has already been initialized,
    /// so [pyo3::prepare_freethreaded_python] is a no-op.
    /// But you still need to call it to let pyo3 know that the interpreter has been initialized.
    // ref: <https://github.com/indygreg/PyOxidizer/blob/1ceca8664c71f39e849ce4873e00d821504b32bd/pyembed/src/interpreter.rs#L130-L255>
    pub fn init(self) -> NewInterpreterResult<()> {
        let status = unsafe { pyffi::Py_InitializeFromConfig(&self.0) };
        if unsafe { pyffi::PyStatus_Exception(status) } != 0 {
            return Err(NewInterpreterError::new_from_pystatus(
                &status,
                "initializing Python core",
            ));
        }
        // The GIL is held.
        debug_assert_eq!(unsafe { pyffi::PyGILState_Check() }, 1);

        // We release the GIL so we can have pyo3's GIL handling take over from
        // an "empty" state. This mirrors what pyo3's prepare_freethreaded_python() does.
        unsafe {
            pyffi::PyEval_SaveThread();
        }
        Ok(())
    }
}

impl Drop for PyConfig {
    fn drop(&mut self) {
        unsafe {
            pyffi::PyConfig_Clear(&mut self.0);
        }
    }
}

/// Get the absolute path of the Python interpreter from the virtual environment path.
///
/// For Unix systems, it is equivalent to `venv_path / "bin/python3"`.
///
/// For Windows systems, it is equivalent to `venv_path / "Scripts/python.exe"`.
///
/// # Panics
///
/// Panics if [std::path::absolute] fails.
pub fn get_python_executable_from_venv(venv_path: impl Into<PathBuf>) -> PathBuf {
    let mut venv_path: PathBuf = venv_path.into();
    #[cfg(unix)]
    venv_path.push("bin/python3");
    #[cfg(windows)]
    venv_path.push(r"Scripts\python.exe");
    #[cfg(not(any(unix, windows)))]
    unimplemented!();
    // NOTE: Use [std::path::absolute] instead of [Path::canonicalize].
    // On Unix, the Python executable in the virtual environment is actually a symbolic link,
    // [canonicalize] will resolve the symbolic link,
    // causing the path to be set to the system-level Python interpreter.
    absolute(&venv_path).expect("failed to get absolute path")
}

/// Whether the Python interpreter is in "multiprocessing worker" mode.
///
/// The `multiprocessing` module can work by spawning new processes
/// with arguments `--multiprocessing-fork [key=value] ...`. This function
/// detects if the current Python interpreter is configured for said execution.
///
/// Useful if you want to use cil arg parsing lib like `clap` in your standalone app.
///
/// # NOTE
///
/// This function only work on Windows with `spawn` multiprocessing start method.
/// On Unix, you should use `fork` start method.
// ---
//
// ref: <https://github.com/indygreg/PyOxidizer/blob/ae36f8672d905a911f1b8243308fe45c5fe981de/pyembed/src/interpreter.rs#L582-L591>
pub fn is_forking() -> bool {
    let mut argv = args_os();
    if let Some(arg) = argv.nth(1) {
        arg == "--multiprocessing-fork"
    } else {
        false
    }
}

fn freeze(py: Python<'_>, current_exe: &Path) -> PyResult<()> {
    let locals = PyDict::new(py);
    locals.set_item("current_exe", current_exe)?;

    // TODO, PERF: compile into python bytecode.
    // see: <https://users.rust-lang.org/t/why-calling-python-from-rust-is-faster-than-python/39789/13>
    py.run(c_str!(include_str!("_freeze.py")), None, Some(&locals))
}

/// Prepare a Python interpreter with the specified Python executable.
///
/// NOTE: the `executable` must be an absolute path.
///
/// - [pyo3::prepare_freethreaded_python] can only be used with system-installed Python,
///   and not with virtual environments or standalone distributions like `python-build-standalone`.
///   This function allows you to do that.
/// - Also, this function correctly configures the `multiprocessing` module.
///
/// > This method will internally call [pyo3::prepare_freethreaded_python], so there is no need to call it manually.
///
/// # Behavior
///
/// > This is the behavior at the time of writing and may change in the future.
///
/// ref:
///
/// - <https://docs.python.org/3.13/c-api/intro.html#embedding-python>
/// - <https://github.com/python/cpython/blob/3.13/Modules/getpath.py>
///
/// behavior:
///
/// - Set `PyConfig.program_name` to `std::env::current_exe()`.
/// - Set `PyConfig.executable` to the specified `executable`.
/// - Set `PyConfig.argv` to `std::env::args_os()`.
/// - Set `PyConfig.parse_argv` to `false`.
/// - Set `sys.frozen` to `True`.
/// - Call `multiprocessing.set_start_method` with
///     - windows: `spawn`
///     - unix: `fork`
/// - Call `multiprocessing.set_executable` with `std::env::current_exe()`
///
/// # Example
/**
```no_run
use pyo3::prelude::*;
use pytauri::standalone::prepare_freethreaded_python_with_executable;

prepare_freethreaded_python_with_executable("/my/python")
    .expect("failed to prepare python interpreter");
Python::with_gil(|_py| {
    // Your code here
});
```
*/
pub fn prepare_freethreaded_python_with_executable(
    executable: impl AsRef<Path>,
) -> NewInterpreterResult<()> {
    let py_executable = executable.as_ref();
    let current_exe = current_exe().map_err(|e| {
        NewInterpreterError::Dynamic(format!("failed to get the current executable path: {}", e))
    })?;

    let mut config = PyConfig::new(PyConfigProfile::Python);

    // 👇 Init config ref:
    // - <https://github.com/python/cpython/blob/3.13/Modules/getpath.py>
    // - <https://docs.python.org/3.13/c-api/init_config.html#python-path-configuration>
    // - <https://docs.python.org/3.13/c-api/intro.html#embedding-python>

    // in fact, unnecessary, python will get it from `argv[0]`
    config.set_program_name(&current_exe)?;
    // necessary for finding python home
    config.set_executable(py_executable)?;
    // necessary for `multiprocessing`
    config.set_argv(&args_os().collect::<Vec<_>>())?;
    // `parse_argv=false` is necessary, because python only accepts following argv pattern:
    //
    // ```shell
    // # <https://docs.python.org/3/using/cmdline.html#using-on-cmdline>
    // python [-bBdEhiIOPqRsSuvVWx?] [-c command | -m module-name | script | - ] [args]
    // ```
    //
    // This will prevent us from using libraries like `clap` to parse command line arguments
    config.set_parse_argv(false);
    config.init()?;

    prepare_freethreaded_python();

    Python::with_gil(|py| {
        freeze(py, &current_exe).map_err(|e| {
            NewInterpreterError::new_from_pyerr(py, e, "failed to freeze the Python interpreter")
        })
    })?;

    Ok(())
}

/// Insert the `pytauri` extension module into `sys.modules`,
/// otherwise your Python code cannot import `pytauri` when built as a standalone application.
///
/// Also, this function will call `multiprocessing.freeze_support()`.
///
/// <div class="warning">
///
/// For child processes spawned by `multiprocessing`,
/// `multiprocessing.freeze_support()` will execute the [`Process`],
/// and after execution, it will raise a [`SystemExit`] exception to inform the Python interpreter to exit.
///
/// This exception will be returned as [pyo3::exceptions::PySystemExit]. When you catch this exception,
/// you should let your Rust code exit and not continue executing your Python app code, otherwise you will get
/// an endless spawn loop of the application process.
/// See: <https://pyinstaller.org/en/stable/common-issues-and-pitfalls.html#multi-processing>.
///
/// > You need to do this only on Windows, because on Unix, we use the `fork` start method instead.
///
/// </div>
///
/// [`Process`]: https://docs.python.org/3/library/multiprocessing.html#multiprocessing.Process
/// [`SystemExit`]: https://docs.python.org/3/library/exceptions.html#SystemExit
///
/// # Example
/**
```no_run
use pyo3::{exceptions::PySystemExit, prelude::*, wrap_pymodule};
use pytauri::standalone::{append_ext_mod, is_forking};

// Your extension module
#[pymodule]
mod ext_mod {}

fn main() -> PyResult<()> {
    Python::with_gil(|py| {
        let ext_mod = wrap_pymodule!(ext_mod)(py).into_bound(py);

        if let Err(err) = append_ext_mod(ext_mod) {
            if err.is_instance_of::<PySystemExit>(py) && is_forking() {
                // just return to end the rust code normally,
                // don't execute your python app code.
                return Ok(());
            } else {
                return Err(err);
            }
        }

        // Or you can just return the error and handle it later.
        // Just dont execute your python app code is enough.
        //
        // ```rust
        // append_ext_mod(ext_mod)?;
        // ```


        // Your python app code here
        py.run(c"print('Hello, world!')", None, None)?;

        Ok(())
    })
}
```
*/
pub fn append_ext_mod(ext_mod: Bound<PyModule>) -> PyResult<()> {
    let py = ext_mod.py();
    let locals = PyDict::new(py);
    locals.set_item("ext_mod", ext_mod)?;

    // TODO, PERF: compile into python bytecode.
    // see: <https://users.rust-lang.org/t/why-calling-python-from-rust-is-faster-than-python/39789/13>
    py.run(
        c_str!(include_str!("_append_ext_mod.py")),
        None,
        Some(&locals),
    )
}

pub fn write_py_err_to_file(
    py: Python<'_>,
    py_err: &PyErr,
    file_path: impl AsRef<Path>,
) -> io::Result<()> {
    if let Some(tb) = py_err.traceback(py) {
        if let Ok(tb) = tb.format() {
            return fs::write(file_path, tb);
        }
    }
    fs::write(file_path, format!("{:?}", py_err))
}
