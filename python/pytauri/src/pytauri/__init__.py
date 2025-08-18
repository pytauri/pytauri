"""[tauri::self](https://docs.rs/tauri/latest/tauri/index.html)"""

from typing import Callable, final

from pydantic import BaseModel

from pytauri.ffi import (
    EXT_MOD,
    IS_DEV,
    RESTART_EXIT_CODE,
    VERSION,
    App,
    AppHandle,
    Assets,
    Builder,
    BuilderArgs,
    CloseRequestApi,
    Context,
    CursorIcon,
    DragDropEvent,
    DragDropEventType,
    Event,
    EventId,
    EventTarget,
    EventTargetType,
    ExitRequestApi,
    ImplEmitter,
    ImplListener,
    ImplManager,
    Listener,
    LogicalRect,
    Manager,
    PhysicalRect,
    Position,
    PositionType,
    Rect,
    RunEvent,
    RunEventType,
    Size,
    SizeType,
    Theme,
    Url,
    UserAttentionType,
    WebviewEvent,
    WebviewEventType,
    WindowEvent,
    WindowEventType,
    builder_factory,
    context_factory,
    webview_version,
)
from pytauri.ffi import (
    Emitter as _Emitter,
)
from pytauri.ipc import Commands, State

__all__ = [
    "EXT_MOD",
    "IS_DEV",
    "RESTART_EXIT_CODE",
    "VERSION",
    "App",
    "AppHandle",
    "Assets",
    "Builder",
    "BuilderArgs",
    "CloseRequestApi",
    "Commands",
    "Context",
    "CursorIcon",
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
    "LogicalRect",
    "Manager",
    "PhysicalRect",
    "Position",
    "PositionType",
    "Rect",
    "RunEvent",
    "RunEventType",
    "Size",
    "SizeType",
    "State",
    "Theme",
    "Url",
    "UserAttentionType",
    "WebviewEvent",
    "WebviewEventType",
    "WindowEvent",
    "WindowEventType",
    "builder_factory",
    "context_factory",
    "webview_version",
]


@final
class Emitter(_Emitter):
    """[tauri::Emitter](https://docs.rs/tauri/latest/tauri/trait.Emitter.html)

    See also: <https://tauri.app/develop/calling-frontend/#event-system>

    # Examples

    ```python
    from pydantic import BaseModel
    from pytauri import AppHandle, Emitter


    class Payload(BaseModel):  # or `RootModel`
        url: str
        num: int


    def emit(app_handle: AppHandle) -> None:
        Emitter.emit(
            app_handle, "event_name", Payload(url="https://example.com", num=42)
        )
    ```
    """

    # `classmethod` instead of `staticmethod`, see: <https://github.com/python/cpython/issues/75301#issuecomment-1093755348>

    @classmethod
    def emit(cls, slf: ImplEmitter, event: str, payload: BaseModel, /) -> None:
        """Emits an event to all `targets`."""
        super().emit_str(slf, event, payload.model_dump_json())

    @classmethod
    def emit_to(
        cls,
        slf: ImplEmitter,
        target: EventTargetType,
        event: str,
        payload: BaseModel,
        /,
    ) -> None:
        """Emits an event to all `targets` matching the given target."""
        super().emit_str_to(slf, target, event, payload.model_dump_json())

    @classmethod
    def emit_filter(
        cls,
        slf: ImplEmitter,
        event: str,
        payload: BaseModel,
        filter: Callable[["EventTargetType"], bool],  # noqa: A002
        /,
    ) -> None:
        """Emits an event to all `targets` based on the given filter.

        !!! warning
            `filter` has the same restrictions as [App.run][pytauri.App.run].
        """
        super().emit_str_filter(slf, event, payload.model_dump_json(), filter)
