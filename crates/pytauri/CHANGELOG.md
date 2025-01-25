# pytauri

## [Unreleased]

### BREAKING

- [#52](https://github.com/WSH032/pytauri/pull/52) - refactor(standalone)!: new API for preparing python interpreter.
    The `pytauri::standalone` module has been completely rewritten.
    Previously, you used `prepare_freethreaded_python_with_executable` and `append_ext_mod`. Now, you need to use `PythonInterpreterBuilder`.
    See the `pytauri` crate rust API docs and tutorial (examples/tauri-app) `main.rs` code for more information on how to migrate.

### Added

- [#51](https://github.com/WSH032/pytauri/pull/51) - feat: support `multiprocessing` for standalone app.
    - For standalone app:
        - set `sys.executable` to the actual python interpreter executable path.
        - set `sys.argv` to `std::env::args_os()`.
        - set `sys.frozen` to `True`.
        - call `multiprocessing.set_start_method` with
            - windows: `spawn`
            - unix: `fork`
        - call `multiprocessing.set_executable` with `std::env::current_exe()`.
    - Add `fn is_forking` for checking if the app is spawned by `multiprocessing`.

### Internal

- [#54](https://github.com/WSH032/pytauri/pull/54) - feat: export the extension module to `sys.modules["__pytauri_ext_mod__"]` if on standalone mode.
- [#52](https://github.com/WSH032/pytauri/pull/52) - feat: set `sys._pytauri_standalone=True` when run on standalone app (i.e., launch from rust).
- [#51](https://github.com/WSH032/pytauri/pull/51) - refactor: use `Python::run` with `locals` as arguments to execute `_append_ext_mod.py` for better performance.

## [0.1.0-beta.0]

[unreleased]: https://github.com/WSH032/pytauri/tree/HEAD
[0.1.0-beta.0]: https://github.com/WSH032/pytauri/releases/tag/rs/pytauri/v0.1.0-beta.0
