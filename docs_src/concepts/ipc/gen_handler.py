from anyio.from_thread import start_blocking_portal
from pytauri import BuilderArgs, Commands, builder_factory, context_factory

commands = Commands()


with start_blocking_portal("asyncio") as portal:  # or "trio"
    builder = builder_factory()
    app = builder.build(
        BuilderArgs(
            context_factory(),
            # 👇
            invoke_handler=commands.generate_handler(portal),
        )
    )
    app.run()
