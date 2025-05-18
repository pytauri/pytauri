# ruff: noqa: D102

"""[tauri::self](https://docs.rs/tauri/latest/tauri/index.html)"""

from abc import ABC, abstractmethod
from collections.abc import Iterator
from enum import Enum, auto
from pathlib import Path
from typing import (
    TYPE_CHECKING,
    Any,
    Callable,
    NamedTuple,
    NewType,
    NoReturn,
    Optional,
    Protocol,
    Union,
    final,
)

from pydantic import NonNegativeInt
from typing_extensions import Never, Self, TypeAliasType, deprecated

from pytauri.ffi._ext_mod import pytauri_mod

__all__ = [
    "App",
    "AppHandle",
    "Assets",
    "Builder",
    "BuilderArgs",
    "CloseRequestApi",
    "Context",
    "DragDropEvent",
    "DragDropEventType",
    "Emitter",
    "Event",
    "EventId",
    "EventTarget",
    "EventTargetType",
    "ExitRequestApi",
    "ImplEmitter",
    "ImplListener",
    "ImplManager",
    "Listener",
    "Manager",
    "Position",
    "PositionType",
    "Rect",
    "RunEvent",
    "RunEventType",
    "Size",
    "SizeType",
    "Theme",
    "Url",
    "WebviewEvent",
    "WebviewEventType",
    "builder_factory",
    "context_factory",
]

if TYPE_CHECKING:
    from pytauri.ffi.ipc import Invoke


class _InvokeHandlerProto(Protocol):
    def __call__(self, invoke: "Invoke", /) -> Any: ...


_AppRunCallbackType = Callable[["AppHandle", "RunEventType"], None]

_EventHandlerType = Callable[["Event"], None]

_PhysicalPositionF64 = tuple[float, float]
"""[tauri::PhysicalPosition](https://docs.rs/tauri/latest/tauri/struct.PhysicalPosition.html)"""
_PhysicalPositionI32 = tuple[int, int]
"""[tauri::PhysicalPosition](https://docs.rs/tauri/latest/tauri/struct.PhysicalPosition.html)"""
_LogicalPositionF64 = tuple[float, float]
"""[tauri::LogicalPosition](https://docs.rs/tauri/latest/tauri/struct.LogicalPosition.html)"""
_PhysicalSizeU32 = tuple[NonNegativeInt, NonNegativeInt]
"""[tauri::PhysicalSize](https://docs.rs/tauri/latest/tauri/struct.PhysicalSize.html)"""
_LogicalSizeF64 = tuple[float, float]
"""[tauri::PhysicalSize](https://docs.rs/tauri/latest/tauri/struct.LogicalSize.html)"""

_VecPathBuf = list[Path]
"""[tauri::DragDropEvent::Enter::paths](https://docs.rs/tauri/latest/tauri/enum.DragDropEvent.html#variant.Enter.field.paths)"""

# TODO: export this type in rust [ext_mod::utils::assets] namespace
_AssetKey = TypeAliasType("_AssetKey", str)
"""[tauri::utils::assets::AssetKey](https://docs.rs/tauri-utils/latest/tauri_utils/assets/struct.AssetKey.html)"""


if TYPE_CHECKING:
    from pytauri.ffi.image import Image
    from pytauri.ffi.menu import Menu, MenuEvent
    from pytauri.ffi.path import PathResolver
    from pytauri.ffi.tray import TrayIcon, TrayIconEventType
    from pytauri.ffi.webview import WebviewWindow

    @final
    class App:
        """[Tauri::app](https://docs.rs/tauri/latest/tauri/struct.App.html)

        !!! warning
            This class is not thread-safe, and should not be shared between threads.

            - You can only use it on the thread it was created on.
            - And you need to ensure it is garbage collected on the thread it was created on,
                otherwise it will cause memory leaks.
        """

        def run(self, callback: Optional[_AppRunCallbackType] = None, /) -> NoReturn:
            """Consume and run this app, will block until the app is exited.

            Args:
                callback: a callback function that will be called on each event.
                    It will be called on the same thread that the app was created on,
                    so you should not block in this function.

            !!! note
                This function will call `std::process::exit` at the end to terminate the entire process,
                which means the Python interpreter cannot be properly finalized.
                If this is a problem for you, please use [pytauri.App.run_return][].

            !!! warning
                If `callback` is specified, it must not raise an exception,
                otherwise it is logical undefined behavior, and in most cases, the program will panic.
            """
            ...

        def run_return(self, callback: Optional[_AppRunCallbackType] = None, /) -> int:
            """Consume and run this application, returning its intended exit code.

            !!! warning
                `callback` has the same restrictions as [App.run][pytauri.App.run].
            """
            ...

        @deprecated(
            """When called in a loop (as suggested by the name), this function will busy-loop.
            To re-gain control of control flow after the app has exited, use `App::run_return` instead.
            See <https://docs.rs/tauri/latest/tauri/struct.App.html#method.run_iteration> for more details.""",
            category=None,
        )
        def run_iteration(
            self, callback: Optional[_AppRunCallbackType] = None, /
        ) -> None:
            """Run this app iteratively without consuming it, calling `callback` on each iteration.

            Args:
                callback: a callback function that will be called on each iteration.

            !!! warning
                `callback` has the same restrictions as [App.run][pytauri.App.run].

            !!! tip
                Approximately 2ms per calling in debug mode.
            """

        def cleanup_before_exit(self, /) -> None:
            """Runs necessary cleanup tasks before exiting the process.

            **You should always exit the tauri app immediately after this function returns and not use any tauri-related APIs.**
            """

        def handle(self, /) -> "AppHandle":
            """Get a handle to this app, which can be used to interact with the app from another thread."""
            ...

    @final
    class AppHandle:
        """[tauri::AppHandle](https://docs.rs/tauri/latest/tauri/app/struct.AppHandle.html)"""

        def run_on_main_thread(self, handler: Callable[[], object], /) -> None:
            """Runs the given closure on the main thread.

            !!! warning
                `handler` has the same restrictions as [App.run][pytauri.App.run].
            """
            ...

        def exit(self, exit_code: int, /) -> None: ...
        def restart(self, /) -> Never: ...
        def on_menu_event(
            self, handler: Callable[["Self", "MenuEvent"], None], /
        ) -> None:
            """Registers a global menu event listener.

            !!! warning
                `handler` has the same restrictions as [App.run][pytauri.App.run].
            """

        def on_tray_icon_event(
            self, handler: Callable[[Self, TrayIconEventType], None], /
        ) -> None:
            """Registers a global tray icon menu event listener.

            !!! warning
                `handler` has the same restrictions as [App.run][pytauri.App.run].
            """

        def tray_by_id(self, id: str, /) -> Optional[TrayIcon]: ...  # noqa: A002
        def remove_tray_by_id(self, id: str, /) -> Optional[TrayIcon]: ...  # noqa: A002
        def set_theme(self, theme: Optional["Theme"], /) -> None: ...
        def default_window_icon(self, /) -> Optional[Image]:
            """Returns the default window icon.

            !!! warning
                Each time you call this function, a new image instance will be created.
                So you should cache the result if you need to use it multiple times.
            """

        def menu(self) -> Optional[Menu]: ...
        def set_menu(self, menu: Menu, /) -> Optional[Menu]: ...
        def remove_menu(self) -> Optional[Menu]: ...
        def hide_menu(self) -> None: ...
        def show_menu(self) -> None: ...
        def invoke_key(self) -> str: ...

    @final
    class BuilderArgs:  # noqa: D101
        def __new__(
            cls,
            /,
            context: "Context",
            *,
            invoke_handler: Optional[_InvokeHandlerProto],
            setup: Optional[Callable[[AppHandle], object]] = None,
        ) -> Self:
            """[tauri::Builder](https://docs.rs/tauri/latest/tauri/struct.Builder.html)

            !!! warning
                The implementer of [invoke_handler][pytauri.ffi.lib.BuilderArgs.__new__(invoke_handler)] must never raise an exception,
                otherwise it is considered logical undefined behavior.
                Additionally, `invoke_handler` must not block.

            !!! warning
                If you do not specify [invoke_handler][pytauri.ffi.lib.BuilderArgs.__new__(invoke_handler)],
                `pytauri` will not register the `tauri-plugin-pytauri` plugin,
                which means you cannot use `pyInvoke` in the frontend to call `Commands`
                (you will receive an error like ["plugin pytauri not found"]).
                If this is indeed the behavior you expect, explicitly pass [None][].

                ["plugin pytauri not found"]: https://github.com/pytauri/pytauri/issues/110

            Args:
                context: use [context_factory][pytauri.context_factory] to get it.
                invoke_handler: use [Commands][pytauri.ipc.Commands] to get it.
                setup: see rust `tauri::Builder::setup`.
            """
            ...

    @final
    class Builder:
        """[Tauri::Builder](https://docs.rs/tauri/latest/tauri/struct.Builder.html)

        use [builder_factory][pytauri.builder_factory] to instantiate this class.

        !!! warning
            This class is not thread-safe, and should not be shared between threads.

            - You can only use it on the thread it was created on.
            - And you need to ensure it is garbage collected on the thread it was created on,
                otherwise it will cause memory leaks.
        """

        def build(self, args: BuilderArgs, /) -> App:
            """Consume this builder and build an app with the given `BuilderArgs`."""
            ...

    @final
    class Context:
        """[tauri::Context](https://docs.rs/tauri/latest/tauri/struct.Context.html)"""

        def set_assets(self, assets: "Assets", /) -> None:
            """Use custom assets instead of the assets bundled by Tauri.

            To make this work:

            - You need to enable the `tauri/custom-protocol` feature.
                - Or build using `tauri build`.
            - Set `frontendDist` in `tauri.conf.json` to an empty directory (do not set it to a URL).
                - Or generate `Context` via:

                    ```rust
                    use tauri::{generate_context, test::noop_assets};

                    let context = generate_context!(assets=noop_assets());
                    ```

                    then we will use this method to set the assets.

                    see: <https://github.com/tauri-apps/tauri/pull/9141>
            """

    @final
    class RunEvent:
        """[tauri::RunEvent](https://docs.rs/tauri/latest/tauri/enum.RunEvent.html)"""

        @final
        class Exit:
            """[tauri::RunEvent::Exit](https://docs.rs/tauri/latest/tauri/enum.RunEvent.html#variant.Exit)"""

        @final
        class ExitRequested:
            """[tauri::RunEvent::ExitRequested](https://docs.rs/tauri/latest/tauri/enum.RunEvent.html#variant.ExitRequested)"""

            code: Optional[int]
            api: "ExitRequestApi"

        @final
        class WindowEvent:
            """[tauri::RunEvent::WindowEvent](https://docs.rs/tauri/latest/tauri/enum.RunEvent.html#variant.WindowEvent)"""

            label: str

        @final
        class WebviewEvent:
            """[tauri::RunEvent::WebviewEvent](https://docs.rs/tauri/latest/tauri/enum.RunEvent.html#variant.WebviewEvent)"""

            label: str
            event: "WebviewEventType"

        @final
        class Ready:
            """[tauri::RunEvent::Ready](https://docs.rs/tauri/latest/tauri/enum.RunEvent.html#variant.Ready)"""

        @final
        class Resumed:
            """[tauri::RunEvent::Resumed](https://docs.rs/tauri/latest/tauri/enum.RunEvent.html#variant.Resumed)"""

        @final
        class MainEventsCleared:
            """[tauri::RunEvent::MainEventsCleared](https://docs.rs/tauri/latest/tauri/enum.RunEvent.html#variant.MainEventsCleared)"""

        @final
        class MenuEvent(NamedTuple):
            """[tauri::RunEvent::MenuEvent](https://docs.rs/tauri/latest/tauri/enum.RunEvent.html#variant.MenuEvent)

            !!! warning
                See [pytauri.ffi.lib.Position.Physical][].
            """

            _0: MenuEvent

        @final
        class TrayIconEvent(NamedTuple):
            """[tauri::RunEvent::TrayIconEvent](https://docs.rs/tauri/latest/tauri/enum.RunEvent.html#variant.TrayIconEvent)

            !!! warning
                See [pytauri.ffi.lib.Position.Physical][].
            """

            _0: TrayIconEventType

        @final
        class _NonExhaustive:
            """Reserved for `#[non_exhaustive]`"""

        # When adding new variants, remember to update `RunEventType`.

    @final
    class ExitRequestApi:
        """[tauri::ExitRequestApi](https://docs.rs/tauri/latest/tauri/struct.ExitRequestApi.html)"""

        def prevent_exit(self, /) -> None: ...

    @final
    class CloseRequestApi:
        """[tauri::CloseRequestApi](https://docs.rs/tauri/latest/tauri/struct.CloseRequestApi.html)"""

        def prevent_close(self, /) -> None: ...

    @final
    class DragDropEvent:
        """[tauri::DragDropEvent](https://docs.rs/tauri/latest/tauri/enum.DragDropEvent.html)"""

        @final
        class Enter:
            """[tauri::DragDropEvent::Enter](https://docs.rs/tauri/latest/tauri/enum.DragDropEvent.html#variant.Enter)"""

            paths: _VecPathBuf
            position: _PhysicalPositionF64

        @final
        class Over:
            """[tauri::DragDropEvent::Over](https://docs.rs/tauri/latest/tauri/enum.DragDropEvent.html#variant.Over)"""

            position: _PhysicalPositionF64

        @final
        class Drop:
            """[tauri::DragDropEvent::Drop](https://docs.rs/tauri/latest/tauri/enum.DragDropEvent.html#variant.Drop)"""

            paths: _VecPathBuf
            position: _PhysicalPositionF64

        @final
        class Leave:
            """[tauri::DragDropEvent::Leave](https://docs.rs/tauri/latest/tauri/enum.DragDropEvent.html#variant.Leave)"""

        @final
        class _NonExhaustive:
            """Reserved for `#[non_exhaustive]`"""

        # When adding new variants, remember to update `DragDropEventType`.

    @final
    class WebviewEvent:
        """[tauri::WebviewEvent](https://docs.rs/tauri/latest/tauri/enum.WebviewEvent.html)"""

        @final
        class DragDrop(NamedTuple):
            """[tauri::WebviewEvent::DragDrop](https://docs.rs/tauri/latest/tauri/enum.WebviewEvent.html#variant.DragDrop)

            !!! warning
                See [pytauri.ffi.lib.Position.Physical][].
            """

            _0: "DragDropEventType"

        @final
        class _NonExhaustive:
            """Reserved for `#[non_exhaustive]`"""

        # When adding new variants, remember to update `WebviewEventType`.

    def builder_factory(*args: Any, **kwargs: Any) -> Builder:
        """A factory function for creating a `Builder` instance.

        This is the closure passed from the Rust side when initializing the pytauri pyo3 module.
        `args` and `kwargs` will be passed to this closure.
        """
        ...

    def context_factory(*args: Any, **kwargs: Any) -> Context:
        """A factory function for creating a `Context` instance.

        This is the closure passed from the Rust side when initializing the pytauri pyo3 module.
        `args` and `kwargs` will be passed to this closure.
        """
        ...

    @final
    class Manager:
        """[tauri::Manager](https://docs.rs/tauri/latest/tauri/trait.Manager.html)"""

        @staticmethod
        def app_handle(slf: "ImplManager", /) -> AppHandle:
            """The application handle associated with this manager."""
            ...

        @staticmethod
        def get_webview_window(
            slf: "ImplManager", label: str, /
        ) -> Optional[WebviewWindow]:
            """Fetch a single webview window from the manager."""
            ...

        @staticmethod
        def webview_windows(slf: "ImplManager", /) -> dict[str, WebviewWindow]:
            """Fetch all managed webview windows."""
            ...

        @staticmethod
        def path(slf: "ImplManager", /) -> PathResolver:
            """The path resolver is a helper class for general and application-specific path APIs."""
            ...

    @final
    class Event:
        """[tauri::Event](https://docs.rs/tauri/latest/tauri/struct.Event.html)"""

        @property
        def id(self) -> "EventId":
            """The `EventId` of the handler that was triggered."""
            ...

        @property
        def payload(self) -> str:
            """The event payload."""
            ...

    @final
    class Listener:
        """[tauri::Listener](https://docs.rs/tauri/latest/tauri/trait.Listener.html)

        See also: <https://tauri.app/develop/calling-rust/#event-system>

        # Examples

        ```python
        from pydantic import BaseModel
        from pytauri import AppHandle, Event, Listener


        class Payload(BaseModel):  # or `RootModel`
            url: str
            num: int


        def listen(app_handle: AppHandle) -> None:
            def handler(event: Event):
                assert event.id == event_id

                serialized_event = Payload.model_validate_json(event.payload)
                print(serialized_event.url, serialized_event.num)

            event_id = Listener.listen(app_handle, "event_name", handler)
        ```
        """

        @staticmethod
        def listen(
            slf: "ImplListener",
            event: str,
            handler: _EventHandlerType,
            /,
        ) -> "EventId":
            """Listen to an emitted event on this manager.

            !!! warning
                `handler` has the same restrictions as [App.run][pytauri.App.run].
            """
            ...

        @staticmethod
        def once(
            slf: "ImplListener",
            event: str,
            handler: _EventHandlerType,
            /,
        ) -> "EventId":
            """Listen to an event on this manager only once.

            !!! warning
                `handler` has the same restrictions as [App.run][pytauri.App.run].
            """
            ...

        @staticmethod
        def unlisten(
            slf: "ImplListener",
            id: "EventId",  # noqa: A002
            /,
        ) -> None:
            """Remove an event listener."""
            ...

        @staticmethod
        def listen_any(
            slf: "ImplListener",
            event: str,
            handler: _EventHandlerType,
            /,
        ) -> "EventId":
            """Listen to an emitted event to any target.

            !!! warning
                `handler` has the same restrictions as [App.run][pytauri.App.run].
            """
            ...

        @staticmethod
        def once_any(
            slf: "ImplListener",
            event: str,
            handler: _EventHandlerType,
            /,
        ) -> "EventId":
            """Listens once to an emitted event to any target .

            !!! warning
                `handler` has the same restrictions as [App.run][pytauri.App.run].
            """
            ...

    @final
    class Position:
        """[tauri::Position](https://docs.rs/tauri/latest/tauri/enum.Position.html)"""

        @final
        class Physical(NamedTuple):
            """[tauri::Position::Physical](https://docs.rs/tauri/latest/tauri/enum.Position.html#variant.Physical)

            !!! warning
                This is actually a `Class` disguised as an `NamedTuple`.
                See also: <https://pyo3.rs/v0.23.4/class.html#pyclass-enums>.
            """

            _0: _PhysicalPositionI32

        @final
        class Logical(NamedTuple):
            """[tauri::Position::Logical](https://docs.rs/tauri/latest/tauri/enum.Position.html#variant.Logical)

            !!! warning
                See [pytauri.ffi.lib.Position.Physical][].
            """

            _0: _LogicalPositionF64

    @final
    class Size:
        """[tauri::Size](https://docs.rs/tauri/latest/tauri/enum.Size.html)"""

        @final
        class Physical(NamedTuple):
            """[tauri::Size::Physical](https://docs.rs/tauri/latest/tauri/enum.Size.html#variant.Physical)

            !!! warning
                See [pytauri.ffi.lib.Position.Physical][].
            """

            _0: _PhysicalSizeU32

        @final
        class Logical(NamedTuple):
            """[tauri::Size::Logical](https://docs.rs/tauri/latest/tauri/enum.Size.html#variant.Logical)

            !!! warning
                See [pytauri.ffi.lib.Position.Physical][].
            """

            _0: _LogicalSizeF64

    @final
    class Rect:
        """[tauri::Rect](https://docs.rs/tauri/latest/tauri/struct.Rect.html)"""

        def __new__(
            cls,
            /,
            *,
            position: "PositionType",
            size: "SizeType",
        ) -> Self: ...

        @property
        def position(self) -> "PositionType": ...
        @property
        def size(self) -> "SizeType": ...

    @final
    class EventTarget:
        """[tauri::EventTarget](https://docs.rs/tauri/latest/tauri/enum.EventTarget.html)"""

        @final
        class Any:
            """Any and all event targets."""

            def __new__(cls, /) -> Self: ...

        @final
        class AnyLabel:
            """Any `Window`, `Webview` or `WebviewWindow` that have this label."""

            label: str
            """Target label."""

            def __new__(cls, label: str, /) -> Self: ...

        @final
        class App:
            """App and AppHandle targets."""

            def __new__(cls, /) -> Self: ...

        @final
        class Window:
            """`Window` target."""

            label: str
            """window label."""

            def __new__(cls, label: str, /) -> Self: ...

        @final
        class Webview:
            """Webview target."""

            label: str
            """webview label."""

            def __new__(cls, label: str, /) -> Self: ...

        @final
        class WebviewWindow:
            """WebviewWindow target."""

            label: str
            """webview window label."""

            def __new__(cls, label: str, /) -> Self: ...

        @final
        class _NonExhaustive:
            """Reserved for `#[non_exhaustive]`"""

        # When adding new variants, remember to update `EventTargetType`.

    class Emitter:
        """[tauri::Emitter](https://docs.rs/tauri/latest/tauri/trait.Emitter.html)"""

        @staticmethod
        def emit_str(
            slf: "ImplEmitter",
            event: str,
            payload: str,
            /,
        ) -> None:
            """Similar to [`Emitter::emit`] but the payload is json serialized."""
            ...

        @staticmethod
        def emit_str_to(
            slf: "ImplEmitter",
            target: "EventTargetType",
            event: str,
            payload: str,
            /,
        ) -> None:
            """Similar to [`Emitter::emit_to`] but the payload is json serialized."""
            ...

        @staticmethod
        def emit_str_filter(
            slf: "ImplEmitter",
            event: str,
            payload: str,
            filter: Callable[["EventTargetType"], bool],  # noqa: A002
            /,
        ) -> None:
            """Similar to [`Emitter::emit_filter`] but the payload is json serialized.

            !!! warning
                `filter` has the same restrictions as [App.run][pytauri.App.run].
            """
            ...

    @final
    class Theme(Enum):
        """[tauri::Theme](https://docs.rs/tauri/latest/tauri/enum.Theme.html)

        !!! warning
            See [pytauri.ffi.menu.NativeIcon][].
        """

        Light = auto()
        Dark = auto()
        _NonExhaustive = object()


else:
    App = pytauri_mod.App
    AppHandle = pytauri_mod.AppHandle
    Builder = pytauri_mod.Builder
    BuilderArgs = pytauri_mod.BuilderArgs
    Context = pytauri_mod.Context
    RunEvent = pytauri_mod.RunEvent
    ExitRequestApi = pytauri_mod.ExitRequestApi
    CloseRequestApi = pytauri_mod.CloseRequestApi
    DragDropEvent = pytauri_mod.DragDropEvent
    WebviewEvent = pytauri_mod.WebviewEvent
    builder_factory = pytauri_mod.builder_factory
    context_factory = pytauri_mod.context_factory
    Manager = pytauri_mod.Manager
    Event = pytauri_mod.Event
    Listener = pytauri_mod.Listener
    Position = pytauri_mod.Position
    Size = pytauri_mod.Size
    Rect = pytauri_mod.Rect
    EventTarget = pytauri_mod.EventTarget
    Emitter = pytauri_mod.Emitter
    Theme = pytauri_mod.Theme

RunEventType = TypeAliasType(
    "RunEventType",
    Union[
        RunEvent.Exit,
        RunEvent.ExitRequested,
        RunEvent.WindowEvent,
        RunEvent.WebviewEvent,
        RunEvent.Ready,
        RunEvent.Resumed,
        RunEvent.MainEventsCleared,
        RunEvent.MenuEvent,
        RunEvent.TrayIconEvent,
        RunEvent._NonExhaustive,  # pyright: ignore[reportPrivateUsage]
    ],
)
"""See [RunEvent][pytauri.ffi.RunEvent] for details."""

DragDropEventType = TypeAliasType(
    "DragDropEventType",
    Union[
        DragDropEvent.Enter,
        DragDropEvent.Over,
        DragDropEvent.Drop,
        DragDropEvent.Leave,
        DragDropEvent._NonExhaustive,  # pyright: ignore[reportPrivateUsage]
    ],
)
"""See [DragDropEvent][pytauri.ffi.DragDropEvent] for details."""

WebviewEventType = TypeAliasType(
    "WebviewEventType",
    Union[
        WebviewEvent.DragDrop,
        WebviewEvent._NonExhaustive,  # pyright: ignore[reportPrivateUsage]
    ],
)
"""See [WebviewEvent][pytauri.ffi.WebviewEvent] for details."""

ImplManager = TypeAliasType("ImplManager", Union[App, AppHandle, "WebviewWindow"])

EventId = NewType("EventId", int)
"""[tauri::EventId](https://docs.rs/tauri/latest/tauri/type.EventId.html)"""

ImplListener = ImplManager

PositionType = TypeAliasType("PositionType", Union[Position.Physical, Position.Logical])
"""See [Position][pytauri.ffi.Position] for details."""

SizeType = TypeAliasType("SizeType", Union[Size.Physical, Size.Logical])
"""See [Size][pytauri.ffi.Size] for details."""


class Assets(ABC):
    """[tauri::Assets](https://docs.rs/tauri/latest/tauri/trait.Assets.html)

    This is an abstract class that you can subclass to implement a custom asset loader.

    See `tauri::Assets` rust docs for more details.

    !!! warning
        The implement has the same restrictions as [App.run][pytauri.App.run].
    """

    @abstractmethod
    def get(self, key: _AssetKey, /) -> Optional[bytes]: ...
    @abstractmethod
    def iter(self, /) -> Iterator[tuple[str, bytes]]: ...

    # TODO: `def csp_hashes`
    # blocked by: <https://github.com/tauri-apps/tauri/issues/12756>

    def setup(self, _app: AppHandle, /) -> object:
        return None


Url = TypeAliasType("Url", str)
"""[tauri::Url](https://docs.rs/tauri/latest/tauri/struct.Url.html#method.parse)"""

ImplEmitter = ImplManager

EventTargetType = TypeAliasType(
    "EventTargetType",
    Union[
        EventTarget.Any,
        EventTarget.AnyLabel,
        EventTarget.App,
        EventTarget.Window,
        EventTarget.Webview,
        EventTarget.WebviewWindow,
        EventTarget._NonExhaustive,  # pyright: ignore[reportPrivateUsage]
    ],
)
"""See [EventTarget][pytauri.ffi.EventTarget] for details."""
