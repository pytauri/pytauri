//! Used for embedding Python in a standalone pytauri application.
//!
//! In most cases you will use:
//!
//! - [PythonInterpreterBuilder]
//! - [PythonInterpreter]
//! - [is_forking]
//!
//! # NOTE
//!
//! This module is licensed under the `MPL-2.0` license from [pyembed].
//!
//! [pyembed]: https://crates.io/crates/pyembed

use std::{
    borrow::Cow,
    env::{args_os, current_exe},
    ffi::OsString,
    ops::Drop,
    path::Path,
};

use pyo3::{
    ffi::{self as pyffi, c_str},
    prelude::*,
    types::{PyDict, PyModule},
};

use crate::pyembed::utils;
pub use crate::pyembed::{NewInterpreterError, NewInterpreterResult};

#[non_exhaustive]
enum PyConfigProfile {
    Python,
    #[expect(dead_code)]
    Isolated,
}

/// see: <https://docs.python.org/3/c-api/init_config.html#c.PyConfig>
struct PyConfig(pyffi::PyConfig);

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

    pub fn set_home(&mut self, home: &Path) -> NewInterpreterResult<()> {
        unsafe { utils::set_config_string_from_path(&self.0, &self.0.home, home, "setting home") }
    }

    #[expect(dead_code)]
    pub fn set_prefix(&mut self, prefix: &Path) -> NewInterpreterResult<()> {
        unsafe {
            utils::set_config_string_from_path(&self.0, &self.0.prefix, prefix, "setting prefix")
        }
    }

    #[expect(dead_code)]
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

    #[expect(dead_code)]
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

    #[expect(dead_code)]
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

    pub fn set_run_command(&mut self, run_command: &str) -> NewInterpreterResult<()> {
        unsafe {
            utils::set_config_string_from_str(
                &self.0,
                &self.0.run_command,
                run_command,
                "setting run_command",
            )
        }
    }

    pub fn set_run_module(&mut self, run_module: &str) -> NewInterpreterResult<()> {
        unsafe {
            utils::set_config_string_from_str(
                &self.0,
                &self.0.run_module,
                run_module,
                "setting run_module",
            )
        }
    }

    pub fn set_run_filename(&mut self, run_filename: &Path) -> NewInterpreterResult<()> {
        unsafe {
            utils::set_config_string_from_path(
                &self.0,
                &self.0.run_filename,
                run_filename,
                "setting run_filename",
            )
        }
    }

    /// Initialize the Python interpreter with the specified configuration.
    ///
    /// - If the Python interpreter is already initialized (e.g, called [pyo3::prepare_freethreaded_python] before),
    ///   this method will return an error.
    ///
    /// - After calling this method, the Python interpreter has already been initialized,
    ///   and you dont need to call [pyo3::prepare_freethreaded_python] again (it's no-op).
    ///
    /// - If this function returns an error, the Python interpreter is not initialized.
    //
    // ref: <https://github.com/indygreg/PyOxidizer/blob/1ceca8664c71f39e849ce4873e00d821504b32bd/pyembed/src/interpreter.rs#L130-L255>
    pub fn init(self) -> NewInterpreterResult<()> {
        if PythonInterpreter::is_initialized() {
            return Err(NewInterpreterError::Simple(
                "Python interpreter has already been initialized",
            ));
        }

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

/// Clear the `PyConfig` to release memory.
impl Drop for PyConfig {
    fn drop(&mut self) {
        unsafe {
            pyffi::PyConfig_Clear(&mut self.0);
        }
    }
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

fn _post_init_pyi(
    py: Python<'_>,
    current_exe: &Path,
    ext_mod: Py<PyModule>,
) -> NewInterpreterResult<()> {
    let script = || {
        let locals = PyDict::new(py);
        locals.set_item("CURRENT_EXE", current_exe)?;
        locals.set_item("EXT_MOD", ext_mod)?;

        // TODO, PERF: compile into python bytecode.
        // see: <https://users.rust-lang.org/t/why-calling-python-from-rust-is-faster-than-python/39789/13>
        py.run(
            c_str!(include_str!("_post_init_pyi.py")),
            None,
            Some(&locals),
        )
    };
    script().map_err(|e| {
        NewInterpreterError::new_from_pyerr(py, e, "failed to post init python interpreter")
    })
}

/// The python interpreter environment you want to use.
///
/// NOTE: This path **must be absolute**.
#[non_exhaustive]
pub enum PythonInterpreterEnv<'a> {
    /// The virtual environment python `root` directory.
    ///
    /// You can get it from env var `VIRTUAL_ENV` if you activated the venv.
    ///
    /// ## Windows
    ///
    /// ```text
    /// root\
    /// ├── Lib\
    /// ├── Scripts\
    /// ├── pyvenv.cfg
    /// ├── ...
    /// ```
    ///
    /// ## Unix
    ///
    /// ```text
    /// root/
    /// ├── bin/
    /// ├── lib/
    /// ├── pyvenv.cfg
    /// ├── ...
    /// ```
    Venv(Cow<'a, Path>),
    /// [python-build-standalone] distribution `root` directory.
    ///
    /// [python-build-standalone]: https://github.com/astral-sh/python-build-standalone
    ///
    /// # Note
    ///
    /// If you get the directory via [tauri::path::PathResolver::resource_dir],
    /// you had better remove the UNC prefix `\\?\` by [self::dunce].
    ///
    /// ## Windows
    ///
    /// ```text
    /// root\
    /// ├── Lib\
    /// ├── python.exe
    /// ├── python3.dll
    /// ├── ...
    /// ```
    ///
    /// ## Unix
    ///
    /// ```text
    /// root/
    /// ├── bin/
    /// ├── lib/
    /// ...
    /// ```
    Standalone(Cow<'a, Path>),
}

impl PythonInterpreterEnv<'_> {
    // ref:
    // - <https://docs.python.org/3.13/c-api/init_config.html#python-path-configuration>
    // - <https://github.com/python/cpython/blob/3.13/Modules/getpath.py>
    fn set_path_for_config(self, config: &mut PyConfig) -> NewInterpreterResult<()> {
        // necessary, because:
        // 1. make sure that `sys.executable` is actually the python executable
        // 2. python can calculate other path such as `PyConfig.prefix`, from `PyConfig.executable`.
        //     ref: <https://github.com/python/cpython/blob/3.13/Modules/getpath.py>
        let executable;
        // Normally we don't need to set `prefix` and `exec_prefix`:
        // - For `Venv`: python can rely on `pyvenv.cfg` to set it.
        //     see: <https://github.com/python/cpython/blob/a3797492179c249417a06d2499a7d535d453ac2c/Modules/getpath.py#L347-L403>
        // - For `Standalone`: python can rely on `executable` and linked `libpython` path to automatically guess it.
        //     see: <https://github.com/python/cpython/blob/a3797492179c249417a06d2499a7d535d453ac2c/Modules/getpath.py#L559-L573>.
        //     In most cases, this works fine because the `libpython` we link is right next to the standard library
        //     (refer to the `python-standalone-build` file layout).
        //
        // But for `AppImage` packaging,
        // `libpython` will be [moved to `${APPDIR}/usr/lib/`](https://github.com/tauri-apps/tauri/issues/11898),
        // instead of the expected [`${APPDIR}/usr/lib/{your-app-name}/lib`](https://wsh032.github.io/pytauri/latest/usage/tutorial/build-standalone/#unix_3).
        // So we need to manually set `prefix` and `exec_prefix`.
        // (as long as we set it correctly, it won't have much impact compared to automatic guessing).
        //
        // Note:
        // - we prefer set `home` instead of setting `prefix/exec_prefix` directly,
        //   because `home` is [`input field`](https://docs.python.org/3.13/c-api/init_config.html#python-path-configuration)
        // - We can set `exec_prefix` the same as `prefix`, it's because:
        //     - On Windows, they are always the same: <https://github.com/python/cpython/blob/eed7865ceea83f56e46307c9dc78cb53526071f6/Modules/getpath.py#L608-L612>
        //     - On unix, the `python-build-standalone` has the [{prefix/}{PLATSTDLIB_LANDMARK}](https://github.com/python/cpython/blob/eed7865ceea83f56e46307c9dc78cb53526071f6/Modules/getpath.py#L617C9-L617C85),
        //       which indicates that `prefix` and `exec_prefix` are the same.
        let home;
        match self {
            PythonInterpreterEnv::Venv(dir) => {
                executable = if cfg!(windows) {
                    dir.join(r"Scripts\python.exe")
                } else {
                    dir.join("bin/python3")
                };

                home = None;
            }
            PythonInterpreterEnv::Standalone(dir) => {
                executable = if cfg!(windows) {
                    dir.join("python.exe")
                } else {
                    dir.join("bin/python3")
                };

                home = Some(dir);
            }
        }

        config.set_executable(&executable)?;
        if let Some(home) = home {
            config.set_home(&home)?;
        }

        Ok(())
    }
}

/// Indicates how to run your Python code, used by [PythonInterpreter::run].
#[non_exhaustive]
pub enum PythonScript<'a> {
    /// Filename passed on the command line.
    /// For example, it is set to `script.py` by the `python3 script.py` arg command line.
    File(Cow<'a, Path>),
    /// Value of the `-m` command line option.
    /// For example, it is set to `module` by the `python3 -m module` arg command line.
    Module(Cow<'a, str>),
    /// Value of the `-c` command line option.
    /// For example, it is set to `code` by the `python3 -c code` arg command line.
    Code(Cow<'a, str>),
    /// If you don't set anything, `Py_RunMain` will runs the interactive Python prompt (REPL)
    /// using the `__main__` module’s global namespace. Usually just used for debugging.
    REPL,
}

/// Build a Python interpreter for your script.
///
/// # Behavior
///
/// > This is the behavior at the time of writing and may change in the future.
///
/// - Set `PyConfig.program_name` to `std::env::current_exe()`.
/// - Set `sys.executable` to the actual python interpreter executable path.
/// - Set `PyConfig.home` to [PythonInterpreterEnv::Standalone::0].
/// - Set `sys.argv` to `std::env::args_os()`.
/// - Set `PyConfig.parse_argv` to `false`.
/// - Set `sys.frozen` to `True`.
/// - Call `multiprocessing.set_start_method` with
///     - windows: `spawn`
///     - unix: `fork`
/// - Call `multiprocessing.set_executable` with `std::env::current_exe()`
#[non_exhaustive]
pub struct PythonInterpreterBuilder<'a, M>
where
    M: for<'py> FnOnce(Python<'py>) -> Py<PyModule> + 'a,
{
    env: PythonInterpreterEnv<'a>,
    script: PythonScript<'a>,
    ext_mod: M,
}

impl<'a, M> PythonInterpreterBuilder<'a, M>
where
    M: for<'py> FnOnce(Python<'py>) -> Py<PyModule> + 'a,
{
    /// # Example
    /**
    ```rust
    use pyo3::{prelude::*, wrap_pymodule};
    use pytauri::standalone::{PythonInterpreterBuilder, PythonInterpreterEnv, PythonScript};
    use std::path::Path;

    // Your extension module
    #[pymodule]
    mod ext_mod {}

    let env = PythonInterpreterEnv::Venv(From::<&Path>::from("/home/myvenv/".as_ref()));
    let script = PythonScript::Code("print('Hello, world!')".into());

    let builder = PythonInterpreterBuilder::new(env, script, |py| wrap_pymodule!(ext_mod)(py));
    ```
    */
    pub fn new(env: PythonInterpreterEnv<'a>, script: PythonScript<'a>, ext_mod: M) -> Self {
        PythonInterpreterBuilder {
            env,
            script,
            ext_mod,
        }
    }

    /// Build the Python interpreter.
    ///
    /// After calling this function, the Python interpreter is initialized.
    /// And you don't need to call [pyo3::prepare_freethreaded_python] again (it's no-op).
    ///
    /// NOTE: you can only build only one Python interpreter per process,
    /// or you will get a [NewInterpreterError].
    pub fn build(self) -> NewInterpreterResult<PythonInterpreter> {
        let current_exe = current_exe().map_err(|e| {
            NewInterpreterError::Dynamic(format!(
                "failed to get the current executable path: {}",
                e
            ))
        })?;

        let mut config = PyConfig::new(PyConfigProfile::Python);

        // 👇 Init config ref:
        // - <https://github.com/python/cpython/blob/3.13/Modules/getpath.py>
        // - <https://docs.python.org/3.13/c-api/init_config.html#python-path-configuration>
        // - <https://docs.python.org/3.13/c-api/intro.html#embedding-python>

        // in fact, unnecessary, python will get it from `argv[0]`
        config.set_program_name(&current_exe)?;
        // necessary for finding the standard library and installed libraries
        self.env.set_path_for_config(&mut config)?;
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

        match self.script {
            PythonScript::File(path) => {
                config.set_run_filename(&path)?;
            }
            PythonScript::Module(module) => {
                config.set_run_module(&module)?;
            }
            PythonScript::Code(code) => {
                config.set_run_command(&code)?;
            }
            PythonScript::REPL => {
                // if we don't set any of the above, `Py_RunMain` will run the REPL
            }
        }

        let interpreter = PythonInterpreter::new(config)?;
        interpreter.with_gil(|py| _post_init_pyi(py, &current_exe, (self.ext_mod)(py)))?;

        Ok(interpreter)
    }
}

/// The Python interpreter to run Python code.
///
/// # Safety
///
/// Dropping a [PythonInterpreter] instance will call `Py_FinalizeEx()` to
/// finalize the Python interpreter and prevent it from running any more Python
/// code.
///
/// If a Python C API is called after interpreter finalization, a segfault can
/// occur.
///
/// If you use pyo3 APIs like [Python::with_gil()] directly, you may
/// inadvertently attempt to operate on a finalized interpreter. Therefore
/// it is recommended to always go through a method on an [PythonInterpreter]
/// instance in order to interact with the Python interpreter.
#[non_exhaustive]
pub struct PythonInterpreter {}

impl PythonInterpreter {
    fn new(config: PyConfig) -> NewInterpreterResult<Self> {
        // [PyConfig::init()] need make sure if it failed, the interpreter is not initialized.
        // So we can just return here and dont need finalize the interpreter.
        config.init()?;

        let slf = Self {};
        Ok(slf)
    }

    fn is_initialized() -> bool {
        unsafe { pyffi::Py_IsInitialized() != 0 }
    }

    /// Runs `Py_RunMain()` and finalizes the interpreter.
    ///
    /// This will execute [PythonScript] and return an integer suitable
    /// for use as a process exit code.
    ///
    /// Calling this function will finalize the interpreter and only gives you an
    /// exit code: there is no opportunity to inspect the return value or handle
    /// an uncaught exception. If you want to keep the interpreter alive or inspect
    /// the evaluation result, consider using [Self::with_gil()] instead.
    //
    // ref: <https://github.com/indygreg/PyOxidizer/blob/1ceca8664c71f39e849ce4873e00d821504b32bd/pyembed/src/interpreter.rs#L505-L523>
    pub fn run(self) -> i32 {
        unsafe {
            // GIL must be acquired before calling Py_RunMain(). And Py_RunMain()
            // finalizes the interpreter. So we don't need to release the GIL
            // afterwards.
            pyffi::PyGILState_Ensure();
            pyffi::Py_RunMain()
        }
    }

    /// Proxy for [pyo3::Python::with_gil()].
    ///
    /// This function is just a wrapper around [pyo3::Python::with_gil()].
    /// But since the function holds a reference to self,
    /// it prevents MainPythonInterpreter from being dropped prematurely.
    ///
    /// This allows running Python code via the PyO3 Rust APIs. Alternatively,
    /// this can be used to run code when the Python GIL is held.
    #[inline]
    pub fn with_gil<F, R>(&self, f: F) -> R
    where
        F: for<'py> FnOnce(Python<'py>) -> R,
    {
        Python::with_gil(f)
    }
}

/// Finalize the python interpreter
//
// ref: <https://github.com/indygreg/PyOxidizer/blob/1ceca8664c71f39e849ce4873e00d821504b32bd/pyembed/src/interpreter.rs#L728-L752>
impl Drop for PythonInterpreter {
    fn drop(&mut self) {
        // Interpreter may have been finalized already. Possibly through our invocation
        // of Py_RunMain(). Possibly something out-of-band beyond our control. We don't
        // muck with the interpreter after finalization because this will likely result
        // in a segfault.
        if !Self::is_initialized() {
            return;
        }

        unsafe {
            pyffi::PyGILState_Ensure();
            pyffi::Py_FinalizeEx();
        }
    }
}

/// This is a re-export of crate [::dunce] to help you remove the UNC prefix `\\?\` for [PythonInterpreterEnv::Standalone].
///
/// Most Python ecosystems do not support Windows [Universal Naming Convention (UNC) paths] (e.g., `\\?\E:\xxx`).
/// However, some Tauri APIs may return UNC paths, such as [tauri::path::PathResolver::resource_dir].
///
/// We re-export [dunce::simplified] to help you remove the UNC prefix,
/// which is especially important when setting [PythonInterpreterEnv::Standalone];
/// because if you don't do this, it will cause:
///
/// 1. `sys.prefix` and `sys.exec_prefix` have UNC prefix
/// 2. further causing `stdlib` and `site-packages` in `sys.path` to have a UNC prefix
/// 3. further causing `__file__` and `module.__file__` to have a UNC prefix
/// 4. then it may cause [some Python packages to not work properly](https://github.com/pallets/jinja/issues/1675#issuecomment-1323555773).
///
/// [Universal Naming Convention (UNC) paths]: https://learn.microsoft.com/dotnet/standard/io/file-path-formats#unc-paths
///
/// # Example
///
/**
```rust
use pytauri::standalone::{dunce::simplified, PythonInterpreterEnv};
use tauri::{path::PathResolver, Result, Runtime};

fn py_env<R: Runtime>(path_resolver: &PathResolver<R>) -> Result<PythonInterpreterEnv<'_>> {
    let resource_dir = path_resolver.resource_dir()?;

    // 👉 Remove the UNC prefix `\\?\`, Python ecosystems don't like it.
    let resource_dir = simplified(&resource_dir).to_owned();
    Ok(PythonInterpreterEnv::Standalone(resource_dir.into()))
}
```
*/
pub mod dunce {
    pub use dunce::simplified;
}
