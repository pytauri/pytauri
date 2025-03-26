# pytauri-plugin-notification

## [Unreleased]

## [0.4.0]

bump workspace version

## [0.3.0]

bump workspace version

## [0.2.0]

### BREAKING

- [#70](https://github.com/pytauri/pytauri/pull/70) - Removed `NotificationBuilderArgs`.
    Previously:

    ```python
    from pytauri_plugin_notification import NotificationBuilderArgs

    notification_builder.show(
        NotificationBuilderArgs(title="Greeting", body=f"Hello, {body.name}!")
    )
    ```

    Now:

    ```python
    notification_builder.show(title="Greeting", body=f"Hello, {body.name}!")
    ```

### Added

- [#70](https://github.com/pytauri/pytauri/pull/70) - feat: add more `NotificationBuilder` parameters
    - `id`
    - `channel_id`
    - `large_body`
    - `summary`
    - `action_type_id`
    - `group`
    - `group_summary`
    - `sound`
    - `inbox_line`
    - `icon`
    - `large_icon`
    - `icon_color`
    - `ongoing`
    - `auto_cancel`
    - `silent`

### Changed

- [#47](https://github.com/pytauri/pytauri/pull/47) - refactor: use `pytauri::ImplManager` as `self::ImplNotificationExt`

## [0.1.0-beta.0]

[unreleased]: https://github.com/pytauri/pytauri/tree/HEAD
[0.4.0]: https://github.com/pytauri/pytauri/releases/tag/py/pytauri-plugin-notification/v0.4.0
[0.3.0]: https://github.com/pytauri/pytauri/releases/tag/py/pytauri-plugin-notification/v0.3.0
[0.2.0]: https://github.com/pytauri/pytauri/releases/tag/py/pytauri-plugin-notification/v0.2.0
[0.1.0-beta.0]: https://github.com/pytauri/pytauri/releases/tag/py/pytauri-plugin-notification/v0.1.0-beta.0
