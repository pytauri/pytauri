from contextlib import ExitStack
from sys import exc_info

from anyio.from_thread import start_blocking_portal
from pytauri import Commands

commands = Commands()

exit_stack = ExitStack()
portal = exit_stack.enter_context(start_blocking_portal("asyncio"))


# 👉 the `invoke_handler` will keep available until the `ExitStack` is closed
invoke_handler = commands.generate_handler(portal)

"""do some stuff ..."""

# 👉 then remember to close the `ExitStack` to exit the portal
exit_stack.__exit__(*exc_info())
