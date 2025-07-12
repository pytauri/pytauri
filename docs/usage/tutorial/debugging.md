# Debugging

## PyTauri Runtime

You can use the following APIs to inspect the current pytauri runtime configuration:

- [pytauri.IS_DEV][pytauri.IS_DEV]: Indicates if running in `tauri dev` mode
- [pytauri.VERSION][pytauri.VERSION]: The current version of tauri
- [pytauri.webview_version][pytauri.webview_version]: The current version of the webview

## Debug in VSCode

### Python

Refer to <https://code.visualstudio.com/docs/python/debugging#_debugging-by-attaching-over-a-network-connection>.

#### Dependencies

First, install the [`Python Debugger` extension](https://marketplace.visualstudio.com/items?itemName=ms-python.debugpy) in VSCode.

Then, add [`debugpy`](https://github.com/microsoft/debugpy) as your python [development dependency](https://docs.astral.sh/uv/concepts/projects/dependencies/#development-dependencies>) and install it:

```toml title="src-tauri/pyproject.toml"
# ...

[dependency-groups]
dev = [
    # ...
    "debugpy == 1.*"
]
```

#### Configure `launch.json`

Refer to <https://code.visualstudio.com/docs/python/debugging#_initialize-configurations>.

Add the following configuration:

```json title=".vscode/launch.json"
{
    "version": "0.2.0",
    "configurations": [
        {
            "name": "Python Debugger: Remote Attach",
            "type": "debugpy",
            "request": "attach",
            "connect": {
                "host": "localhost",
                "port": 5678
            },
        },
    ]
}
```

### Rust

Refer to <https://github.com/vadimcn/codelldb/blob/v1.11.5/MANUAL.md#attaching-to-a-running-process>.

#### Dependencies

First, install the [`CodeLLDB` extension](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb) in VSCode.

Then, add [`codelldb`][codelldb] as your python [development dependency](https://docs.astral.sh/uv/concepts/projects/dependencies/#development-dependencies) and install it:

```toml title="src-tauri/pyproject.toml"
# ...

[dependency-groups]
dev = [
    # ...
    "codelldb == 0.1.*"  # (1)!
]
```

1. This is the version at the time of writing this tutorial. There may be a newer version of pytauri available when you use it.

#### Configure `settings.json`

Refer to <https://github.com/vadimcn/codelldb/blob/v1.11.5/MANUAL.md#rpc-server>.

Add the following configuration:

```json title=".vscode/settings.json"
{
    "lldb.rpcServer": {
        "host": "localhost",
        "port": 9552,
        "token": "secret",
    },
}
```

### Start Debugging

Add the following code to your app:

```python title="src-tauri/python/tauri_app/__main__.py"
--8<-- "docs_src/tutorial/debugging/__main__.py"
```

Start your app:

=== "windows"
    ```powershell
    $env:PYTAURI_DEBUG_PY = "1"
    $env:PYTAURI_DEBUG_RS = "1"
    pnpm tauri dev
    ```

=== "unix"
    ```bash
    export PYTAURI_DEBUG_PY="1"
    export PYTAURI_DEBUG_RS="1"
    pnpm tauri dev
    ```

When you see the `"Waiting for debugger to attach..."` output, you can attach the `Python Debugger` to your app to start debugging Python code:

![debugging](https://github.com/user-attachments/assets/d8d3763d-0285-4265-b071-8cdcd5921efa)

The `CodeLLDB` debugger will automatically attach to your app; you do not need to start it manually.
