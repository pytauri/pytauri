"""PyTauri precompiled wheels."""

from pathlib import Path
from typing import Optional

from pytauri import Context
from pytauri import (
    builder_factory as pytauri_builder_factory,
)
from pytauri import (
    context_factory as pytauri_context_factory,
)

__all__ = ["builder_factory", "context_factory"]

builder_factory = pytauri_builder_factory


def context_factory(
    src_tauri_dir: Path, /, *, tauri_config: Optional[str] = None
) -> Context:
    """Generate a `Context` from `tauri.conf.json` etc in the `src_tauri_dir` directory.

    Args:
        src_tauri_dir: The **absolute** path of the pytauri project directory.
            Usually it is the `src-tauri` directory.
        tauri_config: The config JSON string to be merged, equivalent to the `--config` of `tauri-cli`.
            See: <https://tauri.app/develop/configuration-files/#extending-the-configuration>.
    """
    if not src_tauri_dir.is_absolute():
        raise ValueError("`src_tauri_dir` must be an absolute path.")

    return pytauri_context_factory(src_tauri_dir, tauri_config=tauri_config)
