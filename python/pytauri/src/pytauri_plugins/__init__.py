# ruff: noqa: D102

"""This module provides [pytauri/tauri plugins](https://github.com/tauri-apps/plugins-workspace).

Currently, it is distributed as part of the [`pytauri`](https://pypi.org/project/pytauri/) package on PyPI.
Therefore, running `pip install pytauri` will also install this module.
"""

from types import ModuleType
from typing import Final

from pytauri import EXT_MOD

__all__ = [
    "PLUGIN_NOTIFICATION",
]

_pytauri_plugins_mod: ModuleType = EXT_MOD.pytauri_plugins

PLUGIN_NOTIFICATION: Final[bool] = _pytauri_plugins_mod.PLUGIN_NOTIFICATION
"""Whether the `plugin-notification` feature is enabled."""
