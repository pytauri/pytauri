from collections.abc import Iterator
from contextlib import contextmanager
from typing import Literal

from anyio import create_task_group
from anyio.abc import TaskGroup
from anyio.from_thread import start_blocking_portal
from pydantic import BaseModel, ConfigDict, RootModel
from pydantic.alias_generators import to_camel
from pytauri import (
    AppHandle,
    BuilderArgs,
    Commands,
    builder_factory,
    context_factory,
)
from pytauri.ipc import Channel, JavaScriptChannelId
from pytauri.webview import WebviewWindow

__all__ = ["app_handle_fixture"]

commands = Commands()


ChannelBody = RootModel[Literal["ping"]]


class _CamelModel(BaseModel):
    model_config = ConfigDict(
        alias_generator=to_camel,
    )


class Body(_CamelModel):
    ping: Literal["ping"]
    channel_id: JavaScriptChannelId[ChannelBody]


Pong = RootModel[Literal["pong"]]


async def channel_task(channel: Channel[ChannelBody]) -> None:
    channel.send_model(ChannelBody("ping"))


@commands.command()
async def command(
    body: Body,
    app_handle: AppHandle,  # noqa: ARG001
    webview_window: WebviewWindow,
) -> Pong:
    assert body.ping == "ping"

    channel = body.channel_id.channel_on(webview_window.as_ref_webview())

    await channel_task(channel)

    return Pong("pong")


task_group: TaskGroup


@contextmanager
def app_handle_fixture() -> Iterator[AppHandle]:
    global task_group
    with (
        start_blocking_portal("asyncio") as portal,  # or `trio`
        portal.wrap_async_context_manager(portal.call(create_task_group)) as task_group,
    ):
        app = builder_factory().build(
            BuilderArgs(
                context=context_factory(),
                invoke_handler=commands.generate_handler(portal),
            )
        )
        yield app.handle()
