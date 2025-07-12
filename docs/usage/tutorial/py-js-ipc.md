# IPC between Python and JavaScript

**See [concepts/ipc](../concepts/ipc.md) for more information.**

---

pytauri implements the same IPC API as tauri. You can use it through [pytauri.Commands][].

This tutorial will demonstrate how to use pytauri's IPC API by rewriting the `fn greet` command in `src-tauri/src/lib.rs` in Python.

## Enable pytauri ipc permission

pytauri internally implements IPC through `tauri-plugin-pytauri`.
You need to add it to the dependencies so that you can enable its permission in tauri.

```toml title="src-tauri/Cargo.toml"
# ...

[dependencies]
# ...
tauri-plugin-pytauri = { version = "0.6" }  # (1)!
```

1. This is the version at the time of writing this tutorial. There may be a newer version of pytauri available when you use it.

Refer to <https://tauri.app/security/capabilities/> to add the permission:

```json title="src-tauri/capabilities/default.json"
{
    // ...
    "permissions": [
        // ...
        "pytauri:default"
    ]
}
```

## IPC in python

### install dependencies

pytauri relies on [pydantic](https://github.com/pydantic/pydantic) for serialization and validation, and on [anyio](https://github.com/agronholm/anyio) for `asyncio`/`trio` support.

Therefore, you need to install these dependencies:

```toml title="src-tauri/pyproject.toml"
# ...

[project]
# ...
dependencies = [
    # ...
    "pydantic == 2.*",
    "anyio == 4.*"
]
```

!!! tip
    After adding dependencies, you need to use commands like `uv sync` or `uv pip install` to synchronize your dependency environment.

### add command

see [concepts/ipc](../concepts/ipc.md) for more information.

```python title="src-tauri/python/tauri_app/__init__.py"
--8<-- "docs_src/tutorial/invoke_handler.py:command"
```

### generate invoke handler for app

```python title="src-tauri/python/tauri_app/__init__.py"
--8<-- "docs_src/tutorial/invoke_handler.py"
```

## IPC in JavaScript

pytauri provides an API similar to the [`invoke`](https://tauri.app/reference/javascript/api/namespacecore/#invoke) function in `@tauri-apps/api/core` through `tauri-plugin-pytauri-api`.

First, install it: `#!bash pnpm add tauri-plugin-pytauri-api`.

Now, you can invoke the command from your JavaScript code:

```ts title="src/main.ts"
--8<-- "docs_src/tutorial/cammand.ts:invoke"
```
