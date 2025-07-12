# Build Standalone Binary

pytauri can be distributed as a Python wheel or compiled into a standalone executable (a regular Tauri application).

Unlike packaging with `pyinstaller` after building as a wheel, compiling pytauri into an executable allows you to enjoy all the benefits brought by `tauri-cli`.

## Get portable Python

We will bundle [python-build-standalone](https://github.com/indygreg/python-build-standalone) as a portable Python for distribution.

Please [download the Python version](https://gregoryszorc.com/docs/python-build-standalone/main/running.html#obtaining-distributions) you need. Usually, you will use these versions:

- `cpython-*-x86_64-pc-windows-msvc-install_only_stripped.tar.gz`
- `cpython-*-x86_64-unknown-linux-gnu-install_only_stripped.tar.gz`
- `cpython-*-aarch64-apple-darwin-install_only_stripped.tar.gz`

Extract it to `src-tauri/pyembed`, make sure the file layout is as follows:

=== "windows"
    ```tree
    ├── src-tauri/pyembed/python
        ├── python.exe
        ├── python3.dll
        └── ...
    ```

=== "unix"
    ```tree
    ├── src-tauri/pyembed/python
        ├── bin/
        ├── include/
        ├── lib/
        └── share/
    ```

Tell `tauri-cli` to ignore it during `tauri dev`:

```gitignore title="src-tauri/.taurignore"
# ...
/pyembed/
```

!!! tip
    If you are using an IDE based on `pyright`/`pylance`, please create a `pyproject.toml` file in the root directory of your project (not `src-tauri/pyproject.toml`) and add the following configuration to tell `pyright` not to analyze `src-tauri/pyembed`, as it will consume a large amount of memory:

    ```toml title="pyproject.toml"
    [tool.pyright]
    # see: <https://github.com/microsoft/pyright/blob/1.1.391/docs/configuration.md#environment-options>
    exclude = [
        "**/node_modules",
        "**/__pycache__",
        # 👇 necessary, because when `tauri-cli bundles python,
        # it will copy `pyembed` to the target directory (i.e., rust output dir).
        "**/target",
        # 👆
        "**/dist",
        "**/.venv",
        "**/.*",
        "src-tauri/pyembed/",
        "src-tauri/frontend/",
    ]
    ```

## Install your project into the embedded Python environment

=== "windows"
    ```powershell
    $env:PYTAURI_STANDALONE="1"  # see `setup.py`

    # `tauri-app` is your python package name.
    uv pip install `
        --exact `
        --python=".\src-tauri\pyembed\python\python.exe" `
        --reinstall-package=tauri-app `
        .\src-tauri
    ```

=== "unix"
    ```bash
    export PYTAURI_STANDALONE="1"  # see `setup.py`

    # `tauri-app` is your python package name.
    uv pip install \
        --exact \
        --python="./src-tauri/pyembed/python/bin/python3" \
        --reinstall-package=tauri-app \
        ./src-tauri
    ```
!!! warning
    Unlike `editable install` during development, you need to reinstall your project every time you modify the Python code.

## Configure `tauri-cli`

ref: <https://tauri.app/reference/config/#bundle>

Create following `tauri-cli` configuration file:

```json title="src-tauri/tauri.bundle.json"
{
    "bundle": {
        "active": true,
        "targets": "all",
        "resources": {
            "pyembed/python": "./"
        }
    }
}
```

ref: <https://doc.rust-lang.org/cargo/reference/profiles.html>

Add the following configuration to `Cargo.toml`:

```toml title="src-tauri/Cargo.toml"
# ...

[profile.bundle-dev]
inherits = "dev"

[profile.bundle-release]
inherits = "release"
```

## Build and bundle

ref: <https://pyo3.rs/v0.23.3/building-and-distribution.html#configuring-the-python-version>

Indicate pyo3 to use the embedded Python interpreter through environment variables, so it does not mistakenly use the system Python interpreter.

=== "windows"
    ```powershell
    $env:PYO3_PYTHON = (Resolve-Path -LiteralPath ".\src-tauri\pyembed\python\python.exe").Path
    ```

=== "unix"
    ```bash
    export PYO3_PYTHON=$(realpath ./src-tauri/pyembed/python/bin/python3)
    ```

Configure `RUSTFLAGS`:

=== "windows"
    *Nothing you need to do. Only unix need to set `RUSTFLAGS`.*

=== "unix"
    - There is currently an [issue](https://github.com/astral-sh/python-build-standalone/issues/374) with the `sysconfig` of `python-build-standalone`,
        which causes `pyo3` to fail to automatically find `libpython3` during compilation, so we need to set it manually.
    - We use tauri's [`resource_dir`](https://docs.rs/tauri-utils/latest/tauri_utils/platform/fn.resource_dir.html) to bundle the portable Python,
        so we need to set `rpath` to tell our binary how to find the bundled `libpython3` at runtime.

    === "linux"
        ```bash
        # `tauri-app` is your app `productName` in `tauri.conf.json`.
        export RUSTFLAGS=" \
            -C link-arg=-Wl,-rpath,\$ORIGIN/../lib/tauri-app/lib \
            -L $(realpath ./src-tauri/pyembed/python/lib)"
        ```
    === "macos"
        ```bash
        export RUSTFLAGS=" \
            -C link-arg=-Wl,-rpath,@executable_path/../Resources/lib \
            -L $(realpath ./src-tauri/pyembed/python/lib)"
        ```

Finally, use `tauri-cli` to bundle:

```powershell
pnpm -- tauri build --config="src-tauri/tauri.bundle.json" -- --profile bundle-release
```

!!! warning
    DO NOT set `bundle.resources` in `tauri.conf.json` directly.

    The `tauri-cli` will copy `bundle.resources` to `target/release(debug)`, which is in the same location as your executable.
    This will incorrectly cause the copied Python environment to be the Python environment linked at runtime during `tauri dev`.
    However, during development, you should use a `venv` virtual environment.

    By using `--profile bundle-release`, we ensure that `target/release(debug)` is not affected, allowing you to use `tauri dev` normally.
