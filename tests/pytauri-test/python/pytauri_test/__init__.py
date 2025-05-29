from collections.abc import Iterator
from contextlib import contextmanager
from typing import Literal, Optional, cast

from anyio import create_task_group
from anyio.abc import TaskGroup
from anyio.from_thread import start_blocking_portal
from pydantic import BaseModel, ConfigDict, RootModel
from pydantic.alias_generators import to_camel
from pytauri import (
    AppHandle,
    Commands,
    Emitter,
    Event,
    Listener,
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


# NOTE: dont change the command name `command`,
# it is used in the `test/ipc.rs`.
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


# NOTE: dont change the func name `app_handle_fixture`,
# it is used in the `test/ipc.rs`.
@contextmanager
def app_handle_fixture() -> Iterator[AppHandle]:
    global task_group
    with (
        start_blocking_portal("asyncio") as portal,  # or `trio`
        portal.wrap_async_context_manager(portal.call(create_task_group)) as task_group,
    ):
        app = builder_factory().build(
            context=context_factory(),
            invoke_handler=commands.generate_handler(portal),
        )
        yield app.handle()


def test_event_system():
    """Test `Emitter` and `Listener` event system."""

    event_name = "ping"
    event_payload = Pong("pong")

    with app_handle_fixture() as app_handle:
        received_event: Optional[Event] = None

        def handler(event: Event):
            nonlocal received_event
            received_event = event

        event_id = Listener.once(app_handle, event_name, handler)
        Emitter.emit(app_handle, event_name, event_payload)

        # TODO, FIXME: this is pyright bug, it mistakenly thinks `received_event` is `None`
        received_event = cast(Optional[Event], received_event)

        assert received_event is not None, f"event name `{event_name}` not received"
        assert received_event.id == event_id, "received event id mismatch"
        assert (
            Pong.model_validate_json(received_event.payload).root == event_payload.root
        ), "received event payload mismatch"


test_event_system()
