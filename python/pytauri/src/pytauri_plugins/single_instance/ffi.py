"""Original FFI interface module.

!!! warning
    All APIs under this module should not be considered stable.
    You should use the re-exported APIs under the top-level module.
"""

from types import ModuleType
from typing import TYPE_CHECKING, Callable, Optional

from pytauri import AppHandle
from pytauri.plugin import Plugin

from pytauri_plugins import (
    PLUGIN_SINGLE_INSTANCE,
    _pytauri_plugins_mod,  # pyright: ignore[reportPrivateUsage]
)

__all__ = [
    "init",
]

if PLUGIN_SINGLE_INSTANCE:
    _single_instance_mod: ModuleType = _pytauri_plugins_mod.single_instance
else:
    raise ImportError(
        "Enable the `plugin-single-instance` feature for `pytauri` crate to use this plugin."
    )


if TYPE_CHECKING:

    def init(
        callback: Optional[Callable[[AppHandle, list[str], str], None]], /
    ) -> Plugin:
        """[tauri_plugin_single_instance::init](https://docs.rs/tauri-plugin-single-instance/latest/tauri_plugin_single_instance/fn.init.html)

        Args:
            callback: `(app_handle, args, cwd)`
        """
        ...

else:
    init = _single_instance_mod.init
