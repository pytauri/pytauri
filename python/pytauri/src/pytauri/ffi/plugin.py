"""[tauri::plugin](https://docs.rs/tauri/latest/tauri/plugin/index.html)"""

from typing import (
    TYPE_CHECKING,
    final,
)

from pytauri.ffi._ext_mod import pytauri_mod

__all__ = ["Plugin"]

_plugin_mod = pytauri_mod.plugin

if TYPE_CHECKING:

    @final
    class Plugin:
        """[tauri::plugin::Plugin](https://docs.rs/tauri/latest/tauri/plugin/trait.Plugin.html)"""

else:
    Plugin = _plugin_mod.Plugin
