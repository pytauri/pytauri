# pytauri-wheel

## [Unreleased]

### BREAKING

- [#220](https://github.com/pytauri/pytauri/pull/220) - feat: support registering plugin from python.

    See: <https://pytauri.github.io/pytauri/0.8/usage/pytauri-wheel/#pytauri-plugins>.

    The parameters `pytauri_wheel.builder_factory(opener, clipboard_manager, dialog, fs, notification)` have been removed. Please use `pytauri.BuilderArgs.plugins` or `pytauri.Apphandle.plugin` to manually register plugins.

    !!! tip "Migration"
        === "current"
            ```python
            from pytauri_plugins import clipboard_manager, dialog, fs, notification, opener
            from pytauri_wheel.lib import builder_factory

            builder = builder_factory()
            app = builder.build(
                context=...,
                invoke_handler=...,
                plugins=[
                    opener.init(),
                    clipboard_manager.init(),
                    dialog.init(),
                    fs.init(),
                    notification.init(),
                ],
            )
            ```
        === "previous"
            ```python
            from pytauri_wheel.lib import builder_factory

            builder = builder_factory(
                opener=True, clipboard_manager=True, dialog=True, fs=True, notification=True
            )
            app = builder.build(
                context=...,
                invoke_handler=...,
            )
            ```

### Added

- [#220](https://github.com/pytauri/pytauri/pull/220) - feat: support registering plugin from python.

    enabled following plugin features:

    ```toml
    [
        "plugin-notification",
        "plugin-dialog",
        "plugin-clipboard-manager",
        "plugin-fs",
        "plugin-opener",
        "plugin-autostart",
        "plugin-deep-link",
        "plugin-http",
        "plugin-os",
        "plugin-persisted-scope",
        "plugin-positioner",
        "plugin-process",
        "plugin-shell",
        "plugin-single-instance",
        "plugin-updater",
        "plugin-upload",
        "plugin-websocket",
        "plugin-window-state",
        "plugin-global-shortcut",
    ]
    ```

## [0.7.0]

### Fixed

- [#175](https://github.com/pytauri/pytauri/pull/175) - fix: bump `tauri-plugin-*` to fix rust docs build failures on `docs.rs`.

    See [tauri-apps/tauri#13597](https://github.com/tauri-apps/tauri/pull/13597#issuecomment-2961321899) for details.

    - `tauri-plugin-opener = { version = "2.3.0" }`
    - `tauri-plugin-clipboard-manager = { version = "2.2.3" }`
    - `tauri-plugin-dialog = { version = "2.2.2" }`
    - `tauri-plugin-fs = { version = "2.3.0" }`
    - `tauri-plugin-notification = { version = "2.2.3" }`

## [0.6.0]

### Added

- [#165](https://github.com/pytauri/pytauri/pull/165) - feat: enable `tauri-plugin-notification` and integrate `pytauri_plugins`.

## [0.5.0]

### Highlights

- [#139](https://github.com/pytauri/pytauri/pull/139) - feat: add support for the `windows-11-arm` platform wheel.

## [0.4.0]

the first release

[unreleased]: https://github.com/pytauri/pytauri/tree/HEAD
[0.7.0]: https://github.com/pytauri/pytauri/releases/tag/py/pytauri-wheel/v0.7.0
[0.6.0]: https://github.com/pytauri/pytauri/releases/tag/py/pytauri-wheel/v0.6.0
[0.5.0]: https://github.com/pytauri/pytauri/releases/tag/py/pytauri-wheel/v0.5.0
[0.4.0]: https://github.com/pytauri/pytauri/releases/tag/py/pytauri-wheel/v0.4.0
