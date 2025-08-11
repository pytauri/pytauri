"""[tauri_plugin_deep_link::self](https://docs.rs/tauri-plugin-deep-link/latest/tauri_plugin_deep_link/index.html)

!!! bug
    When using this plugin in `pnpm tauri dev` mode,
    please hardcode [PythonInterpreterEnv::Venv] as an absolute path (e.g., [/absolute/path/to/.venv/])
    instead of relying on the `VIRTUAL_ENV` environment variable.

    This is because the `deep-link` execution environment does not include the `VIRTUAL_ENV` variable.

    [PythonInterpreterEnv::Venv]: https://docs.rs/pytauri/latest/pytauri/standalone/enum.PythonInterpreterEnv.html#variant.Venv
    [/absolute/path/to/.venv/]: https://github.com/pytauri/pytauri/blob/10206e89f4925b35569c93d6797bfd401dea267b/examples/tauri-app/src-tauri/src/main.rs#L19-L33
"""

from pytauri_plugins.deep_link.ffi import (
    init,
)

__all__ = [
    "init",
]
