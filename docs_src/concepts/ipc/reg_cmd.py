# pyright: reportRedeclaration=none
# ruff: noqa: F811

from pytauri import AppHandle, Commands

commands = Commands()


# ⭐ OK
@commands.command()
async def command(body: bytes) -> bytes: ...


# ⭐ OK
@commands.command()
async def command(body: bytes, app_handle: AppHandle) -> bytes: ...


# 💥 ERROR: missing/wrong type annotation
@commands.command()
async def command(
    body: bytes,
    app_handle,  # pyright: ignore[reportUnknownParameterType, reportMissingParameterType]  # noqa: ANN001
) -> bytes: ...


# 💥 ERROR: wrong parameter name
@commands.command()
async def command(body: bytes, foo: AppHandle) -> bytes: ...


# 💥 ERROR: not an async function
@commands.command()  # pyright: ignore[reportArgumentType, reportUntypedFunctionDecorator]
def command(body: bytes) -> bytes: ...
