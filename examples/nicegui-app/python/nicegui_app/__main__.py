"""The main entry point for the Tauri app."""

import sys

from nicegui_app import main

# `True` means that running on pytauri standalone mode.
if getattr(sys, "frozen", False):  # noqa: SIM102
    # If `stderr` or `stdout` is None, it means `windows_subsystem = "windows"` on the Rust side,
    # so there is no console to output.
    # However, uvicorn's logger defaults to outputting to `stderr` and `stdout`,
    # and if they do not exist, it will raise an error.
    # So we need to redirect `stderr` and `stdout` to a file.
    #
    # See more:
    #
    # - <https://github.com/zauberzeug/nicegui/issues/681>
    # - <https://github.com/r0x0r/pywebview/pull/1086>
    if sys.stderr is None or sys.stdout is None:
        _output = open("nicegui-app.log", "w")  # noqa: SIM115 # keep it open until the whole python ends.
        if sys.stderr is None:
            sys.stderr = _output
        if sys.stdout is None:
            sys.stdout = _output

main()
