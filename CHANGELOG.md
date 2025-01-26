<!-- The content will be also use in `docs/CHANGELOG/index.md` by `pymdownx.snippets` -->
<!-- Do not use any **relative link** and  **GitHub-specific syntax** ！-->
<!-- Do not rename or move the file -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

- [#56](https://github.com/WSH032/pytauri/pull/56) - perf(pytauri): all IPC methods that previously accepted `bytearray` as a parameter now only accept `bytes` as a parameter.
- [#52](https://github.com/WSH032/pytauri/pull/52) - refactor(standalone)!: new API for preparing python interpreter.
    The `pytauri::standalone` module has been completely rewritten.
    Previously, you used `prepare_freethreaded_python_with_executable` and `append_ext_mod`. Now, you need to use `PythonInterpreterBuilder`.
    See the `pytauri` crate rust API docs and tutorial (examples/tauri-app) `main.rs` code for more information on how to migrate.

### Docs

- [#55](https://github.com/WSH032/pytauri/pull/55) - Add `integrate with nicegui` example `nicegui-app`. See `examples/nicegui-app`.
- [#52](https://github.com/WSH032/pytauri/pull/52) - update `examples/tauri-app` `main.rs` for new API to prepare python interpreter.
- [#52](https://github.com/WSH032/pytauri/pull/52) - add the usage of `multiprocessing.freeze_support` in `examples/tauri-app` `__main__.py`.

### Changed

- [#46](https://github.com/WSH032/pytauri/pull/46) - bump `tauri` to `v2.2`

## [0.1.0-beta]

[unreleased]: https://github.com/WSH032/pytauri/tree/HEAD
[0.1.0-beta]: https://github.com/WSH032/pytauri/releases/tag/v0.1.0-beta
