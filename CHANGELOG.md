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

- [#51](https://github.com/WSH032/pytauri/pull/51) - rs/pytauri: When the app executable is spawned by python `multiprocessing` to execute subprocess tasks, `fn append_ext_mod` will return a `PySystemExit` exception at the end of execution. At this point, you should exit the Rust code instead of continuing to run the Python app. Please refer to its documentation for more information.

    ```rust
    use pyo3::exceptions::PySystemExit;

    fn execute_python_script(py: Python<'_>) -> PyResult<()> {
        let ext_mod = wrap_pymodule!(ext_mod)(py).into_bound(py);

        // If spawned as subprocess for `multiprocessing`,
        // it will return `PySystemExit` after execution.
        if let Err(err) = append_ext_mod(ext_mod) {
            if err.is_instance_of::<PySystemExit>(py) && is_forking() {
                // just return to end the rust code normally,
                // don't execute your python app code.
                return Ok(());
            } else {
                return Err(err);
            }
        }

        // Or you can just return the error and handle it later.
        // Just dont execute your python app code is enough.
        //
        // ```rust
        // append_ext_mod(ext_mod)?;
        // ```


        // execute your python app.
        // ...
    }
    ```

### Docs

- [#51](https://github.com/WSH032/pytauri/pull/51) - `examples`: Ignore `PySystemExit` exception.

    ```rust
    use pyo3::exceptions::PySystemExit;

    let result = execute_python_script(py);

    result.inspect_err(|e| {
        if e.is_instance_of::<PySystemExit>(py) {
            // python interpreter requires to exit normally, it's not an error
            return;
        }

        // ...
    })
    ```

### Changed

- [#46](https://github.com/WSH032/pytauri/pull/46) - bump `tauri` to `v2.2`

## [0.1.0-beta]

[unreleased]: https://github.com/WSH032/pytauri/tree/HEAD
[0.1.0-beta]: https://github.com/WSH032/pytauri/releases/tag/v0.1.0-beta
