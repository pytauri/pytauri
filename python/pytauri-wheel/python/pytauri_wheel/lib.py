"""PyTauri precompiled wheels.

# Usage

`pytauri-wheel` provides precompiled [pytauri.EXT_MOD][] for [pytauri][],
which means you can normally use the `pytauri` API, except for the following APIs provided by `pytauri-wheel`:

- [pytauri.builder_factory][] -> [pytauri_wheel.lib.builder_factory][]
- [pytauri.context_factory][] -> [pytauri_wheel.lib.context_factory][]
"""

from collections.abc import Mapping
from pathlib import Path
from typing import Any, Optional, Union

from pytauri import Builder, Context
from pytauri import (
    builder_factory as pytauri_builder_factory,
)
from pytauri import (
    context_factory as pytauri_context_factory,
)
from typing_extensions import TypeAliasType, TypeVar

__all__ = ["builder_factory", "context_factory"]


_T = TypeVar("_T", infer_variance=True)

# TODO: move this to `pytauri.ffi._typing`
_PySerdeFrom = TypeAliasType("_PySerdeFrom", Union[str, bytes, _T], type_params=(_T,))
"""If `str` or `bytes` is provided, it will be JSON-deserialized into object `T`."""

# TODO: move this to `pytauri.ffi.lib`
_ConfigFrom = TypeAliasType("_ConfigFrom", Mapping[str, Any])
"""[tauri::Config](https://docs.rs/tauri/latest/tauri/struct.Config.html)"""


def builder_factory() -> Builder:
    """A factory function for creating a [pytauri.Builder][] instance.

    This is a type-hinted wrapper function for [pytauri.builder_factory][].
    """
    return pytauri_builder_factory()


def context_factory(
    src_tauri_dir: Path,
    /,
    *,
    tauri_config: Optional[_PySerdeFrom[_ConfigFrom]] = None,
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
        tauri_config: The config to be merged with `tauri.conf.json`, equivalent to the `--config` of `tauri-cli`.
            See: <https://tauri.app/develop/configuration-files/#extending-the-configuration>.

            If `str` or `bytes` is provided, it will be deserialized into a JSON object.

            ```python
            tauri_config = {
                "build": {
                    "frontendDist": "http://localhost:1420",
                },
            }

            # Or

            tauri_config = json.dumps(
                {
                    "build": {
                        "frontendDist": "http://localhost:1420",
                    },
                }
            )
            ```
    """
    if not src_tauri_dir.is_absolute():
        raise ValueError("`src_tauri_dir` must be an absolute path.")

    return pytauri_context_factory(src_tauri_dir, tauri_config=tauri_config)
