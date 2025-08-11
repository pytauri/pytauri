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
    PLUGIN_WINDOW_STATE,
    _pytauri_plugins_mod,  # pyright: ignore[reportPrivateUsage]
)

__all__ = [
    "Builder",
    "BuilderArgs",
]

if PLUGIN_WINDOW_STATE:
    _window_state_mod: ModuleType = _pytauri_plugins_mod.window_state
else:
    raise ImportError(
        "Enable the `plugin-window-state` feature for `pytauri` crate to use this plugin."
    )


class BuilderArgs(TypedDict, total=False):
    """[tauri_plugin_window_state::Builder](https://docs.rs/tauri-plugin-window-state/latest/tauri_plugin_window_state/struct.Builder.html)"""


if TYPE_CHECKING:

    class Builder:
        """[tauri_plugin_window_state::Builder](https://docs.rs/tauri-plugin-window-state/latest/tauri_plugin_window_state/struct.Builder.html)"""

        @staticmethod
        def build(**kwargs: Unpack[BuilderArgs]) -> Plugin: ...

else:
    Builder = _window_state_mod.Builder
