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

from anyio import create_task_group, sleep
from anyio.abc import TaskGroup
from anyio.from_thread import start_blocking_portal
from pydantic import BaseModel, ConfigDict, RootModel
from pydantic.alias_generators import to_camel
from pytauri import (
    AppHandle,
    Commands,
    builder_factory,
    context_factory,
)
from pytauri.ipc import Channel, JavaScriptChannelId
from pytauri.webview import WebviewWindow
from pytauri_plugins.dialog import DialogExt, MessageDialogButtons, MessageDialogKind
from pytauri_plugins.notification import NotificationExt

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
    body: JavaScriptChannelId[Time], webview_window: WebviewWindow
) -> None:
    """Starts a timer that sends the current time to the specified channel every second.

    Args:
        body: The channel ID to send the time updates to.

    Returns:
        None
    """
    time_channel = body.channel_on(webview_window.as_ref_webview())

    # NOTE:
    #
    # - When this command (`start_timer`) is called, the `task_group` has already been created,
    #   so we can use it directly.
    #
    # - The async context in which this command is called and the async context of the `task_group`
    #   are both the same `blocking_portal`, so we don't need to worry about thread safety issues.
    #   I.e, we can call `task_group.start_soon` directly, instead of needing use `portal.start_soon`.
    #
    #   Thread safety references:
    #   - <https://anyio.readthedocs.io/en/stable/threads.html>
    #   - <https://docs.python.org/3/library/asyncio-task.html#asyncio.run_coroutine_threadsafe>
    #
    # ---
    #
    # Or if you use `asyncio`, you can use `asyncio.create_task` derectly,
    # so that you don't need init `task_group`.
    task_group.start_soon(timer_task, time_channel)


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


# Anyio `TaskGroup` can only be created in async context,
# So we need to use `portal.call` to create it,
# and use `portal.wrap_async_context_manager` to enter it.
task_group: TaskGroup


def main() -> int:
    """Run the tauri-app."""

    # test if the code protected by Cython is working correctly
    assert private_algorithm(42) == 84, "private_algorithm is not working!"

    global task_group
    with (
        start_blocking_portal("asyncio") as portal,  # or `trio`
        portal.wrap_async_context_manager(portal.call(create_task_group)) as task_group,
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
        )
        exit_code = app.run_return()
        return exit_code
