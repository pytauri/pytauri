from os import environ

# This is an env var that can only be used internally by pytauri to distinguish
# between different example extension modules.
# You don't need and shouldn't set this in your own app.
# Must be set before importing any pytauri module.
environ["_PYTAURI_DIST"] = "nicegui-app"

import sys
from typing import Callable

import uvicorn
from anyio.from_thread import start_blocking_portal
from fastapi import FastAPI
from nicegui import ui
from pytauri import (
    AppHandle,
    BuilderArgs,
    Manager,
    RunEvent,
    RunEventType,
    builder_factory,
    context_factory,
)
from pytauri.webview import WebviewWindow
from pytauri_plugin_notification import NotificationExt

from nicegui_app._server import FrontServer
from nicegui_app._tray_menu import init_menu, init_tray

app_handle: AppHandle
"""We will initialize it in the `main` function later."""
webview_window: WebviewWindow
"""We will initialize it in the `main` function later."""


@ui.page("/")
def root() -> None:
    """Draw the nicegui UI and set event callbacks."""

    # Since we only display the window after `app_handle` and `webview_window` are initialized,
    # we can directly use `app_handle` and `webview_window` here.

    async def greet():
        notification_builder = NotificationExt.builder(app_handle)
        notification_builder.show(title="Greeting", body=f"Hello, {name.value}!")

        webview_window.set_title(f"Hello {name.value}!")

        message.set_text(
            f"Hello, {name.value}! You've been greeted from Python {sys.version}!"
        )

    with ui.row():
        name = ui.input("Enter a name...")
        ui.button("Greet").on_click(greet)
    message = ui.label()


def app_setup_hook(front_server: FrontServer) -> Callable[[AppHandle], None]:
    """Set the global var `app_handle` and `webview_window`;
    and initialize the ui, tray icon and menu;
    and show the main window once the front server is ready.
    """

    def _app_setup_hook(app_handle_: AppHandle) -> None:
        global app_handle
        app_handle = app_handle_

        webview_window_ = Manager.get_webview_window(app_handle, "main")
        assert (
            webview_window_ is not None
        ), "you forgot to set the unvisible 'main' window in `tauri.conf.json`"
        global webview_window
        webview_window = webview_window_

        # wait for the front server to start and show the window
        front_server.wait_for_startup()

        # initialize the tray icon and menu
        init_tray(app_handle, webview_window)
        init_menu(app_handle)
        webview_window.show()
        if front_server.serve_exception is not None:
            webview_window.eval(
                "document.body.innerHTML = `failed to start front server, see backend logs for details`"
            )

    return _app_setup_hook


def main() -> None:
    nicegui_app = FastAPI()
    ui.run_with(nicegui_app)
    front_server = FrontServer(
        # `host` and `port` are the same as `frontendDist` in `tauri.conf.json`
        uvicorn.Config(nicegui_app, host="localhost", port=8080),
    )

    with start_blocking_portal("asyncio") as portal:  # or `trio`
        # launch the front server
        portal.start_task_soon(front_server.serve)

        # launch the app
        tauri_app = builder_factory().build(
            BuilderArgs(
                context_factory(),
                setup=app_setup_hook(front_server),
            )
        )

        def tauri_run_callback(_: AppHandle, run_event: RunEventType) -> None:
            """Add a callback to show the main window after the server is started and
            shutdown the server when the app is going to exit."""

            match run_event:
                # user closed the window so the app is going to exit,
                # we need shutdown the front server first.
                case RunEvent.Exit():
                    front_server.request_shutdown()
                case _:
                    pass

        tauri_app.run(tauri_run_callback)
