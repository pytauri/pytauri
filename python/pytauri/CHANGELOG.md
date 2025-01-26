# pytauri

## [Unreleased]

### BREAKING

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
[0.1.0-beta.0]: https://github.com/WSH032/pytauri/releases/tag/py/pytauri/v0.1.0-beta.0
