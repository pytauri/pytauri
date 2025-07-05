from os import getenv
from pathlib import Path

from anyio.from_thread import start_blocking_portal
from pydantic import BaseModel, ConfigDict
from pydantic.alias_generators import to_camel
from pytauri import (
    Commands,
    builder_factory,
    context_factory,
)

# ⭐ You should only enable this feature in development (not production)
PYTAURI_GEN_TS = getenv("PYTAURI_GEN_TS") != "0"

# ⭐ Enable this feature first
commands = Commands(experimental_gen_ts=PYTAURI_GEN_TS)


class _BaseModel(BaseModel):
    model_config = ConfigDict(
        # Accepts camelCase js ipc arguments for snake_case python fields.
        #
        # See: <https://docs.pydantic.dev/2.10/concepts/alias/#using-an-aliasgenerator>
        alias_generator=to_camel,
        # By default, pydantic allows unknown fields,
        # which results in TypeScript types having `[key: string]: unknown`.
        #
        # See: <https://docs.pydantic.dev/2.10/concepts/models/#extra-data>
        extra="forbid",
    )


class Person(_BaseModel):
    """A simple model representing a person.

    @property name - The name of the person.
    """

    # 👆 This pydoc will be converted to tsdoc
    name: str


# ⭐ Just use `commands` as usual
@commands.command()
async def greet_to_person(body: Person) -> str:
    """A simple command that returns a greeting message.

    @param body - The person to greet.
    """
    # 👆 This pydoc will be converted to tsdoc
    return f"Hello, {body.name}!"


def main() -> int:
    with start_blocking_portal("asyncio") as portal:
        if PYTAURI_GEN_TS:
            # ⭐ Generate TypeScript Client to your frontend `src/client` directory
            output_dir = Path(__file__).parent.parent.parent.parent / "src" / "client"
            # ⭐ The CLI to run `json-schema-to-typescript`,
            # `--format=false` is optional to improve performance
            json2ts_cmd = "pnpm json2ts --format=false"

            # ⭐ Start the background task to generate TypeScript types
            portal.start_task_soon(
                lambda: commands.experimental_gen_ts_background(
                    output_dir, json2ts_cmd, cmd_alias=to_camel
                )
            )

        app = builder_factory().build(
            context=context_factory(),
            invoke_handler=commands.generate_handler(portal),
        )
        exit_code = app.run_return()
        return exit_code
