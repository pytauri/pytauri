"""Original FFI interface module.

!!! warning
    All APIs under this module should not be considered stable.
    You should use the re-exported APIs under the top-level module.
"""

from types import ModuleType
from typing import TYPE_CHECKING

from pytauri.plugin import Plugin

from pytauri_plugins import (
    PLUGIN_POSITIONER,
    _pytauri_plugins_mod,  # pyright: ignore[reportPrivateUsage]
)

__all__ = ["init"]

if PLUGIN_POSITIONER:
    _positioner_mod: ModuleType = _pytauri_plugins_mod.positioner
else:
    raise ImportError(
        "Enable the `plugin-positioner` feature for `pytauri` crate to use this plugin."
    )


if TYPE_CHECKING:

    def init() -> Plugin:
        """[tauri_plugin_positioner::init](https://docs.rs/tauri-plugin-positioner/latest/tauri_plugin_positioner/fn.init.html)"""
        ...

else:
    init = _positioner_mod.init
