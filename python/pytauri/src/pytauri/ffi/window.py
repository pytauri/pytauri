# ruff: noqa: D102

"""[tauri::window](https://docs.rs/tauri/latest/tauri/window/index.html)"""

from collections.abc import Sequence
from enum import Enum, auto
from typing import (
    TYPE_CHECKING,
    Optional,
    final,
)

from typing_extensions import NotRequired, TypedDict

from pytauri.ffi._ext_mod import pytauri_mod

__all__ = [
    "Effect",
    "EffectState",
    "Effects",
    "Monitor",
    "ProgressBarState",
    "ProgressBarStatus",
    "TitleBarStyle",
    "Window",
]

_window_mod = pytauri_mod.window

if TYPE_CHECKING:
    from pytauri.ffi.lib import (
        PhysicalRect,
        _PhysicalPositionI32,  # pyright: ignore[reportPrivateUsage]
        _PhysicalSizeU32,  # pyright: ignore[reportPrivateUsage]
    )
    from pytauri.ffi.webview import Color

    @final
    class Window:
        """[tauri::window::Window](https://docs.rs/tauri/latest/tauri/window/struct.Window.html)"""

    @final
    class Monitor:
        """[tauri::window::Monitor](https://docs.rs/tauri/latest/tauri/window/struct.Monitor.html)"""

        @property
        def name(self) -> Optional[str]: ...
        @property
        def size(self) -> _PhysicalSizeU32: ...
        @property
        def position(self) -> _PhysicalPositionI32: ...
        @property
        def work_area(self) -> PhysicalRect: ...
        @property
        def scale_factor(self) -> float: ...

    @final
    class Effect(Enum):
        """[tauri::window::Effect](https://docs.rs/tauri/latest/tauri/window/enum.Effect.html)"""

        AppearanceBased = auto()
        Light = auto()
        Dark = auto()
        MediumLight = auto()
        UltraDark = auto()
        Titlebar = auto()
        Selection = auto()
        Menu = auto()
        Popover = auto()
        Sidebar = auto()
        HeaderView = auto()
        Sheet = auto()
        WindowBackground = auto()
        HudWindow = auto()
        FullScreenUI = auto()
        Tooltip = auto()
        ContentBackground = auto()
        UnderWindowBackground = auto()
        UnderPageBackground = auto()
        Mica = auto()
        MicaDark = auto()
        MicaLight = auto()
        Tabbed = auto()
        TabbedDark = auto()
        TabbedLight = auto()
        Blur = auto()
        Acrylic = auto()

    class EffectState(Enum):
        """[tauri::window::EffectState](https://docs.rs/tauri/latest/tauri/window/enum.EffectState.html)"""

        FollowsWindowActiveState = auto()
        Active = auto()
        Inactive = auto()

    class ProgressBarStatus(Enum):
        """[tauri::window::ProgressBarStatus](https://docs.rs/tauri/latest/tauri/window/enum.ProgressBarStatus.html)"""

        None_ = auto()
        Normal = auto()
        Indeterminate = auto()
        Paused = auto()
        Error = auto()

    class TitleBarStyle(Enum):
        """[tauri::TitleBarStyle](https://docs.rs/tauri/latest/tauri/enum.TitleBarStyle.html)"""

        Visible = auto()
        Transparent = auto()
        Overlay = auto()
        _NonExhaustive = object()

else:
    Window = _window_mod.Window
    Monitor = _window_mod.Monitor
    Effect = _window_mod.Effect
    EffectState = _window_mod.EffectState
    ProgressBarStatus = _window_mod.ProgressBarStatus
    TitleBarStyle = _window_mod.TitleBarStyle


class Effects(TypedDict, closed=True):
    """[tauri::window::EffectsBuilder](https://docs.rs/tauri/latest/tauri/window/struct.EffectsBuilder.html)"""

    effects: NotRequired[Sequence[Effect]]
    state: NotRequired[EffectState]
    radius: NotRequired[float]
    color: NotRequired["Color"]


class ProgressBarState(TypedDict, closed=True):
    """[tauri::window::ProgressBarState](https://docs.rs/tauri/latest/tauri/window/struct.ProgressBarState.html)"""

    status: NotRequired[ProgressBarStatus]
    progress: NotRequired[int]
