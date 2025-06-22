"""[tauri::ipc](https://docs.rs/tauri/latest/tauri/ipc/index.html)"""

from typing import (
    TYPE_CHECKING,
    Any,
    Generic,
    Optional,
    Union,
    final,
)

from typing_extensions import ReadOnly, TypeAliasType, TypedDict, TypeVar

from pytauri.ffi._ext_mod import pytauri_mod

__all__ = [
    "ArgumentsType",
    "Channel",
    "Headers",
    "Invoke",
    "InvokeResolver",
    "JavaScriptChannelId",
    "ParametersType",
]

_ipc_mod = pytauri_mod.ipc

if TYPE_CHECKING:
    from pytauri.ffi.lib import AppHandle
    from pytauri.ffi.webview import Webview, WebviewWindow

_InvokeResponseBody = TypeAliasType("_InvokeResponseBody", Union[str, bytes])
"""The body of an IPC response.

- str: InvokeResponseBody:Json (Any)
- bytes: InvokeResponseBody:Raw (ArrayBuffer)
"""

Headers = TypeAliasType("Headers", list[tuple[bytes, bytes]])
"""[http::header::HeaderMap::iter](https://docs.rs/http/latest/http/header/struct.HeaderMap.html#method.iter)

`(key, value)` pairs of headers.

> Each key will be yielded once per associated value.
> So, if a key has 3 associated values, it will be yielded 3 times.

```python
[(b"key0", b"value00"), (b"key0", b"value01"), (b"key1", b"value1")]
```

!!! tip
    You can use libraries like [multidict][] or [httpx.Headers] to convert it into dict
    for more efficient retrieval of a specific header.

    [httpx.Headers]: https://www.python-httpx.org/api/#headers
"""


class ParametersType(TypedDict, total=False):
    """The parameters of a command.

    All keys are optional, and values can be of any type.
    If a key exists, it will be assigned a value corresponding to [ArgumentsType][pytauri.ffi.ipc.ArgumentsType].
    """

    body: ReadOnly[Any]
    """Whatever. We just use the `key`, not the `value`."""
    app_handle: ReadOnly[Any]
    """Whatever. We just use the `key`, not the `value`."""
    webview_window: ReadOnly[Any]
    """Whatever. We just use the `key`, not the `value`."""
    headers: ReadOnly[Any]
    """Whatever. We just use the `key`, not the `value`."""


class ArgumentsType(TypedDict, total=False):
    """The bound arguments of a command.

    Each key is optional, depending on the keys of the bound [ParametersType][pytauri.ffi.ipc.ParametersType].

    You can use it like `**kwargs`, for example `command(**arguments)`.
    """

    body: bytes
    """The body of this ipc message."""
    app_handle: "AppHandle"
    """The handle of the app."""
    webview_window: "WebviewWindow"
    """The `WebviewWindow` of this `Invoke`."""
    headers: Headers
    """The headers of this ipc message."""


_ArgumentsTypeVar = TypeVar("_ArgumentsTypeVar", default=dict[str, Any])


if TYPE_CHECKING:

    @final
    class Invoke:
        """[tauri::ipc::Invoke](https://docs.rs/tauri/latest/tauri/ipc/struct.Invoke.html)"""

        @property
        def command(self) -> str:
            """The name of the current command."""
            ...

        def bind_to(
            self, parameters: ParametersType
        ) -> Optional["InvokeResolver[_ArgumentsTypeVar]"]:
            """Consumes this `Invoke` and binds parameters.

            If the frontend illegally calls the IPC,
            this method will automatically reject this `Invoke` and return `None`.

            The return value [InvokeResolver.arguments][pytauri.ffi.ipc.InvokeResolver.arguments]
            is not the same object as the input `parameters`.
            """

        def resolve(self, value: _InvokeResponseBody) -> None:
            """Consumes this `Invoke` and resolves the command with the given value."""
            ...

        def reject(self, value: str) -> None:
            """Consumes this `Invoke` and rejects the command with the given value."""
            ...

    @final
    class InvokeResolver(Generic[_ArgumentsTypeVar]):
        """[tauri::ipc::InvokeResolver](https://docs.rs/tauri/latest/tauri/ipc/struct.InvokeResolver.html)"""

        @property
        def arguments(self) -> _ArgumentsTypeVar:
            """The bound arguments of the current command."""
            ...

        def resolve(self, value: _InvokeResponseBody) -> None:
            """Consumes this `InvokeResolver` and resolves the command with the given value."""

        def reject(self, value: str) -> None:
            """Consumes this `InvokeResolver` and rejects the command with the given value."""
            ...

    @final
    class JavaScriptChannelId:
        """[tauri::ipc::JavaScriptChannelId](https://docs.rs/tauri/latest/tauri/ipc/struct.JavaScriptChannelId.html)"""

        @staticmethod
        def from_str(value: str, /) -> "JavaScriptChannelId":
            """Parse a string to a `JavaScriptChannelId`.

            Raises:
                ValueError: If the string is ivnalid.
                TypeError: If the `value` is not a string.
            """
            ...

        def channel_on(self, webview: Webview, /) -> "Channel":
            """Gets a `Channel` for this channel ID on the given `Webview`."""
            ...

    @final
    class Channel:
        """[tauri::ipc::Channel](https://docs.rs/tauri/latest/tauri/ipc/struct.Channel.html)"""

        def id(self, /) -> int:
            """The channel identifier."""
            ...

        def send(self, data: _InvokeResponseBody, /) -> None:
            """Sends the given data through the channel.

            Args:
                data: The data to send.
                    - If `str`, it will be deserialized as JSON on the frontend.
                    - If `bytes`, it will be sent as `ArrayBuffer` to the frontend.
            """
            ...

else:
    Invoke = _ipc_mod.Invoke
    InvokeResolver = _ipc_mod.InvokeResolver
    JavaScriptChannelId = _ipc_mod.JavaScriptChannelId
    Channel = _ipc_mod.Channel
