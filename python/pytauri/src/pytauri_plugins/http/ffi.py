"""Original FFI interface module.

!!! warning
    All APIs under this module should not be considered stable.
    You should use the re-exported APIs under the top-level module.
"""

from types import ModuleType
from typing import TYPE_CHECKING

from pytauri.plugin import Plugin

from pytauri_plugins import (
    PLUGIN_HTTP,
    _pytauri_plugins_mod,  # pyright: ignore[reportPrivateUsage]
)

__all__ = [
    "init",
]

if PLUGIN_HTTP:
    _http_mod: ModuleType = _pytauri_plugins_mod.http
else:
    raise ImportError(
        "Enable the `plugin-http` feature for `pytauri` crate to use this plugin."
    )


if TYPE_CHECKING:

    def init() -> Plugin:
        """[tauri_plugin_http::init](https://docs.rs/tauri-plugin-http/latest/tauri_plugin_http/fn.init.html)"""
        ...

else:
    init = _http_mod.init
