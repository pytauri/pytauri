<!-- The content will be also use in `docs/CHANGELOG/index.md` by `pymdownx.snippets` -->
<!-- Do not use any **relative link** and  **GitHub-specific syntax** ！-->
<!-- Do not rename or move the file -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

- `Highlights` for the most attractive new features.
- `BREAKING` for breaking changes.
- `Added` for new features.
- `Changed` for changes in existing functionality.
- `Deprecated` for soon-to-be removed features.
- `Removed` for now removed features.
- `Fixed` for any bug fixes.
- `Security` in case of vulnerabilities.
- `Docs` for documentation changes.
- `YANKED` for deprecated releases.
- `Internal` for internal changes. Only for maintainers.

!!! tip
    This homepage is used to provide a blog-like changelog and `BREAKING CHANGE` migration guide.

    You can expand sub-projects to view detailed changelogs.

<!-- Refer to: https://github.com/olivierlacan/keep-a-changelog/blob/main/CHANGELOG.md -->
<!-- Refer to: https://github.com/gradio-app/gradio/blob/main/CHANGELOG.md -->
<!-- Refer to: https://github.com/WSH032/fastapi-proxy-lib/blob/main/CHANGELOG.md -->

## [Unreleased]

### BREAKING

- [#157](https://github.com/pytauri/pytauri/pull/157) - feat(pytauri)!: `Position.Physical(x, y)` -> `Position.Physical((x, y))`.

    These APIs have changed:

    - `Position.Physical`
    - `Position.Logical`
    - `Size.Physical`
    - `Size.Logical`

    ??? tip "Migration"

        ```diff
        from pytauri import Position, PositionType, Size, SizeType

        def foo(pos: PositionType, size: SizeType) -> None:
            match pos:
        -        case Position.Physical(x, y):
        +        case Position.Physical((x, y)):
                    print(f"Physical position: {x}, {y}")
        -        case Position.Logical(x, y):
        +        case Position.Logical((x, y)):
                    print(f"Logical position: {x}, {y}")
            match size:
        -        case Size.Physical(w, h):
        +        case Size.Physical((w, h)):
                    print(f"Physical size: {w}, {h}")
        -        case Size.Logical(w, h):
        +        case Size.Logical((w, h)):
                    print(f"Logical size: {w}, {h}")

        -foo(pos=Position.Physical(1, 2), size=Size.Physical(3, 4))
        +foo(pos=Position.Physical((1, 2)), size=Size.Physical((3, 4)))
        ```

### Added

- [#157](https://github.com/pytauri/pytauri/pull/157) - feat(pytauri): fully implement `tauri::RunEvent` bindings.
    - `mod tauri::`
        - `Theme`
        - `CloseRequestApi`
        - `ExitRequestApi`
        - `DragDropEvent` (`DragDropEventType`)
        - `WebviewEvent` (`WebviewEventType`)
        - `WindowEvent` (`WindowEventType`)
        - `RunEvent::{ExitRequested::api, WebviewEvent::event, WindowEvent::event}` fields
        - `webview::WebviewWindow::{on_webview_event, on_window_event}` methods
        - `AppHandle::set_theme` and `webview::WebviewWindow::{set_theme, theme}` methods

    - add `_NonExhaustive` field to all `#[non_exhaustive]` `enum`s

    ??? tip "Usage"

        ```py
        from pytauri import AppHandle, Manager, WindowEvent, WindowEventType


        def register_window_event_handler(app_handle: AppHandle):
            webview_window = Manager.get_webview_window(app_handle, "main")
            assert webview_window is not None

            close_requested = False

            def window_event_handler(event: WindowEventType) -> None:
                nonlocal close_requested
                match event:
                    case WindowEvent.CloseRequested(api=api):
                        if not close_requested:
                            print("Preventing window closing")
                            api.prevent_close()
                            close_requested = True
                    case WindowEvent.Focused(focused):
                        print(f"Window focused: {focused}")
                    case WindowEvent.Moved((x, y)):
                        print(f"Moved to ({x}, {y})")
                    case _:
                        pass

            webview_window.on_window_event(window_event_handler)
        ```

## [0.5.0]

### Highlights

#### Wheel on Windows Arm64

> - [#139](https://github.com/pytauri/pytauri/pull/139) - feat: add support for the `windows-11-arm` platform wheel.

You can now install `pytauri-wheel` from PyPI on Windows 11 Arm64.

#### Using Unreleased Commits

> - [#147](https://github.com/pytauri/pytauri/pull/147) - feat: support installation from git repository.

See: <https://pytauri.github.io/pytauri/0.5/usage/using-unreleased-commits/>

#### Accessing the request headers

> - [#136](https://github.com/pytauri/pytauri/pull/136) - feat(pytauri): accessing the request headers in `Commands`.

See: <https://pytauri.github.io/pytauri/0.5/usage/concepts/ipc/#accessing-request-headers>

#### Protect source code

> - [#134](https://github.com/pytauri/pytauri/pull/134) - docs: add tutorial on using `Cython` to protect source code.

![cythonized](https://github.com/user-attachments/assets/d4ecc6ae-b27f-4b8a-ad3e-28d406cd6ead)

See: <https://pytauri.github.io/pytauri/0.5/usage/tutorial/build-standalone-cython/>

### BREAKING

- [#136](https://github.com/pytauri/pytauri/pull/136) - `tauri v2.5` requires upgrading `@tauri-apps/api: ^2.5` and `tauri-plugin-pytauri-api: ^0.5`.
- [#141](https://github.com/pytauri/pytauri/pull/141) - feat(pytauri)!: `pytauri.path.PathResolver` now returns a `pathlib.Path` object instead of a `str`.
- [#133](https://github.com/pytauri/pytauri/pull/133) - fix(pytauri)!: make `BuilderArgs.invoke_handler` as required parameter for #110.

    If you do not specify `invoke_handler`,
    `pytauri` will not register the `tauri-plugin-pytauri` plugin,
    which means you cannot use `pyInvoke` in the frontend to call `Commands`
    (you will receive an error like ["plugin pytauri not found"]).
    If this is indeed the behavior you expect, explicitly pass `None`.

    ["plugin pytauri not found"]: https://github.com/pytauri/pytauri/issues/110

### Added

- [#141](https://github.com/pytauri/pytauri/pull/141) - chore: bump `pyo3` to `0.24.1`.
- [#124](https://github.com/pytauri/pytauri/pull/124) - [#136](https://github.com/pytauri/pytauri/pull/136) - chore: bump `tauri` dependencies:
    - `tauri` to `2.5.1`
    - `tauri-utils` to `~2.4`
    - `tauri-build` to `2.2`
    - `tauri-plugin` to `2.2`
    - `tauri-plugin-notification` to `2.2`
    - `tauri-plugin-opener` to `2.2`
    - `@tauri-apps/api` to `^2.5`
    - `@tauri-apps/plugin-opener` to `^2.2`
    - `@tauri-apps/plugin-dialog` to `^2.2`

### Docs

- [#124](https://github.com/pytauri/pytauri/pull/124) - docs: update example to use `App.run_return` instead of `App.run`, allowing cleanup tasks to be performed after the app exits (e.g., shutting down the `nicegui` server) and retrieving the exit code.

## [0.4.0]

### Highlights

#### Precompiled python wheel (goodbye Rust compiler)

> - [#117](https://github.com/pytauri/pytauri/pull/117) - docs: add usage docs for `pytauri-wheel`
> - [#108](https://github.com/pytauri/pytauri/pull/108) - feat: initial precompiled python wheel support

In `v0.4.0`, we have introduced an exciting new feature: precompiled Python wheel support! 🎉

This means you can use PyTauri without writing any Rust code or needing a Rust compiler.

This allows you to perform full-stack development in pure Python (like [`pywebview`](https://github.com/r0x0r/pywebview) but battery-included 🤓).

Please refer to the [PyTauri Wheel documentation](https://pytauri.github.io/pytauri/0.4/usage/pytauri-wheel/) for more information.

#### New logo for PyTauri

Thanks to [@ISOR3X](https://github.com/ISOR3X) in [#111](https://github.com/pytauri/pytauri/pull/111)! PyTauri now has its own logo 🎉:

![pytauri-banner](https://github.com/pytauri/pytauri/raw/06d468a851ad21268df458580fab327e4ab5a941/docs/assets/banner.png)

> It is indeed a snake, but not eating a bean, or perhaps it was not intended to be. It is more so a combination of the Tauri logo (two dots with rings around them) and the Python logo (the snake, specifically the head). The left part is more intended to visualize the snake curling around. Perhaps it is a bit too abstract.

### Changed

- [#63](https://github.com/pytauri/pytauri/pull/63) - chore: bump `tauri` to `v2.3`.

### Internal

- [#119](https://github.com/pytauri/pytauri/pull/119) - ci(rs/release): add `--no-verify` to `cargo publish` so that we can release parallelly.
- [#113](https://github.com/pytauri/pytauri/pull/113) - ci: add `macos-latest` os in `lint-test` CI.
- [#111](https://github.com/pytauri/pytauri/pull/111) - docs: added PyTauri logo and updated documentation colors
- [#103](https://github.com/pytauri/pytauri/pull/103) - chore: transfer repo to `pytauri` org.

## [0.3.0]

### Highlights

- [menu](https://tauri.app/learn/window-menu/) and [tray](https://tauri.app/learn/system-tray/) python API bindings, see `pytauri` changelog for more details.

    | tray | menu |
    |:----:|:----:|
    | ![tray](https://github.com/user-attachments/assets/28d69270-4c76-4b48-854f-764fd5996a27) | ![menu](https://github.com/user-attachments/assets/4b8fd7e0-8c65-4566-bb77-8cb1bcc4f014) |

### Changed

- [#79](https://github.com/pytauri/pytauri/pull/79) - bump `rust-version = "1.82"`

### Docs

- [#88](https://github.com/pytauri/pytauri/pull/88) - docs: add rust api reference section.
- [#85](https://github.com/pytauri/pytauri/pull/85) - docs: add concepts `IPC` and `using multiprocessing` sections.
- [#80](https://github.com/pytauri/pytauri/pull/80) - `example/nicegui-app`:
    - Use `BuilderArgs.setup` for initialization instead of listening to the `RunEvent.Ready` event.
    - Rewrite the `FrontServer` `startup`/`shutdown` event hook logic.
    - Modularize the code.
- [#79](https://github.com/pytauri/pytauri/pull/79) - `example/nicegui-app`:
    - use `tray` and `menu` feature
    - use `python3.10` `match` statement instead of `if-else` statement
    - bump `requires-python = ">=3.10"`

### Internal

- [#81](https://github.com/pytauri/pytauri/pull/81) - ci: add `clear-cache.yml` workflow.

## [0.2.0]

### BREAKING

- [#70](https://github.com/pytauri/pytauri/pull/70) - feat(notification): removed `NotificationBuilderArgs`.
    See `CHANGELOG.md` of `py/pytauri-plugin-notification` for how to migrate.
- [#57](https://github.com/pytauri/pytauri/pull/57) - refactor(py/pytauri): remove `RunEventEnum`, use matched `RunEvent` directly.
    See `CHANGELOG.md` of `py/pytauri` for how to migrate.
- [#56](https://github.com/pytauri/pytauri/pull/56) - perf(pytauri): all IPC methods that previously accepted `bytearray` as a parameter now only accept `bytes` as a parameter.
- [#52](https://github.com/pytauri/pytauri/pull/52) - refactor(standalone)!: new API for preparing python interpreter.
    The `pytauri::standalone` module has been completely rewritten.
    Previously, you used `prepare_freethreaded_python_with_executable` and `append_ext_mod`. Now, you need to use `PythonInterpreterBuilder`.
    See the `pytauri` crate rust API docs and tutorial (examples/tauri-app) `main.rs` code for more information on how to migrate.

### Docs

- [#60](https://github.com/pytauri/pytauri/pull/60) - update `examples` `main.rs` to remove `resource_dir()` UNC path prefix `\\?\` for `PythonInterpreterEnv::Standalone`. Fix [pallets/jinja#1675](https://github.com/pallets/jinja/issues/1675#issuecomment-1323555773) for `nicegui-app` standalone example.
- [#55](https://github.com/pytauri/pytauri/pull/55) - Add `integrate with nicegui` example `nicegui-app`. See `examples/nicegui-app`.
- [#52](https://github.com/pytauri/pytauri/pull/52) - update `examples/tauri-app` `main.rs` for new API to prepare python interpreter.
- [#52](https://github.com/pytauri/pytauri/pull/52) - add the usage of `multiprocessing.freeze_support` in `examples/tauri-app` `__main__.py`.

### Changed

- [#46](https://github.com/pytauri/pytauri/pull/46) - bump `tauri` to `v2.2`

### Internal

- [#83](https://github.com/pytauri/pytauri/pull/83) - chore: bump `pyo3` to `v0.23.4` in `Cargo.lock` to fix PyO3/pyo3#4828.
- [#64](https://github.com/pytauri/pytauri/pull/64) - test: add integration tests for `command` and `channel` ipc

## [0.1.0-beta]

[unreleased]: https://github.com/pytauri/pytauri/tree/HEAD
[0.5.0]: https://github.com/pytauri/pytauri/releases/tag/v0.5.0
[0.4.0]: https://github.com/pytauri/pytauri/releases/tag/v0.4.0
[0.3.0]: https://github.com/pytauri/pytauri/releases/tag/v0.3.0
[0.2.0]: https://github.com/pytauri/pytauri/releases/tag/v0.2.0
[0.1.0-beta]: https://github.com/pytauri/pytauri/releases/tag/v0.1.0-beta
