# ruff: noqa: D102

"""Original FFI interface module.

!!! warning
    All APIs under this module should not be considered stable.
    You should use the re-exported APIs under the top-level module.
"""

from types import ModuleType
from typing import TYPE_CHECKING

from pytauri.plugin import Plugin
from typing_extensions import TypedDict, Unpack

from pytauri_plugins import (
    PLUGIN_GLOBAL_SHORTCUT,
    _pytauri_plugins_mod,  # pyright: ignore[reportPrivateUsage]
)

__all__ = [
    "Builder",
    "BuilderArgs",
]

if PLUGIN_GLOBAL_SHORTCUT:
    _global_shortcut_mod: ModuleType = _pytauri_plugins_mod.global_shortcut
else:
    raise ImportError(
        "Enable the `plugin-global-shortcut` feature for `pytauri` crate to use this plugin."
    )


class BuilderArgs(TypedDict, total=False):
    """[tauri_plugin_global_shortcut::Builder](https://docs.rs/tauri-plugin-global-shortcut/latest/tauri_plugin_global_shortcut/struct.Builder.html)"""


if TYPE_CHECKING:

    class Builder:
        """[tauri_plugin_global_shortcut::Builder](https://docs.rs/tauri-plugin-global-shortcut/latest/tauri_plugin_global_shortcut/struct.Builder.html)"""

        @staticmethod
        def build(**kwargs: Unpack[BuilderArgs]) -> Plugin: ...

else:
    Builder = _global_shortcut_mod.Builder
