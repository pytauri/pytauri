"""The main entry point for the Tauri app."""

from multiprocessing import freeze_support

from tauri_app import main

# - If you don't use `multiprocessing`, you can remove this line.
# - If you do use `multiprocessing` but without this line,
#   you will get endless spawn loop of your application process.
#   See: <https://pyinstaller.org/en/v6.11.1/common-issues-and-pitfalls.html#multi-processing>.
freeze_support()

main()
