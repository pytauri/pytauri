from typing import Annotated

from anyio.from_thread import start_blocking_portal
from pytauri import (
    Commands,
    ImplManager,
    Manager,
    State,
    builder_factory,
    context_factory,
)
from pytauri_utils.async_tools import AsyncTools

commands = Commands()


def main() -> int:
    with (
        start_blocking_portal("asyncio") as portal,  # or `trio`
        AsyncTools(portal) as async_tools,  # ⭐ initialize AsyncTools
    ):
        app = builder_factory().build(
            context=context_factory(),
            invoke_handler=commands.generate_handler(portal),
        )

        # ⭐ Add `AsyncTools` to app state
        Manager.manage(app, async_tools)

        exit_code = app.run_return()
        return exit_code


# ⭐ Access `AsyncTools` from app state
def access_async_tools_via_api(manager: ImplManager) -> AsyncTools:
    async_tools = Manager.state(manager, AsyncTools)
    return async_tools


# ⭐ Access `AsyncTools` from app state in a command
@commands.command()
async def access_async_via_command_injection(
    async_tools: Annotated[AsyncTools, State()],
) -> None:
    print(async_tools)
