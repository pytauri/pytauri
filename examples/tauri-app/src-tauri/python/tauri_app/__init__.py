# ruff: noqa: D101, D103

"""The tauri-app."""

import sys

from anyio.from_thread import start_blocking_portal
from pydantic import BaseModel
from pytauri import (
    AppHandle,
    BuilderArgs,
    Commands,
    builder_factory,
    context_factory,
)
from pytauri.webview import WebviewWindow
from pytauri_plugin_notification import NotificationBuilderArgs, NotificationExt

commands: Commands = Commands()


class Person(BaseModel):
    name: str


class Greeting(BaseModel):
    message: str


@commands.command()
async def greet(
    body: Person, app_handle: AppHandle, webview_window: WebviewWindow
) -> Greeting:
    notification_builder = NotificationExt.builder(app_handle)
    notification_builder.show(
        NotificationBuilderArgs(title="Greeting", body=f"Hello, {body.name}!")
    )

    webview_window.set_title(f"Hello {body.name}!")

    return Greeting(
        message=f"Hello, {body.name}! You've been greeted from Python {sys.version}!"
    )


def main() -> None:
    """Run the tauri-app."""
    with start_blocking_portal("asyncio") as portal:  # or `trio`
        app = builder_factory().build(
            BuilderArgs(
                context=context_factory(),
                invoke_handler=commands.generate_handler(portal),
            )
        )
        app.run()
