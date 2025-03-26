# Using tauri plugins

The Tauri team and community have developed some [plugins](https://tauri.app/plugin/), you can use them by:

1. Official Tauri plugins usually provide corresponding JavaScript APIs, which you can use directly on the frontend.
2. [Write your own Rust functions using pyo3 and expose them to Python](https://pyo3.rs/v0.23.3/function.html).

PyTauri has provided Python APIs for some Tauri plugins using the second method, and they are called `pytauri-plugin-*`.

Below, we use `pytauri-plugin-notification` as an example to demonstrate how to use pytauri plugins.

## Using the plugin

### install tauri plugin

All PyTauri plugins are just Python bindings, which means you need to initialize the underlying Tauri extensions normally:

```bash
pnpm tauri add notification
```

### expose the pyo3 bingings to python

PyTauri plugins usually export their pyo3 API with the following conventions:

1. Export a Rust `mod` with the same name as the plugin at the top level.
2. Export a function named `pymodule_export` at the top level.
    - The pyo3 API of `pytauri` itself is exported in this way: `pytauri::pymodule_export`

`pytauri-plugin-notification` uses the first method.

Add the rust dependency:

```bash
cd src-tauri
cargo add pytauri-plugin-notification@0.4  # (1)!
cd ..
```

1. This is the version at the time of writing this tutorial. There may be a newer version of pytauri available when you use it.

ref to <https://pyo3.rs/v0.23.3/module.html> to export the pyo3 bindings:

```rust title="src-tauri/src/lib.rs"
use pyo3::prelude::*;
// ...

#[pymodule(gil_used = false)]
#[pyo3(name = "ext_mod")]
pub mod ext_mod {

    #[pymodule_export]
    use pytauri_plugin_notification::notification;

    // ...
}
```

### use plugin api from python

Add the python dependency:

```toml title="src-tauri/pyproject.toml"
# ...

[project]
# ...
dependencies = [
    # ...
    "pytauri-plugin-notification == 0.4.*",  # (1)!
]
```

1. This is the version at the time of writing this tutorial. There may be a newer version of pytauri available when you use it.

!!! tip
    After adding dependencies, you need to use commands like `uv sync` or `uv pip install` to synchronize your dependency environment.

The PyTauri API maps very well to the original Rust API of the plugin. You can refer to the [Rust documentation](https://tauri.app/plugin/notification/) to understand how to use it:

```python title="src-tauri/python/__init__.py"
--8<-- "docs_src/tutorial/plugin.py"
```
