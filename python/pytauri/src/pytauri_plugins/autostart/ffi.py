"""Original FFI interface module.

!!! warning
    All APIs under this module should not be considered stable.
    You should use the re-exported APIs under the top-level module.
"""

from collections.abc import Sequence
from enum import Enum, auto
from types import ModuleType
from typing import TYPE_CHECKING, Optional, final

from pytauri.plugin import Plugin

from pytauri_plugins import (
    PLUGIN_AUTOSTART,
    _pytauri_plugins_mod,  # pyright: ignore[reportPrivateUsage]
)

__all__ = ["MacosLauncher", "init"]

if PLUGIN_AUTOSTART:
    _autostart_mod: ModuleType = _pytauri_plugins_mod.autostart
else:
    raise ImportError(
        "Enable the `plugin-autostart` feature for `pytauri` crate to use this plugin."
    )


if TYPE_CHECKING:

    @final
    class MacosLauncher(Enum):
        """[tauri_plugin_autostart::MacosLauncher](https://docs.rs/tauri-plugin-autostart/latest/tauri_plugin_autostart/enum.MacosLauncher.html)"""

        LaunchAgent = auto()
        AppleScript = auto()

    def init(
        macos_launcher: MacosLauncher = MacosLauncher.LaunchAgent,
        args: Optional[Sequence[str]] = None,
    ) -> Plugin:
        """[tauri_plugin_autostart::init](https://docs.rs/tauri-plugin-autostart/latest/tauri_plugin_autostart/fn.init.html)"""
        ...

else:
    MacosLauncher = _autostart_mod.MacosLauncher
    init = _autostart_mod.init
