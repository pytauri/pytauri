# ruff: noqa: D102

"""This module provides configuration for the compiled pytauri extension module."""

from typing import (
    TYPE_CHECKING,
)

from pytauri.ffi._ext_mod import pytauri_mod

__all__ = ["PLUGIN_NOTIFICATION"]

_pytauri_config_mod = pytauri_mod.pytauri_config

if TYPE_CHECKING:
    PLUGIN_NOTIFICATION: bool
    """Whether the `plugin-notification` feature is enabled."""

else:
    PLUGIN_NOTIFICATION = _pytauri_config_mod.PLUGIN_NOTIFICATION
