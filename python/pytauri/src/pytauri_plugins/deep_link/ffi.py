"""Original FFI interface module.

!!! warning
    All APIs under this module should not be considered stable.
    You should use the re-exported APIs under the top-level module.
"""

from types import ModuleType
from typing import TYPE_CHECKING

from pytauri.plugin import Plugin

from pytauri_plugins import (
    PLUGIN_DEEP_LINK,
    _pytauri_plugins_mod,  # pyright: ignore[reportPrivateUsage]
)

__all__ = [
    "init",
]

if PLUGIN_DEEP_LINK:
    _deep_link_mod: ModuleType = _pytauri_plugins_mod.deep_link
else:
    raise ImportError(
        "Enable the `plugin-deep-link` feature for `pytauri` crate to use this plugin."
    )


if TYPE_CHECKING:

    def init() -> Plugin:
        """[tauri_plugin_deep_link::init](https://docs.rs/tauri-plugin-deep-link/latest/tauri_plugin_deep_link/fn.init.html)"""
        ...

else:
    init = _deep_link_mod.init
