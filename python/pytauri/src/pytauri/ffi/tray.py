# ruff: noqa: D102

"""[tauri::tray](https://docs.rs/tauri/latest/tauri/tray/index.html)"""

from enum import Enum, auto
from os import PathLike
from pathlib import Path
from typing import (
    TYPE_CHECKING,
    Callable,
    Optional,
    Union,
    final,
)

from typing_extensions import Self, TypeAliasType

from pytauri.ffi._ext_mod import pytauri_mod

__all__ = [
    "MouseButton",
    "MouseButtonState",
    "TrayIcon",
    "TrayIconEvent",
    "TrayIconEventType",
    "TrayIconId",
]

_ToPyo3Path = Union[str, PathLike[str], Path]

_tray_mod = pytauri_mod.tray

TrayIconId = TypeAliasType("TrayIconId", str)
"""[tauri::tray::TrayIconId](https://docs.rs/tauri/latest/tauri/tray/struct.TrayIconId.html)"""

_PyPhysicalPositionF64 = tuple[float, float]

if TYPE_CHECKING:
    from pytauri.ffi.image import Image
    from pytauri.ffi.lib import AppHandle, ImplManager, Rect
    from pytauri.ffi.menu import ImplContextMenu, MenuEvent

    @final
    class TrayIcon:
        """[tauri::tray::TrayIcon](https://docs.rs/tauri/latest/tauri/tray/struct.TrayIcon.html)"""

        def __new__(cls, manager: ImplManager, /) -> Self: ...
        @staticmethod
        def with_id(manager: ImplManager, id: TrayIconId, /) -> "TrayIcon": ...  # noqa: A002
        def app_handle(self, /) -> AppHandle: ...
        def on_menu_event(
            self, handler: Callable[[AppHandle, MenuEvent], None], /
        ) -> None:
            """This is an alias for [pytauri.ffi.AppHandle.on_menu_event][]."""
            ...

        def on_tray_icon_event(
            self, handler: Callable[[Self, "TrayIconEventType"], None], /
        ) -> None:
            """Set a handler for this tray icon events.

            !!! warning
                `handler` has the same restrictions as [App.run][pytauri.App.run].
            """
            ...

        def id(self, /) -> TrayIconId: ...
        def set_icon(self, icon: Optional[Image], /) -> None: ...
        def set_menu(self, menu: Optional[ImplContextMenu], /) -> None: ...
        def set_tooltip(self, tooltip: Optional[str], /) -> None: ...
        def set_title(self, title: Optional[str], /) -> None: ...
        def set_visible(self, visible: bool, /) -> None: ...
        def set_temp_dir_path(self, path: Optional[_ToPyo3Path], /) -> None: ...
        def set_icon_as_template(self, is_template: bool, /) -> None: ...
        def set_show_menu_on_left_click(self, enable: bool, /) -> None: ...
        def rect(self, /) -> Optional[Rect]: ...

    @final
    class TrayIconEvent:
        """[tauri::tray::TrayIconEvent](https://docs.rs/tauri/latest/tauri/tray/enum.TrayIconEvent.html)"""

        @final
        class Click:
            """[tauri::tray::TrayIconEvent::Click](https://docs.rs/tauri/latest/tauri/tray/enum.TrayIconEvent.html#variant.Click)"""

            @property
            def id(self, /) -> TrayIconId: ...
            @property
            def position(self, /) -> _PyPhysicalPositionF64: ...
            @property
            def rect(self, /) -> Rect: ...
            @property
            def button(self, /) -> "MouseButton": ...
            @property
            def button_state(self, /) -> "MouseButtonState": ...

        @final
        class DoubleClick:
            """[tauri::tray::TrayIconEvent::DoubleClick](https://docs.rs/tauri/latest/tauri/tray/enum.TrayIconEvent.html#variant.DoubleClick)"""

            @property
            def id(self, /) -> TrayIconId: ...
            @property
            def position(self, /) -> _PyPhysicalPositionF64: ...
            @property
            def rect(self, /) -> Rect: ...
            @property
            def button(self, /) -> "MouseButton": ...

        @final
        class Enter:
            """[tauri::tray::TrayIconEvent::Enter](https://docs.rs/tauri/latest/tauri/tray/enum.TrayIconEvent.html#variant.Enter)"""

            @property
            def id(self, /) -> TrayIconId: ...
            @property
            def position(self, /) -> _PyPhysicalPositionF64: ...
            @property
            def rect(self, /) -> Rect: ...

        @final
        class Move:
            """[tauri::tray::TrayIconEvent::Move](https://docs.rs/tauri/latest/tauri/tray/enum.TrayIconEvent.html#variant.Move)"""

            @property
            def id(self, /) -> TrayIconId: ...
            @property
            def position(self, /) -> _PyPhysicalPositionF64: ...
            @property
            def rect(self, /) -> Rect: ...

        @final
        class Leave:
            """[tauri::tray::TrayIconEvent::Leave](https://docs.rs/tauri/latest/tauri/tray/enum.TrayIconEvent.html#variant.Leave)"""

            @property
            def id(self, /) -> TrayIconId: ...
            @property
            def position(self, /) -> _PyPhysicalPositionF64: ...
            @property
            def rect(self, /) -> Rect: ...

        # When adding new variants, remember to update `TrayIconEventType`.

    @final
    class MouseButton(Enum):
        """[tauri::tray::MouseButton](https://docs.rs/tauri/latest/tauri/tray/enum.MouseButton.html)

        !!! warning
            See [pytauri.ffi.menu.NativeIcon][].
        """

        Left = auto()
        Right = auto()
        Middle = auto()

    @final
    class MouseButtonState(Enum):
        """[tauri::tray::MouseButtonState](https://docs.rs/tauri/latest/tauri/tray/enum.MouseButtonState.html)

        !!! warning
            See [pytauri.ffi.menu.NativeIcon][].
        """

        Up = auto()
        Down = auto()


else:
    TrayIcon = _tray_mod.TrayIcon
    TrayIconEvent = _tray_mod.TrayIconEvent
    MouseButton = _tray_mod.MouseButton
    MouseButtonState = _tray_mod.MouseButtonState

TrayIconEventType = TypeAliasType(
    "TrayIconEventType",
    Union[
        TrayIconEvent.Click,
        TrayIconEvent.DoubleClick,
        TrayIconEvent.Enter,
        TrayIconEvent.Move,
        TrayIconEvent.Leave,
    ],
)
"""See [TrayIconEvent][pytauri.ffi.tray.TrayIconEvent] for details."""
