# pyright: reportRedeclaration=none
# ruff: noqa: F811

from datetime import datetime
from typing import Optional, Union

from pydantic import RootModel
from pytauri import AppHandle, Commands

commands = Commands()

StrModel = RootModel[str]


# ⭐ OK
@commands.command()
async def command(body: datetime, app_handle: AppHandle) -> None: ...


# ⭐ OK
@commands.command()
async def command(body: Union[str, int]) -> bytes: ...


# ⭐ OK
@commands.command()
async def command(body: Optional[str]) -> StrModel: ...


# ⭐ OK
@commands.command()
async def command() -> bool: ...
