from typing import Annotated

from anyio import create_memory_object_stream
from pytauri import AppHandle, Commands, Event, Listener, State
from pytauri_utils.async_tools import AsyncTools

commands = Commands()


@commands.command()
async def command_handler(
    app_handle: AppHandle, async_tools: Annotated[AsyncTools, State()]
) -> None:
    send_stream, receive_stream = create_memory_object_stream[str]()

    # ⭐ Convert an asynchronous function to a synchronous one
    @async_tools.to_sync
    async def listener(event: Event) -> None:
        async with send_stream:
            await send_stream.send(event.payload)

    # ⭐ Then we can use it as synchronous callback
    Listener.once(app_handle, "foo-event", listener)

    async with receive_stream:
        print("Received: ", await receive_stream.receive())
