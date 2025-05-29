from anyio.from_thread import start_blocking_portal
from pytauri import Commands, builder_factory, context_factory

commands = Commands()


with start_blocking_portal("asyncio") as portal:  # or "trio"
    builder = builder_factory()
    app = builder.build(
        context_factory(),
        invoke_handler=commands.generate_handler(portal),
    )
    exit_code = app.run_return()
