# pytauri-core

## [Unreleased]

### BREAKING

- [#79](https://github.com/WSH032/pytauri/pull/79) - pref: the fields of `enum RunEvent` `struct` variants become `Py<T>` types from rust types.

### Added

- [#79](https://github.com/WSH032/pytauri/pull/79) - feat: implement [tauri `tray` feature](https://tauri.app/learn/system-tray/):
    enable `tauri/tray-icon` feature
    - `mod tauri::`
        - `Rect`
        - `Size`
        - `enum RunEvent::{MenuEvent, TrayIconEvent}`
        - `AppHandle::{run_on_main_thread, exit, restart, on_tray_icon_event, tray_by_id, remove_tray_by_id, default_window_icon, invoke_key}`
    - `mod tauri::tray`
    - `mod webview::`
        - `WebviewWindow::{run_on_main_thread, set_icon}`
- [#75](https://github.com/WSH032/pytauri/pull/75) - feat: implement [tauri `menu` feature](https://tauri.app/learn/window-menu/):
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

- [#79](https://github.com/WSH032/pytauri/pull/79) - perf: almost all of pyo3 `pymethods` will release the `GIL` now.
- [#76](https://github.com/WSH032/pytauri/pull/76) - perf: use `pyo3::intern!` in `Invoke::bind_to` for commands `IPC` performance.
- [#75](https://github.com/WSH032/pytauri/pull/75) - perf: all methods of `WebviewWindow` will release the `GIL` now.
- [#75](https://github.com/WSH032/pytauri/pull/75) - perf: `App::{run, run_iteration}` will use a singleton `Py<AppHandle>` as an argument instead of fetching it from `tauri::State` each loop.

## [0.2.0]

### BREAKING

- [#57](https://github.com/WSH032/pytauri/pull/57) - refactor: remove `RunEventEnum`, use matched `RunEvent` directly.
- [#56](https://github.com/WSH032/pytauri/pull/56) - perf: `Invoke::bind_to` now returns `[Self::BODY_KEY]`: `PyBytes` instead of `PyByteArray`.

### Added

- [#50](https://github.com/WSH032/pytauri/pull/50) - feat: add `ipc::Channel`, `ipc::JavaScriptChannelId`, `webview::Webview`, `webview::WebviewWindow::as_ref::<webview>` for [channels ipc](https://tauri.app/develop/calling-frontend/#channels).
- [#46](https://github.com/WSH032/pytauri/pull/46) - feat: add `webview::WebviewWindow`, `Manager`, `ImplManager`, `App::handle`.
- [#48](https://github.com/WSH032/pytauri/pull/48) - feat: accessing the `WebviewWindow` in `Commands`.
- [#49](https://github.com/WSH032/pytauri/pull/49) - feat: add `Event`, `EventId`, `Listener`, `ImplListener` for [Event System](https://tauri.app/develop/calling-frontend/#event-system).

## [0.1.0-beta.1]

## [0.1.0-beta.0]

[unreleased]: https://github.com/WSH032/pytauri/tree/HEAD
[0.2.0]: https://github.com/WSH032/pytauri/releases/tag/rs/pytauri-core/v0.2.0
[0.1.0-beta.1]: https://github.com/WSH032/pytauri/releases/tag/rs/pytauri-core/v0.1.0-beta.1
[0.1.0-beta.0]: https://github.com/WSH032/pytauri/releases/tag/rs/pytauri-core/v0.1.0-beta.0
