# pyo3-utils

## [Unreleased]

### Added

- [#220](https://github.com/pytauri/pytauri/pull/220) - feat: support registering plugin from python.

    `from_py_dict::derive_from_py_dict!` can now accept struct with no fields:

    ```rust
    derive_from_py_dict!(Foo {});
    ```

## [0.3.0]

### Added

- [#160](https://github.com/pytauri/pytauri/pull/160) - feat: added unstable mod `from_py_dict`.

    Refer to [PyO3/pyo3#5163](https://github.com/PyO3/pyo3/issues/5163).

- [#158](https://github.com/pytauri/pytauri/pull/158) - chore: bump `pyo3` to `0.25`.

## [0.2.0]

### Added

- [#141](https://github.com/pytauri/pytauri/pull/141) - chore: bump `pyo3` to `0.24.1`.

## [0.1.0]

### Docs

- [#57](https://github.com/pytauri/pytauri/pull/57) - add documentation to `mod py_match` indicating it should only be used for `state-machine-like` `enum`, not for `Union-like` `enum`.

### Internal

- [#79](https://github.com/pytauri/pytauri/pull/79) - refactor: use `rust v1.82` feature `Omitting empty types in pattern matching` in place of `Result<T, Infallible>::unwrap`.

## [0.1.0-beta.0]

[unreleased]: https://github.com/pytauri/pytauri/tree/HEAD
[0.3.0]: https://github.com/pytauri/pytauri/releases/tag/rs/pyo3-utils/v0.3.0
[0.2.0]: https://github.com/pytauri/pytauri/releases/tag/rs/pyo3-utils/v0.2.0
[0.1.0]: https://github.com/pytauri/pytauri/releases/tag/rs/pyo3-utils/v0.1.0
[0.1.0-beta.0]: https://github.com/pytauri/pytauri/releases/tag/rs/pyo3-utils/v0.1.0-beta.0
