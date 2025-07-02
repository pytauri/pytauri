"""The main entry point for the Tauri app."""

from os import environ

if environ.get("PYTAURI_DEBUG_PY") == "1":
    import debugpy  # pyright: ignore[reportMissingTypeStubs]

    debugpy.listen(5678)
    print("Waiting for debugger to attach...")
    # debugpy.wait_for_client()


if environ.get("PYTAURI_DEBUG_RS") == "1":
    import codelldb

    codelldb.debug(host="localhost", port=9552, token="secret")

# -- main ---------------------------------------------------------------------

import sys
from multiprocessing import freeze_support

from tauri_app import main

# - If you don't use `multiprocessing`, you can remove this line.
# - If you do use `multiprocessing` but without this line,
#   you will get endless spawn loop of your application process.
#   See: <https://pyinstaller.org/en/v6.11.1/common-issues-and-pitfalls.html#multi-processing>.
freeze_support()

sys.exit(main())
