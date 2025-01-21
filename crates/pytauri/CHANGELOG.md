# pytauri

## [Unreleased]

### BREAKING

- [#51](https://github.com/WSH032/pytauri/pull/51) - When the app executable is spawned by python `multiprocessing` to execute subprocess tasks, `fn append_ext_mod` will return a `PySystemExit` exception at the end of execution. At this point, you should exit the Rust code instead of continuing to run the Python app. Please refer to its documentation for more information.

### Added

- [#51](https://github.com/WSH032/pytauri/pull/51) - feat: support `multiprocessing` for standalone app.
    - For standalone app:
        - set `sys.argv` to `std::env::args_os()`
        - set `sys.frozen` to `True`
        - call `multiprocessing.set_start_method` with
            - windows: `spawn`
            - unix: `fork`
        - call `multiprocessing.set_executable` with `std::env::current_exe()`
        - call `multiprocessing.freeze_support()` at the end of `append_ext_mod`
    - Add `fn is_forking` for checking if the app is spawned by `multiprocessing`.

### Internal

- [#51](https://github.com/WSH032/pytauri/pull/51) - refactor: use `Python::run` with `locals` as arguments to execute `_append_ext_mod.py` for better performance.

## [0.1.0-beta.0]

[unreleased]: https://github.com/WSH032/pytauri/tree/HEAD
[0.1.0-beta.0]: https://github.com/WSH032/pytauri/releases/tag/rs/pytauri/v0.1.0-beta.0
