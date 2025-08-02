from os import environ

# This is an env var that can only be used internally by pytauri to distinguish
# between different example extension modules.
# You don't need and shouldn't set this in your own app.
# Must be set before importing any pytauri module.
environ["_PYTAURI_DIST"] = "tauri-app"

import sys
from datetime import datetime
from functools import partial
from pathlib import Path
from typing import Annotated

from anyio import sleep
from anyio.from_thread import start_blocking_portal
from pydantic import BaseModel, ConfigDict, RootModel
from pydantic.alias_generators import to_camel
from pytauri import (
    AppHandle,
    Commands,
    Manager,
    State,
    builder_factory,
    context_factory,
)
from pytauri.ipc import Channel, JavaScriptChannelId
from pytauri.webview import WebviewWindow
from pytauri_plugins import (
    autostart,
    clipboard_manager,
    deep_link,
    dialog,
    fs,
    http,
    notification,
    opener,
    os,
    persisted_scope,
    positioner,
)
from pytauri_plugins.dialog import DialogExt, MessageDialogButtons, MessageDialogKind
from pytauri_plugins.notification import NotificationExt
from pytauri_utils.async_tools import AsyncTools

from tauri_app.private import private_algorithm

PYTAURI_GEN_TS = environ.get("PYTAURI_GEN_TS") != "0"


commands = Commands(experimental_gen_ts=PYTAURI_GEN_TS)


Time = RootModel[datetime]


async def timer_task(time_channel: Channel[Time]) -> None:
    time = Time(datetime.now())
    while True:
        time_channel.send_model(time)
        await sleep(1)
        time.root = datetime.now()


@commands.command()
async def start_timer(
    body: JavaScriptChannelId[Time],
    webview_window: WebviewWindow,
    async_tools: Annotated[AsyncTools, State()],
) -> None:
    """Starts a timer that sends the current time to the specified channel every second.

    Args:
        body: The channel ID to send the time updates to.

    Returns:
        None
    """
    time_channel = body.channel_on(webview_window.as_ref_webview())
    async_tools.task_group.start_soon(timer_task, time_channel)


class _BaseModel(BaseModel):
    """Accepts camelCase js ipc arguments for snake_case python fields.

    See: <https://docs.pydantic.dev/2.10/concepts/alias/#using-an-aliasgenerator>
    """

    model_config = ConfigDict(
        alias_generator=to_camel,
        extra="forbid",
    )


class Person(_BaseModel):
    name: str


@commands.command()
async def greet(
    body: Person, app_handle: AppHandle, webview_window: WebviewWindow
) -> str:
    """Greets a person with a message.

    @param body - The person to greet.
    @returns The greeting message.
    """
    notification_builder = NotificationExt.builder(app_handle)

    message_dialog_builder = DialogExt.message(app_handle, f"Hello {body.name}!")
    message_dialog_builder.show(
        lambda is_ok: notification_builder.show(body=f"Dialog closed with: {is_ok}"),
        parent=webview_window,
        buttons=MessageDialogButtons.OkCancelCustom("ok", "cancel"),
        kind=MessageDialogKind.Info,
    )

    webview_window.set_title(f"Hello {body.name}!")

    return f"Hello, {body.name}! You've been greeted from Python {sys.version}!"


def main() -> int:
    """Run the tauri-app."""

    # test if the code protected by Cython is working correctly
    assert private_algorithm(42) == 84, "private_algorithm is not working!"

    with (
        start_blocking_portal("asyncio") as portal,  # or `trio`
        AsyncTools(portal) as async_tools,
    ):
        if PYTAURI_GEN_TS:
            output_dir = Path(__file__).parent.parent.parent.parent / "src" / "client"
            json2ts_cmd = "pnpm json2ts --format=false"
            portal.start_task_soon(
                partial(
                    commands.experimental_gen_ts_background,
                    output_dir,
                    json2ts_cmd,
                )
            )

        app = builder_factory().build(
            context=context_factory(),
            invoke_handler=commands.generate_handler(portal),
            plugins=(
                dialog.init(),
                notification.init(),
                clipboard_manager.init(),
                fs.init(),
                opener.init(),
                autostart.init(),
                deep_link.init(),
                http.init(),
                os.init(),
                persisted_scope.init(),
                positioner.init(),
            ),
        )
        Manager.manage(app, async_tools)

        exit_code = app.run_return()
        return exit_code
