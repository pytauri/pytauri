# pytauri

## [Unreleased]

## [0.8.0]

### Added

- [#265](https://github.com/pytauri/pytauri/pull/265) - feat(pytauri): add `WebviewWindowBuilder` bindings.

    Added `tauri-macos-private-api` feature.

- [#259](https://github.com/pytauri/pytauri/pull/259) - feat(pytauri): more `WebviewWindow` and `AppHandle` bindings.

    Added `tauri-devtools` feature.

## [0.7.0]

## [0.6.1]

- [#175](https://github.com/pytauri/pytauri/pull/175) - fix(pytauri): bump `tauri-plugin-*` to fix rust docs build failures on `docs.rs`.

## [0.6.0]

### BREAKING

- [#161](https://github.com/pytauri/pytauri/pull/161) - refactor(pytauri)!: refactor `BuilderArgs` to `TypedDict`.

### Added

- [#160](https://github.com/pytauri/pytauri/pull/160) - feat!: integrate `plugin-notification` as a gated-feature of `pytauri`.

### Fixed

- [#159](https://github.com/pytauri/pytauri/pull/159) - fix(standalone): explicitly pass `PyString` to `multiprocessing.set_executable`.

## [0.5.0]

### BREAKING

- [#133](https://github.com/pytauri/pytauri/pull/113) - fix(pytauri)!: make `BuilderArgs.invoke_handler` as required parameter for #110.

## [0.4.0]

## [0.3.0]

### Added

- [#80](https://github.com/pytauri/pytauri/pull/80) - feat: `BuilderArgs`:
    - add `BuilderArgs::setup` to support tauri app setup hook.
    - `BuilderArgs::context` now can be `Position and Keyword` arguments.

## [0.2.0]

### BREAKING

- [#52](https://github.com/pytauri/pytauri/pull/52) - refactor(standalone)!: new API for preparing python interpreter.
    The `pytauri::standalone` module has been completely rewritten.
    Previously, you used `prepare_freethreaded_python_with_executable` and `append_ext_mod`. Now, you need to use `PythonInterpreterBuilder`.
    See the `pytauri` crate rust API docs and tutorial (examples/tauri-app) `main.rs` code for more information on how to migrate.

### Added

- [#60](https://github.com/pytauri/pytauri/pull/60) - feat: re-export `dunce::simplified` to remove `resource_dir()` UNC path prefix `\\?\` for `PythonInterpreterEnv::Standalone`. Fix [pallets/jinja#1675](https://github.com/pallets/jinja/issues/1675#issuecomment-1323555773) for `nicegui-app` standalone example.
- [#51](https://github.com/pytauri/pytauri/pull/51) - feat: support `multiprocessing` for standalone app.
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

- [#54](https://github.com/pytauri/pytauri/pull/54) - feat: export the extension module to `sys.modules["__pytauri_ext_mod__"]` if on standalone mode.
- [#52](https://github.com/pytauri/pytauri/pull/52) - feat: set `sys._pytauri_standalone=True` when run on standalone app (i.e., launch from rust).
- [#51](https://github.com/pytauri/pytauri/pull/51) - refactor: use `Python::run` with `locals` as arguments to execute `_append_ext_mod.py` for better performance.

## [0.1.0-beta.0]

[unreleased]: https://github.com/pytauri/pytauri/tree/HEAD
[0.8.0]: https://github.com/pytauri/pytauri/releases/tag/rs/pytauri/v0.8.0
[0.7.0]: https://github.com/pytauri/pytauri/releases/tag/rs/pytauri/v0.7.0
[0.6.1]: https://github.com/pytauri/pytauri/releases/tag/rs/pytauri/v0.6.1
[0.6.0]: https://github.com/pytauri/pytauri/releases/tag/rs/pytauri/v0.6.0
[0.5.0]: https://github.com/pytauri/pytauri/releases/tag/rs/pytauri/v0.5.0
[0.4.0]: https://github.com/pytauri/pytauri/releases/tag/rs/pytauri/v0.4.0
[0.3.0]: https://github.com/pytauri/pytauri/releases/tag/rs/pytauri/v0.3.0
[0.2.0]: https://github.com/pytauri/pytauri/releases/tag/rs/pytauri/v0.2.0
[0.1.0-beta.0]: https://github.com/pytauri/pytauri/releases/tag/rs/pytauri/v0.1.0-beta.0
