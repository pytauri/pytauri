# pytauri-plugin-notification

## [Unreleased]

### BREAKING

- [#70](https://github.com/WSH032/pytauri/pull/70) - Removed `NotificationBuilderArgs`

### Added

- [#70](https://github.com/WSH032/pytauri/pull/70) - feat: add more `NotificationBuilder` parameters
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

- [#47](https://github.com/WSH032/pytauri/pull/47) - refactor: use `pytauri::ImplManager` as `self::ImplNotificationExt`

## [0.1.0-beta.0]

[unreleased]: https://github.com/WSH032/pytauri/tree/HEAD
[0.1.0-beta.0]: https://github.com/WSH032/pytauri/releases/tag/rs/pytauri-plugin-notification/v0.1.0-beta.0
