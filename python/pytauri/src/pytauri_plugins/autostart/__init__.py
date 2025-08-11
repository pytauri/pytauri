"""[tauri_plugin_autostart::self](https://docs.rs/tauri-plugin-autostart/latest/tauri_plugin_autostart/index.html)"""

from pytauri_plugins.autostart.ffi import (
    MacosLauncher,
    init,
)

__all__ = [
    "MacosLauncher",
    "init",
]
