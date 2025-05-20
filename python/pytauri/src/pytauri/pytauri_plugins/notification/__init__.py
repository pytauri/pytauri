"""[tauri_plugin_notification::self](https://docs.rs/tauri-plugin-notification/latest/tauri_plugin_notification/index.html)"""

from pytauri.ffi.pytauri_config import PLUGIN_NOTIFICATION

if PLUGIN_NOTIFICATION:
    from pytauri.pytauri_plugins.notification.ffi import (
        ImplNotificationExt,
        NotificationBuilder,
        NotificationExt,
    )

    __all__ = [
        "ImplNotificationExt",
        "NotificationBuilder",
        "NotificationExt",
    ]
