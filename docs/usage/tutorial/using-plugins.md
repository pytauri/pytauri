# Using Tauri Plugins

The Tauri team and community have developed some [plugins](https://tauri.app/plugin/), you can use them by:

1. Official Tauri plugins usually provide corresponding JavaScript APIs, which you can use directly on the frontend.
2. Write your own Rust functions using pyo3 and expose them to Python: <https://github.com/pytauri/pytauri/discussions/45#discussioncomment-11870767>

    **We encourage you to distribute plugins written in this way to benefit the entire community 💪.**

In addition, PyTauri has already integrated some official Tauri plugins. Below, we use [tauri-plugin-notification] as an example to demonstrate how to use a PyTauri-integrated plugins.

[tauri-plugin-notification]: https://github.com/tauri-apps/tauri-plugin-notification

## All plugins we support

| Plugin | JS Docs | Rust Docs | Python Docs |
|--------|---------|-----------|-------------|
| plugin-notification | [JS docs](https://tauri.app/reference/javascript/notification/) | [Rust docs](https://docs.rs/tauri-plugin-notification/latest/tauri_plugin_notification/) | [Python docs][pytauri_plugins.notification] |
| plugin-dialog | [JS docs](https://tauri.app/reference/javascript/dialog/) | [Rust docs](https://docs.rs/tauri-plugin-dialog/latest/tauri_plugin_dialog/) | [Python docs][pytauri_plugins.dialog] |

## Using the plugin

### install tauri plugin

All PyTauri plugins are just Python bindings, which means you need to [initialize the underlying Tauri extensions normally](https://github.com/tauri-apps/tauri-plugin-notification/blob/665d8f08bcf2e8af3c0f95af12ca1f06d71a0d6d/README.md#install):

```bash
pnpm tauri add notification
```

### expose the pyo3 bingings to python

Enable the `pytauri` feature:

```diff title="src-tauri/Cargo.toml"
[dependencies]
# ...
-pytauri = { version = "*" }
+pytauri = { version = "*", features = ["plugin-notification"] }
```

### use plugin api from python

The PyTauri API maps very well to the original Rust API of the plugin. You can refer to the [Js docs](https://tauri.app/plugin/notification/), [Rust docs](https://docs.rs/tauri-plugin-notification/latest/tauri_plugin_notification/) and [Python docs][pytauri_plugins.notification] to understand how to use it:

!!! tip
    `pytauri_plugins` is distributed as part of the [`pytauri`](https://pypi.org/project/pytauri/) package on PyPI.
    Therefore, running `pip install pytauri` will also install it.

```python title="src-tauri/python/tauri_app/__init__.py"
--8<-- "docs_src/tutorial/plugin.py"
```
