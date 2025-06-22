# pytauri

## [Unreleased]

### Added

- [#178](https://github.com/pytauri/pytauri/pull/178) - feat(plugin-api)!: remove `rawPyInvoke` and `Channel`.

    [Channel.send][pytauri.ipc.Channel.send], [Invoke.resolve][pytauri.ipc.Invoke.resolve] and [InvokeResolver.resolve][pytauri.ipc.InvokeResolver.resolve] now can send `Union[bytes, str]`:

    - If `str`, it will be deserialized as JSON on the frontend.
    - If `bytes`, it will be sent as `ArrayBuffer` to the frontend.

## [0.6.0]

### BREAKING

- [#161](https://github.com/pytauri/pytauri/pull/161) - refactor(pytauri)!: refactor `BuilderArgs` to `TypedDict`.
- [#157](https://github.com/pytauri/pytauri/pull/157) - feat!: `Position.Physical(x, y)` -> `Position.Physical((x, y))`. See home `/CHANGELOG` for more details.

### Added

- [#163](https://github.com/pytauri/pytauri/pull/163) - feat(plugin): implement `tauri-plugin-dialog` bindings.
- [#160](https://github.com/pytauri/pytauri/pull/160) - feat!: integrate `plugin-notification` as a gated-feature of `pytauri`.

    Added: `pytauri_plugins::notification::NotificationBuilderArgs`.

- [#157](https://github.com/pytauri/pytauri/pull/157) - feat: fully implement `tauri::RunEvent` bindings.

### Fixed

- [#157](https://github.com/pytauri/pytauri/pull/157) - fix(typing): namedtuple member name can't start with underscore.

    > ref: <https://github.com/python/typing/pull/1979#issuecomment-2889160095>

    bump `pyright` to `^1.1.400`

## [0.5.0]

### BREAKING

- [#141](https://github.com/pytauri/pytauri/pull/141) - feat!: `pytauri.path.PathResolver` now returns a `pathlib.Path` object instead of a `str`.
- [#133](https://github.com/pytauri/pytauri/pull/113) - fix(pytauri)!: make `BuilderArgs.invoke_handler` as required parameter for #110.

### Added

- [#136](https://github.com/pytauri/pytauri/pull/136) - feat(pytauri): accessing the request headers in `Commands`:

    Added `ipc.Headers`, `ipc.ParametersType.headers` and `ipc.ArgumentsType.headers`.

- [#124](https://github.com/pytauri/pytauri/pull/124) - feat: introduce `App::run_return`:

    Unlike `App::run`, which terminates the entire process, `App::run_return` allows you to perform cleanup tasks after the app exits.
    For example, you can use `sys.exit(app.run_return())` to gracefully finalize the Python interpreter with an exit code.

### Deprecated

- [#124](https://github.com/pytauri/pytauri/pull/124) - fix: deprecate `App::run_iteration`.

## [0.4.0]

### Added

- [#62](https://github.com/WSH032/pytauri/pull/62) - feat: add `path::PathResolver`, `Manager::path`.
- [#61](https://github.com/WSH032/pytauri/pull/61) - feat: add `Emitter`, `EventTarget`, `EventTargetType`, `ImplEmitter` for [Event System](https://tauri.app/develop/calling-frontend/#event-system).
- [#63](https://github.com/pytauri/pytauri/pull/63) - feat: add `Url` and `webview::WebviewWindow::navigate`.

## [0.3.0]

### Added

- [#83](https://github.com/pytauri/pytauri/pull/83) - feat: add `def Context.set_assets` and `class Assets` to allow using custom assets (e.g, loading from memory/disk).
- [#80](https://github.com/pytauri/pytauri/pull/80) - feat: `BuilderArgs`:
    - add `BuilderArgs::setup` to support tauri app setup hook.
    - `BuilderArgs::context` now can be `Position and Keyword` arguments.
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
- [#75](https://github.com/pytauri/pytauri/pull/75) - feat: add `pillow >= 11.1` as dependency.

### Changed

- [#76](https://github.com/pytauri/pytauri/pull/76) - perf: use `pyo3::intern!` in `Invoke::bind_to` for commands `IPC` performance.
- [#75](https://github.com/pytauri/pytauri/pull/75) - perf: all methods of `WebviewWindow` will release the `GIL` now.
- [#75](https://github.com/pytauri/pytauri/pull/75) - perf: `App::{run, run_iteration}` will use a singleton `Py<AppHandle>` as an argument instead of fetching it from `tauri::State` each loop.

### Internal

- [#79](https://github.com/pytauri/pytauri/pull/79) - `ffi.ipc.JavaScriptChannelId.from_str` becomes `staticmethod` from `classmethod`.

## [0.2.0]

### BREAKING

- [#57](https://github.com/pytauri/pytauri/pull/57) - refactor: remove `RunEventEnum`, use matched `RunEvent` directly.
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

- [#56](https://github.com/pytauri/pytauri/pull/56) - perf: all IPC methods that previously accepted `bytearray` as a parameter now only accept `bytes` as a parameter.

### Added

- [#50](https://github.com/pytauri/pytauri/pull/50) - feat: add `ipc::Channel`, `ipc::JavaScriptChannelId`, `webview::Webview`, `webview::WebviewWindow::as_ref::<webview>` for [channels ipc](https://tauri.app/develop/calling-frontend/#channels).
- [#46](https://github.com/pytauri/pytauri/pull/46) - feat: add `webview::WebviewWindow`, `Manager`, `ImplManager`, `App::handle`.
- [#48](https://github.com/pytauri/pytauri/pull/48) - feat: accessing the `WebviewWindow` in `Commands`.
- [#49](https://github.com/pytauri/pytauri/pull/49) - feat: add `Event`, `EventId`, `Listener`, `ImplListener` for [Event System](https://tauri.app/develop/calling-frontend/#event-system).

### Internal

- [#54](https://github.com/pytauri/pytauri/pull/54)
    - feat: import the extension module from `sys.modules["__pytauri_ext_mod__"]` if on standalone mode (`sys._pytauri_standalone`).
    - feat: support specifying `entry_point` package name which be used to import the extension module via `os.environ["_PYTAURI_DIST"]` (only for non-standalone mode).

## [0.1.0-beta.0]

[unreleased]: https://github.com/pytauri/pytauri/tree/HEAD
[0.6.0]: https://github.com/pytauri/pytauri/releases/tag/py/pytauri/v0.6.0
[0.5.0]: https://github.com/pytauri/pytauri/releases/tag/py/pytauri/v0.5.0
[0.4.0]: https://github.com/pytauri/pytauri/releases/tag/py/pytauri/v0.4.0
[0.3.0]: https://github.com/pytauri/pytauri/releases/tag/py/pytauri/v0.3.0
[0.2.0]: https://github.com/pytauri/pytauri/releases/tag/py/pytauri/v0.2.0
[0.1.0-beta.0]: https://github.com/pytauri/pytauri/releases/tag/py/pytauri/v0.1.0-beta.0
