# pytauri-core

## [Unreleased]

### BREAKING

- [#163](https://github.com/pytauri/pytauri/pull/163) - refactor: `ext_mod::Url` -> `ext_mod::Url<'a'>` and removed the implementations of `Deref` and `DerefMut`.
- [#157](https://github.com/pytauri/pytauri/pull/157) - feat!: `Position.Physical(x, y)` -> `Position.Physical((x, y))`. See home `/CHANGELOG` for more details.

### Added

- [#163](https://github.com/pytauri/pytauri/pull/163) - feat(plugin): implement `tauri-plugin-dialog` bindings.
- [#160](https://github.com/pytauri/pytauri/pull/160) - feat!: integrate `plugin-notification` as a gated-feature of `pytauri`.

    Added: `pytauri_plugins::notification::NotificationBuilderArgs`.

- [#157](https://github.com/pytauri/pytauri/pull/157) - feat: fully implement `tauri::RunEvent` bindings.

### Internal

- [#155](https://github.com/pytauri/pytauri/pull/155) - refactor: modularize `ext_mod_impl::self`.

## [0.5.0]

### BREAKING

- [#141](https://github.com/pytauri/pytauri/pull/141) - feat!: `pytauri.path.PathResolver` now returns a `pathlib.Path` object instead of a `str`.

### Added

- [#136](https://github.com/pytauri/pytauri/pull/136) - feat(pytauri): accessing the request headers in `Commands`.
- [#124](https://github.com/pytauri/pytauri/pull/124) - feat: introduce `App::run_return`.

### Deprecated

- [#124](https://github.com/pytauri/pytauri/pull/124) - fix: deprecate `App::run_iteration`.

## [0.4.0]

### Added

- [#62](https://github.com/WSH032/pytauri/pull/62) - feat: add `path::PathResolver`, `Manager::path`.
- [#61](https://github.com/WSH032/pytauri/pull/61) - feat: add `Emitter`, `EventTarget`, `EventTargetType`, `ImplEmitter` for [Event System](https://tauri.app/develop/calling-frontend/#event-system).
- [#63](https://github.com/pytauri/pytauri/pull/63) - feat: add `Url` and `webview::WebviewWindow::navigate`.

## [0.3.0]

### BREAKING

- [#80](https://github.com/pytauri/pytauri/pull/80) - refactor: `trait PyAppHandleExt` is now sealed and no longer has generic parameters.
- [#79](https://github.com/pytauri/pytauri/pull/79) - pref: the fields of `enum RunEvent` `struct` variants become `Py<T>` types from rust types.

### Added

- [#83](https://github.com/pytauri/pytauri/pull/83) - feat: add `Context::set_assets` to allow using custom assets (e.g, loading from memory/disk).
- [#80](https://github.com/pytauri/pytauri/pull/80) - feat: add `PyAppHandleExt::get_or_init_py_app_handle`, and the methods return `&Py<AppHandle>` instead of `impl Deref<Target = Py<AppHandle>>` now.
- [#79](https://github.com/pytauri/pytauri/pull/79) - feat: implement [tauri `tray` feature](https://tauri.app/learn/system-tray/):
    enable `tauri/tray-icon` feature
    - `mod tauri::`
        - `Rect`
        - `Size`
        - `enum RunEvent::{MenuEvent, TrayIconEvent}`
        - `AppHandle::{run_on_main_thread, exit, restart, on_tray_icon_event, tray_by_id, remove_tray_by_id, default_window_icon, invoke_key}`
    - `mod tauri::tray`
    - `mod webview::`
        - `WebviewWindow::{run_on_main_thread, set_icon}`
- [#75](https://github.com/pytauri/pytauri/pull/75) - feat: implement [tauri `menu` feature](https://tauri.app/learn/window-menu/):
    - `mod tauri::`
        - `AppHandle::{on_menu_event, menu, set_menu, remove_menu, hide_menu, show_menu}`
        - `Position`
        - `PositionType`
    - `mod tauri::menu`
    - `mod tauri::image`
    - `mod tauri::window`
    - `mod tauri::webview`
        - `WebviewWindow::{on_menu_event, menu, set_menu, remove_menu, hide_menu, show_menu, is_menu_visible, popup_menu, popup_menu_at}`
        - `Webview::window`

### Changed

- [#86](https://github.com/pytauri/pytauri/pull/86) - pref: use `Cow<'_, [u8]>` instead of `Vec<u8>` as `pymehtods`/`pyfunction` and `extract` parameters to improve performance.
    see [PyO3/pyo3#3310](https://github.com/PyO3/pyo3/issues/3310#issuecomment-2674022839) and [PyO3/pyo3#2888](https://github.com/PyO3/pyo3/issues/2888) for more details.
- [#79](https://github.com/pytauri/pytauri/pull/79) - perf: almost all of pyo3 `pymethods` will release the `GIL` now.
- [#76](https://github.com/pytauri/pytauri/pull/76) - perf: use `pyo3::intern!` in `Invoke::bind_to` for commands `IPC` performance.
- [#75](https://github.com/pytauri/pytauri/pull/75) - perf: all methods of `WebviewWindow` will release the `GIL` now.
- [#75](https://github.com/pytauri/pytauri/pull/75) - perf: `App::{run, run_iteration}` will use a singleton `Py<AppHandle>` as an argument instead of fetching it from `tauri::State` each loop.

### Internal

- [#83](https://github.com/pytauri/pytauri/pull/83) - refactor: add trait `utils::PyResultExt` to handle unraisable `PyErr`.

## [0.2.0]

### BREAKING

- [#57](https://github.com/pytauri/pytauri/pull/57) - refactor: remove `RunEventEnum`, use matched `RunEvent` directly.
- [#56](https://github.com/pytauri/pytauri/pull/56) - perf: `Invoke::bind_to` now returns `[Self::BODY_KEY]`: `PyBytes` instead of `PyByteArray`.

### Added

- [#50](https://github.com/pytauri/pytauri/pull/50) - feat: add `ipc::Channel`, `ipc::JavaScriptChannelId`, `webview::Webview`, `webview::WebviewWindow::as_ref::<webview>` for [channels ipc](https://tauri.app/develop/calling-frontend/#channels).
- [#46](https://github.com/pytauri/pytauri/pull/46) - feat: add `webview::WebviewWindow`, `Manager`, `ImplManager`, `App::handle`.
- [#48](https://github.com/pytauri/pytauri/pull/48) - feat: accessing the `WebviewWindow` in `Commands`.
- [#49](https://github.com/pytauri/pytauri/pull/49) - feat: add `Event`, `EventId`, `Listener`, `ImplListener` for [Event System](https://tauri.app/develop/calling-frontend/#event-system).

## [0.1.0-beta.1]

## [0.1.0-beta.0]

[unreleased]: https://github.com/pytauri/pytauri/tree/HEAD
[0.5.0]: https://github.com/pytauri/pytauri/releases/tag/rs/pytauri-core/v0.5.0
[0.4.0]: https://github.com/pytauri/pytauri/releases/tag/rs/pytauri-core/v0.4.0
[0.3.0]: https://github.com/pytauri/pytauri/releases/tag/rs/pytauri-core/v0.3.0
[0.2.0]: https://github.com/pytauri/pytauri/releases/tag/rs/pytauri-core/v0.2.0
[0.1.0-beta.1]: https://github.com/pytauri/pytauri/releases/tag/rs/pytauri-core/v0.1.0-beta.1
[0.1.0-beta.0]: https://github.com/pytauri/pytauri/releases/tag/rs/pytauri-core/v0.1.0-beta.0
