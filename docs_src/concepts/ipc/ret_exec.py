from pytauri import Commands
from pytauri.ipc import InvokeException

commands = Commands()


@commands.command()
async def command() -> bytes:
    raise InvokeException("error message")
