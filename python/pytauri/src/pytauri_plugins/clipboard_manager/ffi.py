"""Original FFI interface module.

!!! warning
    All APIs under this module should not be considered stable.
    You should use the re-exported APIs under the top-level module.
"""

from types import ModuleType
from typing import TYPE_CHECKING

from pytauri.plugin import Plugin

from pytauri_plugins import (
    PLUGIN_CLIPBOARD_MANAGER,
    _pytauri_plugins_mod,  # pyright: ignore[reportPrivateUsage]
)

__all__ = [
    "init",
]

if PLUGIN_CLIPBOARD_MANAGER:
    _clipboard_manager_mod: ModuleType = _pytauri_plugins_mod.clipboard_manager
else:
    raise ImportError(
        "Enable the `plugin-clipboard-manager` feature for `pytauri` crate to use this plugin."
    )


if TYPE_CHECKING:

    def init() -> Plugin:
        """[tauri_plugin_clipboard_manager::init](https://docs.rs/tauri-plugin-clipboard-manager/latest/tauri_plugin_clipboard_manager/fn.init.html)"""
        ...

else:
    init = _clipboard_manager_mod.init
