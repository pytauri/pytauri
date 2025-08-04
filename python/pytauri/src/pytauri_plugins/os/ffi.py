"""Original FFI interface module.

!!! warning
    All APIs under this module should not be considered stable.
    You should use the re-exported APIs under the top-level module.
"""

from types import ModuleType
from typing import TYPE_CHECKING

from pytauri.plugin import Plugin

from pytauri_plugins import (
    PLUGIN_OS,
    _pytauri_plugins_mod,  # pyright: ignore[reportPrivateUsage]
)

__all__ = ["init"]

if PLUGIN_OS:
    _os_mod: ModuleType = _pytauri_plugins_mod.os
else:
    raise ImportError(
        "Enable the `plugin-os` feature for `pytauri` crate to use this plugin."
    )


if TYPE_CHECKING:

    def init() -> Plugin:
        """[tauri_plugin_os::init](https://docs.rs/tauri-plugin-os/latest/tauri_plugin_os/fn.init.html)"""
        ...

else:
    init = _os_mod.init
