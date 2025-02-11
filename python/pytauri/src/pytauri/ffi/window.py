# ruff: noqa: D102

"""[tauri::window](https://docs.rs/tauri/latest/tauri/window/index.html)"""

from typing import (
    TYPE_CHECKING,
    final,
)

from pytauri.ffi._ext_mod import pytauri_mod

__all__ = [
    "Window",
]

_window_mod = pytauri_mod.window

if TYPE_CHECKING:

    @final
    class Window:
        """[tauri::window::Window](https://docs.rs/tauri/latest/tauri/window/struct.Window.html)"""

else:
    Window = _window_mod.Window
