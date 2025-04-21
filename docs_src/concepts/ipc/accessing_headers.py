from pytauri import Commands
from pytauri.ipc import Headers

commands = Commands()


@commands.command()
async def command(body: bytes, headers: Headers) -> bytes:  # noqa: ARG001
    print(headers)
    return b"null"
