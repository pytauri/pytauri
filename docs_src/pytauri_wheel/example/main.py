from os import environ

# This is an env var that can only be used internally by pytauri to distinguish
# between different example extension modules.
# You don't need and shouldn't set this in your own app.
# Must be set before importing any pytauri module.
environ["_PYTAURI_DIST"] = "pytauri-wheel"

# --8<-- [start:code]

import json
import sys
from os import environ
from pathlib import Path

from anyio.from_thread import start_blocking_portal
from pytauri import Commands
from pytauri_wheel.lib import builder_factory, context_factory

SRC_TAURI_DIR = Path(__file__).parent.absolute()
"""In rust tauri project, it's usually `src-tauri` dir.

📁 {SRC_TAURI_DIR}/
├── 📁 capabilities/
├── 📄 tauri.conf.json
└── ...
"""

DEV_SERVER = environ.get("DEV_SERVER")
"""Whether to use frontend dev server to serve frontend assets.

e.g, `Vite` dev server on `http://localhost:1420`.
"""

commands = Commands()


@commands.command()
async def greet() -> bytes:
    return json.dumps(sys.version).encode()


if DEV_SERVER is not None:
    # ref: <https://tauri.app/reference/config/#frontenddist-1>
    tauri_config = json.dumps(
        {
            "build": {
                "frontendDist": DEV_SERVER,
            },
        }
    )
else:
    tauri_config = None


def main() -> int:
    with start_blocking_portal("asyncio") as portal:  # or `trio`
        app = builder_factory().build(
            context=context_factory(SRC_TAURI_DIR, tauri_config=tauri_config),
            invoke_handler=commands.generate_handler(portal),
        )
        exit_code = app.run_return()
        return exit_code


if __name__ == "__main__":
    sys.exit(main())

# --8<-- [end:code]
