"""Original FFI interface module.

!!! warning
    All APIs under this module should not be considered stable.
    You should use the re-exported APIs under the top-level module.
"""

from types import ModuleType
from typing import TYPE_CHECKING, final

from pytauri import ImplManager
from pytauri.plugin import Plugin
from typing_extensions import TypeAlias, TypedDict, Unpack

from pytauri_plugins import (
    PLUGIN_NOTIFICATION,
    _pytauri_plugins_mod,  # pyright: ignore[reportPrivateUsage]
)

__all__ = [
    "ImplNotificationExt",
    "NotificationBuilder",
    "NotificationBuilderArgs",
    "NotificationExt",
    "init",
]

if PLUGIN_NOTIFICATION:
    _notification_mod: ModuleType = _pytauri_plugins_mod.notification
else:
    raise ImportError(
        "Enable the `plugin-notification` feature for `pytauri` crate to use this plugin."
    )


class NotificationBuilderArgs(TypedDict, total=False):
    """[tauri_plugin_notification::NotificationBuilder](https://docs.rs/tauri-plugin-notification/latest/tauri_plugin_notification/struct.NotificationBuilder.html)"""

    id: int
    channel_id: str
    title: str
    body: str
    large_body: str
    summary: str
    action_type_id: str
    group: str
    group_summary: bool
    sound: str
    inbox_line: str
    icon: str
    large_icon: str
    icon_color: str
    ongoing: bool
    auto_cancel: bool
    silent: bool


if TYPE_CHECKING:

    def init() -> Plugin:
        """[tauri_plugin_notification::init](https://docs.rs/tauri-plugin-notification/latest/tauri_plugin_notification/fn.init.html)"""
        ...

    @final
    class NotificationBuilder:
        """[tauri_plugin_notification::NotificationBuilder](https://docs.rs/tauri-plugin-notification/latest/tauri_plugin_notification/struct.NotificationBuilder.html)"""

        def show(self, /, **kwargs: Unpack[NotificationBuilderArgs]) -> None:
            """Consume this builder and show the notification.

            # FIXME, XXX, TODO:

            See: <https://github.com/tauri-apps/tauri/issues/3700>

            On windows, you must install the package via the `.msi` or `nsis`, or `tauri-plugin-notification` will not work.

            Tracker issue: <https://github.com/tauri-apps/plugins-workspace/issues/2156>
            """
            ...

    @final
    class NotificationExt:
        """[tauri_plugin_notification::NotificationExt](https://docs.rs/tauri-plugin-notification/latest/tauri_plugin_notification/trait.NotificationExt.html)"""

        @staticmethod
        def builder(slf: "ImplNotificationExt", /) -> NotificationBuilder:
            """Create a new notification builder."""
            ...

else:
    init = _notification_mod.init
    NotificationBuilder = _notification_mod.NotificationBuilder
    NotificationExt = _notification_mod.NotificationExt

ImplNotificationExt: TypeAlias = ImplManager
"""The implementers of `NotificationExt`."""
