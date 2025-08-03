"""This module provides [pytauri/tauri plugins](https://github.com/tauri-apps/plugins-workspace).

Currently, it is distributed as part of the [`pytauri`](https://pypi.org/project/pytauri/) package on PyPI.
Therefore, running `pip install pytauri` will also install this module.
"""

from types import ModuleType
from typing import Final

from pytauri import EXT_MOD

__all__ = [
    "PLUGIN_AUTOSTART",
    "PLUGIN_CLIPBOARD_MANAGER",
    "PLUGIN_DEEP_LINK",
    "PLUGIN_DIALOG",
    "PLUGIN_FS",
    "PLUGIN_HTTP",
    "PLUGIN_NOTIFICATION",
    "PLUGIN_OPENER",
    "PLUGIN_OS",
    "PLUGIN_PERSISTED_SCOPE",
    "PLUGIN_POSITIONER",
    "PLUGIN_PROCESS",
    "PLUGIN_SHELL",
    "PLUGIN_SINGLE_INSTANCE",
    "PLUGIN_UPDATER",
    "PLUGIN_UPLOAD",
]

_pytauri_plugins_mod: ModuleType = EXT_MOD.pytauri_plugins

PLUGIN_NOTIFICATION: Final[bool] = _pytauri_plugins_mod.PLUGIN_NOTIFICATION
"""Whether the `plugin-notification` feature is enabled."""
PLUGIN_DIALOG: Final[bool] = _pytauri_plugins_mod.PLUGIN_DIALOG
"""Whether the `plugin-dialog` feature is enabled."""
PLUGIN_CLIPBOARD_MANAGER: Final[bool] = _pytauri_plugins_mod.PLUGIN_CLIPBOARD_MANAGER
"""Whether the `plugin-clipboard-manager` feature is enabled."""
PLUGIN_FS: Final[bool] = _pytauri_plugins_mod.PLUGIN_FS
"""Whether the `plugin-fs` feature is enabled."""
PLUGIN_OPENER: Final[bool] = _pytauri_plugins_mod.PLUGIN_OPENER
"""Whether the `plugin-opener` feature is enabled."""
PLUGIN_AUTOSTART: Final[bool] = _pytauri_plugins_mod.PLUGIN_AUTOSTART
"""Whether the `plugin-autostart` feature is enabled."""
PLUGIN_DEEP_LINK: Final[bool] = _pytauri_plugins_mod.PLUGIN_DEEP_LINK
"""Whether the `plugin-deep-link` feature is enabled."""
PLUGIN_HTTP: Final[bool] = _pytauri_plugins_mod.PLUGIN_HTTP
"""Whether the `plugin-http` feature is enabled."""
PLUGIN_OS: Final[bool] = _pytauri_plugins_mod.PLUGIN_OS
"""Whether the `plugin-os` feature is enabled."""
PLUGIN_PERSISTED_SCOPE: Final[bool] = _pytauri_plugins_mod.PLUGIN_PERSISTED_SCOPE
"""Whether the `plugin-persisted-scope` feature is enabled."""
PLUGIN_POSITIONER: Final[bool] = _pytauri_plugins_mod.PLUGIN_POSITIONER
"""Whether the `plugin-positioner` feature is enabled."""
PLUGIN_PROCESS: Final[bool] = _pytauri_plugins_mod.PLUGIN_PROCESS
"""Whether the `plugin-process` feature is enabled."""
PLUGIN_SHELL: Final[bool] = _pytauri_plugins_mod.PLUGIN_SHELL
"""Whether the `plugin-shell` feature is enabled."""
PLUGIN_SINGLE_INSTANCE: Final[bool] = _pytauri_plugins_mod.PLUGIN_SINGLE_INSTANCE
"""Whether the `plugin-single-instance` feature is enabled."""
PLUGIN_UPDATER: Final[bool] = _pytauri_plugins_mod.PLUGIN_UPDATER
"""Whether the `plugin-updater` feature is enabled."""
PLUGIN_UPLOAD: Final[bool] = _pytauri_plugins_mod.PLUGIN_UPLOAD
"""Whether the `plugin-upload` feature is enabled."""
