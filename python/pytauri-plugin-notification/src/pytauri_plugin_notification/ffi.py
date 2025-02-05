"""Original FFI interface module.

!!! warning
    All APIs under this module should not be considered stable.
    You should use the re-exported APIs under the top-level module.
"""

from types import ModuleType
from typing import TYPE_CHECKING, Optional, final

from pytauri import EXT_MOD, ImplManager
from typing_extensions import TypeAlias

__all__ = [
    "ImplNotificationExt",
    "NotificationBuilder",
    "NotificationExt",
]


def _load_notification_mod(ext_mod: ModuleType) -> ModuleType:
    try:
        notification_mod = ext_mod.notification
    except AttributeError as e:
        raise RuntimeError(
            "Submodule `notification` is not found in the extension module"
        ) from e

    assert isinstance(notification_mod, ModuleType)
    return notification_mod


_notification_mod = _load_notification_mod(EXT_MOD)

if TYPE_CHECKING:

    @final
    class NotificationBuilder:
        """[tauri_plugin_notification::NotificationBuilder](https://docs.rs/tauri-plugin-notification/latest/tauri_plugin_notification/struct.NotificationBuilder.html)"""

        def show(
            self,
            /,
            *,
            id: Optional[int] = None,  # noqa: A002
            channel_id: Optional[str] = None,
            title: Optional[str] = None,
            body: Optional[str] = None,
            large_body: Optional[str] = None,
            summary: Optional[str] = None,
            action_type_id: Optional[str] = None,
            group: Optional[str] = None,
            group_summary: bool = False,
            sound: Optional[str] = None,
            inbox_line: Optional[str] = None,
            icon: Optional[str] = None,
            large_icon: Optional[str] = None,
            icon_color: Optional[str] = None,
            ongoing: bool = False,
            auto_cancel: bool = False,
            silent: bool = False,
        ) -> None:
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
    NotificationBuilder = _notification_mod.NotificationBuilder
    NotificationExt = _notification_mod.NotificationExt

ImplNotificationExt: TypeAlias = ImplManager
"""The implementors of `NotificationExt`."""
