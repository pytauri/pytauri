# pytauri

## [Unreleased]

## [0.2.0]

### Added

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
- [#75](https://github.com/WSH032/pytauri/pull/75) - feat: add `pillow >= 11.1` as dependency.

### Changed

- [#75](https://github.com/WSH032/pytauri/pull/75) - perf: all methods of `WebviewWindow` will release the `GIL` now.
- [#75](https://github.com/WSH032/pytauri/pull/75) - perf: `App::{run, run_iteration}` will use a singleton `Py<AppHandle>` as an argument instead of fetching it from `tauri::State` each loop.

### BREAKING

- [#57](https://github.com/WSH032/pytauri/pull/57) - refactor: remove `RunEventEnum`, use matched `RunEvent` directly.
    Previously:

    ```python
    def callback(app_handle: AppHandle, run_event: RunEvent) -> None:
        run_event_enum: RunEventEnumType = run_event.match_ref()
        match run_event_enum:
            case RunEventEnum.Ready: ...

    app.run(callback)
    ```

    Now:

    ```python
    def callback(app_handle: AppHandle, run_event: RunEventType) -> None:
        match run_event:
            case RunEvent.Ready: ...

    app.run(callback)
    ```

- [#56](https://github.com/WSH032/pytauri/pull/56) - perf: all IPC methods that previously accepted `bytearray` as a parameter now only accept `bytes` as a parameter.

### Added

- [#50](https://github.com/WSH032/pytauri/pull/50) - feat: add `ipc::Channel`, `ipc::JavaScriptChannelId`, `webview::Webview`, `webview::WebviewWindow::as_ref::<webview>` for [channels ipc](https://tauri.app/develop/calling-frontend/#channels).
- [#46](https://github.com/WSH032/pytauri/pull/46) - feat: add `webview::WebviewWindow`, `Manager`, `ImplManager`, `App::handle`.
- [#48](https://github.com/WSH032/pytauri/pull/48) - feat: accessing the `WebviewWindow` in `Commands`.
- [#49](https://github.com/WSH032/pytauri/pull/49) - feat: add `Event`, `EventId`, `Listener`, `ImplListener` for [Event System](https://tauri.app/develop/calling-frontend/#event-system).

### Internal

- [#54](https://github.com/WSH032/pytauri/pull/54)
    - feat: import the extension module from `sys.modules["__pytauri_ext_mod__"]` if on standalone mode (`sys._pytauri_standalone`).
    - feat: support specifying `entry_point` package name which be used to import the extension module via `os.environ["_PYTAURI_DIST"]` (only for non-standalone mode).

## [0.1.0-beta.0]

[unreleased]: https://github.com/WSH032/pytauri/tree/HEAD
[0.2.0]: https://github.com/WSH032/pytauri/releases/tag/py/pytauri/v0.2.0
[0.1.0-beta.0]: https://github.com/WSH032/pytauri/releases/tag/py/pytauri/v0.1.0-beta.0
