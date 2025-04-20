# PyTauri Wheel

The `pytauri-wheel` is built using the method described in [`tutorial/build-wheel`](../tutorial/build-wheel.md) to provide a precompiled [pytauri.EXT_MOD][]. This means you can use [pytauri][] without writing any Rust code and needing a Rust compiler.

## Installation

### PyPi

#### Prerequisites

=== "windows"

    Install WebView2 by visiting the [WebView2 Runtime download section](https://developer.microsoft.com/en-us/microsoft-edge/webview2/#download-section). Download the "Evergreen Bootstrapper" and install it.

=== "linux"

    As mentioned in <https://tauri.app/distribute/debian/#debian>:

    - `libwebkit2gtk-4.1-0`
    - `libgtk-3-0`
    - `libappindicator3-1`

=== "macOS"

    *Nothing*

#### Wheel

We provide precompiled Wheels for the following platforms and supported Python versions:

| OS | Arch |
| --- | --- |
| windows-2022 | x64 |
| windows-11 | arm64 |
| manylinux_2_35 | x64, arm64 |
| macOS-13 | intel |
| macOS-14 | arm64 |

```bash
pip install "pytauri-wheel == 0.4.*"  # (1)!
```

1. This is the version at the time of writing this tutorial. There may be a newer version of pytauri available when you use it.

### Sdist

If the above platform and Python version requirements are not met, the `Wheel` will be automatically built (compiled) from the [`source distribution`](https://packaging.python.org/en/latest/discussions/package-formats/#what-is-a-source-distribution) when installing from `pypi`.

This requires you to meet the [`tutorial/#prerequisites`](../tutorial/index.md#prerequisites).

## Usage

The development experience of `pytauri-wheel` is almost the same as Rust `tauri`. You can find a complete example in [`examples/tauri-app-wheel`](https://github.com/pytauri/pytauri/tree/main/examples/tauri-app-wheel).

### Tauri Config

First, we need to create a `Tauri.toml`, refer to <https://tauri.app/develop/configuration-files/#tauri-config>:

```toml title="/Tauri.toml"
--8<-- "docs_src/pytauri_wheel/example/Tauri.toml"
```

!!! tip
    `pytauri-wheel` also support `tauri.conf.json` and `tauri.conf.json` and [`tauri.linux.conf.json`](https://tauri.app/develop/configuration-files/#platform-specific-configuration) and etc.

### Capabilities

Refer to：

- <https://tauri.app/security/capabilities/>
- [`tutorial/py-js-ipc`](../tutorial/py-js-ipc.md)

Create the following capabilities file to enable `ipc` permissions:

```toml title="/capabilities/default.toml"
--8<-- "docs_src/pytauri_wheel/example/capabilities/default.toml"
```

!!! tip
    `pytauri-wheel` also support `json` and `json5` capabilities files.

### PyTauri Wheel App

The final step, refer to:

- [`tutorial/using-pytauri`](../tutorial/using-pytauri.md)
- [`tutorial/py-js-ipc/`](../tutorial/py-js-ipc.md)
- [pytauri][] API
- [pytauri_wheel][] API

```py title="/main.py"
--8<-- "docs_src/pytauri_wheel/example/main.py:code"
```

!!! note
    The frontend assets directory must the same as the one in `Tauri.toml`.

```html title="/frontend/index.html"
--8<-- "docs_src/pytauri_wheel/example/frontend/index.html"
```

You get following directory structure:

```
📁 {SRC_TAURI_DIR}
├── 📁 capabilities
│   └─── 📄 default.toml
├── 📁 frontend
│   └─── 📄 index.html
├── 📄 Tauri.toml
└── 📄 main.py
```

Then run the app:

```bash
python main.py
```

## Development Mode

If you want to use frontend dev server such as `vite` in development, or you want to dynamically/programmatically set tarui config, please refer to [pytauri_wheel.lib.context_factory(tauri_config)][]

## Tauri Plugins

`pytauri-wheel` enables a set of Tauri plugins during compilation, please refer to [pytauri_wheel.lib.builder_factory][].

For example, the [tauri-plugin-dialog](https://tauri.app/plugin/dialog/) can be used in the frontend as follows:

```js
import { ask } from '@tauri-apps/plugin-dialog';
// when using `"withGlobalTauri": true`, you may use
// const { ask } = window.__TAURI__.dialog;
```

!!! warning
    Remember to enable the permissions for these plugins in `/capabilities`, otherwise you will receive an error on the frontend.
