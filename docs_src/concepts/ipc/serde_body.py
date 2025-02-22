# pyright: reportRedeclaration=none
# ruff: noqa: F811

from pydantic import BaseModel, RootModel
from pytauri import AppHandle, Commands

commands = Commands()


class Input(BaseModel):
    foo: str
    bar: int


Output = RootModel[list[str]]


# ⭐ OK
@commands.command()
async def command(body: Input, app_handle: AppHandle) -> Output: ...


# ⭐ OK
@commands.command()
async def command(body: Input) -> bytes: ...


# ⭐ OK
@commands.command()
async def command(body: bytes) -> Output: ...
