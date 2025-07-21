# Using PyTauri

!!! note
    The dependency versions specified in the following tutorial are the versions at the time of writing. There may be newer versions available when you use it.

## Create venv

Create a virtual environment using `uv`:

```bash
uv venv --python-preference only-system
```

!!! warning
    `--python-preference only-system` is necessary. Using `uv`'s managed Python may result in not finding dynamic libraries.

activate the virtual environment:

=== "bash"
    ```bash
    source .venv/bin/activate
    ```

=== "powershell"
    ```powershell
    .venv\Scripts\Activate.ps1
    ```

## Init pyproject

Create the `src-tauri/python/tauri_app` folder to store Python code, and add the following file:

ref: <https://packaging.python.org/en/latest/guides/writing-pyproject-toml/>

```toml title="src-tauri/pyproject.toml" hl_lines="8 9"
[project]
name = "tauri-app"  # (1)!
version = "0.1.0"
description = "Add your description here"
requires-python = ">=3.9"
dependencies = ["pytauri == 0.7.*"]  # (2)!

[project.entry-points.pytauri]
ext_mod = "tauri_app.ext_mod"

[build-system]
requires = ["setuptools>=61"]
build-backend = "setuptools.build_meta"

[tool.setuptools.packages]
find = { where = ["python"] }  # (3)!
```

1. your python package name.
2. This is the version at the time of writing this tutorial. There may be a newer version of pytauri available when you use it.
3. the folder where your python code is stored, i.e., `src-tauri/python`.

!!! tip
    Note the highlighted `project.entry-points`. We will explain its specific meaning when building the Wheel. For now, let's continue with the tutorial.

## Install your project

Use `uv` to install your Python package in [editable mode](https://setuptools.pypa.io/en/latest/userguide/development_mode.html):

```bash
uv pip install -e src-tauri
```

Add following code:

```python title="src-tauri/python/tauri_app/__init__.py"
--8<-- "docs_src/tutorial/using_pytauri/__init__.py"
```

```python title="src-tauri/python/tauri_app/__main__.py"
--8<-- "docs_src/tutorial/using_pytauri/__main__.py"
```

## Run pytauri from rust

Add following dependencies to `Cargo.toml`:

ref: <https://doc.rust-lang.org/cargo/reference/cargo-targets.html#binaries>

```toml title="src-tauri/Cargo.toml"
# ...

[[bin]]
# the same as the package name
name = "tauri-app"
path = "src/main.rs"
required-features = ["pytauri/standalone"]

[dependencies]
# ...
pytauri = { version = "0.7" }  # (1)!
pyo3 = { version = "0.25" }  # (2)!
```

1. This is the version at the time of writing this tutorial. There may be a newer version of pytauri available when you use it.
2. This is the version at the time of writing this tutorial. There may be a newer version of pytauri available when you use it.

Also, enable the `pytauri/standalone` feature:

```json title="src-tauri/tauri.conf.json"
{
    "build": {
        "features": ["pytauri/standalone"]
    }
}
```

!!! warning
    If you do not enable `required-features` in `tauri-cli`, [cargo will silently skip building your `main.rs` executable file](https://github.com/rust-lang/cargo/issues/4663).

Change following rust code:

```rust title="src-tauri/src/lib.rs"
--8<-- "docs_src/tutorial/using_pytauri/lib.rs"
```

```rust title="src-tauri/src/main.rs"
--8<-- "docs_src/tutorial/using_pytauri/main.rs"
```

## Launch the app in dev mode

The `tauri-cli` has the ability to watch code changes and hot reload. Before starting, we need to add the following file to tell `tauri-cli` to ignore the python bytecode:

ref: <https://tauri.app/develop/#reacting-to-source-code-changes>

```gitignore title="src-tauri/.taurignore"
__pycache__
```

Also, we need tell `vite` to ignore `.venv`:

```ts title="vite.config.ts"
--8<-- "docs_src/tutorial/using_pytauri/vite.config.ts"
```

Run `#!bash pnpm tauri dev`, and after recompiling, you will see a window similar to the previous step.

Try modifying the Python code, and you will notice that the Python code is quickly reloaded **without needing to recompile the Rust code**.

## Next Steps

Next, we will demonstrate how to package your application.
