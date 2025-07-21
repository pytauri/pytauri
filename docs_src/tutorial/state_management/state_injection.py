# pyright: reportRedeclaration=none
# ruff: noqa: ARG001, F811

from typing import Annotated

from pytauri import AppHandle, Commands, State

commands = Commands()


class MyState: ...


@commands.command()
async def command(
    app_handle: AppHandle,
    my_state: Annotated[MyState, State()],  # ⭐
) -> None:
    assert isinstance(my_state, MyState)


@commands.command()
async def command(
    body: int,
    # ⭐ Arbitrary names, arbitrary quantities, and arbitrary types of state injection
    my_state_foo: Annotated[MyState, State()],
    my_state_bar: Annotated[str, State()],
) -> None:
    pass
