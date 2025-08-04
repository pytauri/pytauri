"""Original FFI interface module.

!!! warning
    All APIs under this module should not be considered stable.
    You should use the re-exported APIs under the top-level module.
"""

from types import ModuleType
from typing import TYPE_CHECKING

from pytauri.plugin import Plugin

from pytauri_plugins import (
    PLUGIN_PERSISTED_SCOPE,
    _pytauri_plugins_mod,  # pyright: ignore[reportPrivateUsage]
)

__all__ = ["init"]

if PLUGIN_PERSISTED_SCOPE:
    _persisted_scope_mod: ModuleType = _pytauri_plugins_mod.persisted_scope
else:
    raise ImportError(
        "Enable the `plugin-persisted-scope` feature for `pytauri` crate to use this plugin."
    )


if TYPE_CHECKING:

    def init() -> Plugin:
        """[tauri_plugin_persisted_scope::init](https://docs.rs/tauri-plugin-persisted-scope/latest/tauri_plugin_persisted_scope/fn.init.html)"""
        ...

else:
    init = _persisted_scope_mod.init
