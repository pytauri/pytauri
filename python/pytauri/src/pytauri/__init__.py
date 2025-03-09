"""[tauri::self](https://docs.rs/tauri/latest/tauri/index.html)"""

from typing import Callable, final

from pydantic import BaseModel

from pytauri.ffi import (
    EXT_MOD,
    App,
    AppHandle,
    Assets,
    Builder,
    BuilderArgs,
    Context,
    Event,
    EventId,
    EventTarget,
    EventTargetType,
    ImplEmitter,
    ImplListener,
    ImplManager,
    Listener,
    Manager,
    Position,
    PositionType,
    Rect,
    RunEvent,
    RunEventType,
    Size,
    SizeType,
    Url,
    builder_factory,
    context_factory,
)
from pytauri.ffi import (
    Emitter as _Emitter,
)
from pytauri.ipc import Commands

__all__ = [
    "EXT_MOD",
    "App",
    "AppHandle",
    "Assets",
    "Builder",
    "BuilderArgs",
    "Commands",
    "Context",
    "Emitter",
    "Event",
    "EventId",
    "EventTarget",
    "EventTargetType",
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
    "Url",
    "builder_factory",
    "context_factory",
]


@final
class Emitter(_Emitter):
    """[tauri::Emitter](https://docs.rs/tauri/latest/tauri/trait.Emitter.html)

    See also: <https://tauri.app/develop/calling-frontend/#event-system>

    # Example

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
