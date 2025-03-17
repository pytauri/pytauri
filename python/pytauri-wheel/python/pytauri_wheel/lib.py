"""PyTauri precompiled wheels.

# Usage

`pytauri-wheel` provides precompiled [pytauri.EXT_MOD][] for [pytauri][],
which means you can normally use the `pytauri` API, except for the following APIs provided by `pytauri-wheel`:

- [pytauri.builder_factory][] -> [pytauri_wheel.lib.builder_factory][]
- [pytauri.context_factory][] -> [pytauri_wheel.lib.context_factory][]
"""

from pathlib import Path
from typing import Optional

from pytauri import Builder, Context
from pytauri import (
    builder_factory as pytauri_builder_factory,
)
from pytauri import (
    context_factory as pytauri_context_factory,
)

__all__ = ["builder_factory", "context_factory"]


def builder_factory(
    *,
    opener: bool = True,
    clipboard_manager: bool = True,
    dialog: bool = True,
    fs: bool = True,
) -> Builder:
    """A factory function for creating a [pytauri.Builder][] instance.

    This is a type-hinted wrapper function for [pytauri.builder_factory][].

    Args:
        opener: Whether to enable the plugin `opener`
        clipboard_manager: Whether to enable the plugin `clipboard-manager`
        dialog: Whether to enable the plugin `dialog`
        fs: Whether to enable the plugin `fs`
    """
    return pytauri_builder_factory(
        opener=opener, clipboard_manager=clipboard_manager, dialog=dialog, fs=fs
    )


def context_factory(
    src_tauri_dir: Path, /, *, tauri_config: Optional[str] = None
) -> Context:
    """Generate a `Context` based on `tauri.conf.json`, `capabilities/` and etc in the `src_tauri_dir` directory.

    This type-hinted wrapper function for [pytauri.context_factory][].

    Args:
        src_tauri_dir: The **absolute** path of the pytauri project directory.
            In a typical Tauri project, it exists as the `src-tauri` directory;
            in the `pytauri-wheel` project, you only need to provide a similar file layout to `src-tauri`:
                ```text
                src-tauri/
                    __init__.py
                    tauri.conf.json
                    capabilities/
                    ...
                ```
        tauri_config: The config JSON string to be merged with `tauri.conf.json`, equivalent to the `--config` of `tauri-cli`.
            See: <https://tauri.app/develop/configuration-files/#extending-the-configuration>.
    """
    if not src_tauri_dir.is_absolute():
        raise ValueError("`src_tauri_dir` must be an absolute path.")

    return pytauri_context_factory(src_tauri_dir, tauri_config=tauri_config)
