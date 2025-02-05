from os import environ

# This is an env var that can only be used internally by pytauri to distinguish
# between different example extension modules.
# You don't need and shouldn't set this in your own app.
# Must be set before importing any pytauri module.
environ["_PYTAURI_DIST"] = "nicegui-app"

import sys
from concurrent.futures import Future
from socket import socket
from threading import Event
from typing import Any, Optional

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
from typing_extensions import override


class FrontServer(uvicorn.Server):
    """Override `uvicorn.Server` to set events on startup and shutdown."""

    def __init__(
        self, config: uvicorn.Config, *, startup_event: Event, shutdown_event: Event
    ) -> None:
        super().__init__(config)
        self.startup_event = startup_event
        self.shutdown_event = shutdown_event

    @override
    async def startup(self, sockets: Optional[list[socket]] = None) -> None:
        """Set the startup event after the server is started."""
        await super().startup(sockets)
        self.startup_event.set()

    @override
    async def shutdown(self, sockets: Optional[list[socket]] = None) -> None:
        """Set the shutdown event after the server is shutdown."""
        await super().shutdown(sockets)
        self.shutdown_event.set()

    def request_shutdown(self) -> None:
        """Request the server to shutdown.

        Note:
            This method is not thread-safe.

        Ref:
            - <https://github.com/zauberzeug/nicegui/discussions/1957#discussioncomment-7484548>
            - <https://github.com/encode/uvicorn/discussions/1103#discussioncomment-6187606>
        """
        self.should_exit = True


app_handle: AppHandle
"""We will initialize it in the `main` function later."""
webview_window: WebviewWindow
"""We will initialize it in the `main` function later."""


@ui.page("/")
def root_ui() -> None:
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


def main() -> None:
    nicegui_app = FastAPI()
    ui.run_with(nicegui_app)
    server_startup_event = Event()
    server_shutdown_event = Event()
    server = FrontServer(
        # `host` and `port` are the same as `frontendDist` in `tauri.conf.json`
        uvicorn.Config(nicegui_app, host="localhost", port=8080),
        shutdown_event=server_shutdown_event,
        startup_event=server_startup_event,
    )

    with start_blocking_portal("asyncio") as portal:  # or `trio`
        server_exception: Optional[BaseException] = None

        def server_failed_callback(future: Future[Any]) -> None:
            """Add a callback to check if the server broke down."""
            nonlocal server_exception
            server_exception = future.exception()
            if server_exception is not None:
                # server startup failed, so we must set these events manually for app,
                # or the app will hang for waiting these `Event`s forever.
                server_startup_event.set()
                server_shutdown_event.set()

        # launch the front server
        portal.start_task_soon(server.serve).add_done_callback(server_failed_callback)

        # launch the app
        tauri_app = builder_factory().build(
            BuilderArgs(
                context=context_factory(),
            )
        )

        # set the global variable `app_handle`.
        global app_handle
        app_handle = tauri_app.handle()

        def tauri_run_callback(app_handle: AppHandle, run_event: RunEventType) -> None:
            """Add a callback to show the main window after the server is started and
            shutdown the server when the app is going to exit."""

            # show the main window after the server is started,
            # and set the global variable `webview_window`.
            if isinstance(run_event, RunEvent.Ready):
                webview_window_ = Manager.get_webview_window(app_handle, "main")
                assert (
                    webview_window_ is not None
                ), "you forgot to set the unvisible 'main' window in `tauri.conf.json`"
                global webview_window
                webview_window = webview_window_

                # wait for the front server to start and show the window
                server_startup_event.wait()
                webview_window_.show()

                # check is the server failed to start, if so, show the error message.
                if (
                    server_exception is not None
                    or server.should_exit  # server/nicegui_app startup failed
                ):
                    webview_window_.eval(
                        "document.body.innerHTML = `failed to start front server, see backend logs for details`"
                    )

            elif isinstance(run_event, RunEvent.Exit):
                # user closed the window so the app is going to exit,
                # we need shutdown the front server first.
                server.request_shutdown()
                server_shutdown_event.wait()

        tauri_app.run(tauri_run_callback)
