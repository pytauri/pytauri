# IPC

## Calling Python from the Frontend

Ref:

- <https://tauri.app/develop/calling-rust/>
- [pytauri.ipc.Commands][]

pytauri implements IPC API consistent with rust tauri. Reading tauri's documentation is like reading pytauri's documentation.

### Commands

#### Registering Commands

You can register a command handler using the decorator [@Commands.command][pytauri.ipc.Commands.command].

Similar to `tauri::command!`, the `handler` signature can be arbitrary. We will use [inspect.signature][] to inspect its signature and dynamically pass the required parameters.

!!! info
    You might have seen this pattern in `FastAPI`🤓.

The currently supported signature pattern is [ArgumentsType][pytauri.ipc.ArgumentsType]. You must ensure that the parameter names and type annotations are correct, and `@Commands.command` will check them.

```python
--8<-- "docs_src/concepts/ipc/reg_cmd.py"
```

#### Deserializing the Body using `BaseModel`

For the `body` argument, it is of type `bytes`, allowing you to pass binary data such as files between the frontend and backend.

However, in most cases, we want strong type checking when calling. Rust `tauri` achieves this through `serde`, while `pytauri` uses [pydantic](https://github.com/pydantic/pydantic).

!!! info
    `pydantic` is a super-fast Python validation and serialization library written in `rust`/`pyo3` 🤓.

If you use [BaseModel][pydantic.BaseModel]/[RootModel][pydantic.RootModel] as the type annotation for the `body` parameter/return value, pytauri will automatically serialize/deserialize it for you:

```python
--8<-- "docs_src/concepts/ipc/serde_body.py"
```

#### Deserializing the Body using arbitrary types

!!! info
    This is an experimental feature. If we find that it causes more problems than benefits, it may be removed in the future. You may also encounter some bugs—please report them on GitHub issues!

For types like `str`, it would be cumbersome to explicitly declare `#!python StrModel = RootModel[str]` every time. Since pytauri `v0.7`, if you use types other than `bytes`/`BaseModel`/`RootModel` as the type annotation for the `body` parameter or return value, pytauri will automatically convert them to [BaseModel][pydantic.BaseModel]/[TypeAdapter][pydantic.TypeAdapter] behind the scenes.

```python
--8<-- "docs_src/concepts/ipc/serde_body_as_any.py"
```

??? tip "Implementation details"
    These are the current implementation details (which may change in the future). This means you need to pay attention to the [lru_cache][functools.lru_cache] cache missing issue:

    ```python
    --8<-- "docs_src/concepts/ipc/serde_body_as_any_impl.py"
    ```

#### Generate Invoke Handler for App

To execute async commands, we need an async runtime. We use [anyio.from_thread.BlockingPortal][] as the async runtime in a child thread (the main thread is used for the Tauri app's event loop).

Refer to the [anyio docs](https://anyio.readthedocs.io/en/stable/threads.html#calling-asynchronous-code-from-an-external-thread) for more information.

You can obtain a `BlockingPortal` as follows:

- [anyio.from_thread.start_blocking_portal][]
- [anyio.from_thread.BlockingPortalProvider][]

After that, you generate an `invoke_handler` and pass it to the `App`, similar to Rust's `tauri::generate_handler`:

```python
--8<-- "docs_src/concepts/ipc/gen_handler.py"
```

The key point here is that **you must not close the `BlockingPortal` (i.e., do not exit the context manager) while [App.run][pytauri.App.run] is still running**.

If you want to obtain this `invoke_handler` and keep the `BlockingPortal` running, you can use [contextlib.ExitStack][] to achieve this:

```python
--8<-- "docs_src/concepts/ipc/exit_stack.py"
```

You can also spawn tasks in the async runtime (in the child thread) from the main thread in a thread-safe manner using the `portal`: <https://anyio.readthedocs.io/en/stable/threads.html#spawning-tasks-from-worker-threads>

#### Calling Commands

```typescript
--8<-- "docs_src/concepts/ipc/calling_cmd.ts"
```

The usage of `pyInvoke` is exactly the same as Tauri's `invoke`:

- <https://tauri.app/develop/calling-rust/#basic-example>
- <https://tauri.app/reference/javascript/api/namespacecore/#invoke>

#### Returning Errors to the Frontend

Similar to `FastAPI`, as long as you throw an [InvokeException][pytauri.ipc.InvokeException] in the `command`, the promise will reject with the error message.

```python
--8<-- "docs_src/concepts/ipc/ret_exec.py"
```

#### Accessing Request Headers

ref: <https://tauri.app/develop/calling-rust/#accessing-raw-request>

When passing binary data, you may find it useful to send and access custom request headers:

```python
--8<-- "docs_src/concepts/ipc/accessing_headers.py"
```

```typescript
--8<-- "docs_src/concepts/ipc/accessing_headers.ts"
```

## Calling Frontend from Python

Ref:

- <https://tauri.app/develop/calling-frontend/>
- [pytauri.ipc.JavaScriptChannelId][] and [pytauri.ipc.Channel][]
- [pytauri.webview.WebviewWindow.eval][]

### Channels

> Channels are designed to be fast and deliver ordered data. They are used internally for streaming operations such as download progress, child process output, and WebSocket messages.

To use a `channel`, you only need to add the [JavaScriptChannelId][pytauri.ipc.JavaScriptChannelId] field to the `BaseModel`/`RootModel`, and then use [JavaScriptChannelId.channel_on][pytauri.ipc.JavaScriptChannelId.channel_on] to get a [Channel][pytauri.ipc.Channel] instance.

!!! info
    `JavaScriptChannelId` itself is a `RootModel`, so you can directly use it as the `body` parameter.

```python
--8<-- "docs_src/concepts/ipc/py_channel.py"
```

```typescript
--8<-- "docs_src/concepts/ipc/js_channel.ts"
```

### Evaluating JavaScript

You can use [WebviewWindow.eval][pytauri.webview.WebviewWindow.eval] to evaluate JavaScript code in the frontend.

## Event System

Ref:

- <https://tauri.app/develop/calling-frontend/#event-system>
- <https://tauri.app/develop/calling-rust/#event-system>
- [pytauri.Listener][]
- [pytauri.Emitter][]

> Tauri ships a simple event system you can use to have bi-directional communication between Rust and your frontend.
>
> The event system was designed for situations where small amounts of data need to be streamed or you need to implement a multi consumer multi producer pattern (e.g. push notification system).
>
> The event system is not designed for low latency or high throughput situations. See the channels section for the implementation optimized for streaming data.
>
> The major differences between a Tauri command and a Tauri event are that events have no strong type support, event payloads are always JSON strings making them not suitable for bigger messages and there is no support of the capabilities system to fine grain control event data and channels.

See:

- [pytauri.Listener--examples][]
- [pytauri.Emitter--examples][]
