from pytauri import (
    AppHandle,
)
from pytauri.menu import Menu, MenuEvent, MenuItem, PredefinedMenuItem
from pytauri.tray import MouseButton, TrayIcon, TrayIconEvent, TrayIconEventType
from pytauri.webview import WebviewWindow

__all__ = ["init_menu", "init_tray"]


def init_tray(app_handle: AppHandle, webview_window: WebviewWindow) -> None:
    """Initialize the tray icon."""

    tray = TrayIcon(app_handle)
    tray.set_icon(app_handle.default_window_icon())
    tray.set_show_menu_on_left_click(False)  # see tauri docs, Linux: Unsupported.
    tray.set_menu(
        Menu.with_items(
            app_handle,
            (
                MenuItem.with_id(app_handle, "Show", "Show", True),
                MenuItem.with_id(app_handle, "Hide", "Hide", True),
                PredefinedMenuItem.separator(app_handle),
                MenuItem.with_id(app_handle, "Quit", "Quit", True),
            ),
        )
    )

    def on_menu_event(app_handle: AppHandle, menu_event: MenuEvent) -> None:
        """Hide, show or quit the app when the tray menu is clicked."""
        match menu_event:
            case "Hide":
                webview_window.hide()
            case "Show":
                webview_window.show()
                webview_window.set_focus()
            case "Quit":
                webview_window.close()
                app_handle.exit(0)
            case _:
                pass

    tray.on_menu_event(on_menu_event)

    def on_tray_icon_event(_tray: TrayIcon, event: TrayIconEventType) -> None:
        """Show the main window when the tray icon is double-left-clicked."""
        match event:
            case TrayIconEvent.DoubleClick(button=MouseButton.Left):
                webview_window.show()
                webview_window.set_focus()
            case _:
                pass

    tray.on_tray_icon_event(on_tray_icon_event)


def init_menu(app_handle: AppHandle) -> None:
    """Initialize the default app menu."""

    menu = Menu.default(app_handle)
    menu.set_as_app_menu()
