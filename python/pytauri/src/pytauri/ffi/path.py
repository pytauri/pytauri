# ruff: noqa: D102

"""[tauri::path](https://docs.rs/tauri/latest/tauri/path/index.html)"""

from typing import (
    TYPE_CHECKING,
    final,
)

from pytauri.ffi._ext_mod import pytauri_mod

__all__ = ["PathResolver"]

_path_mod = pytauri_mod.path

if TYPE_CHECKING:

    @final
    class PathResolver:
        """[tauri::path::PathResolver](https://docs.rs/tauri/latest/tauri/path/struct.PathResolver.html)"""

        def audio_dir(self) -> str: ...
        def cache_dir(self) -> str: ...
        def config_dir(self) -> str: ...
        def data_dir(self) -> str: ...
        def local_data_dir(self) -> str: ...
        def desktop_dir(self) -> str: ...
        def document_dir(self) -> str: ...
        def download_dir(self) -> str: ...
        def executable_dir(self) -> str: ...
        def font_dir(self) -> str: ...
        def home_dir(self) -> str: ...
        def picture_dir(self) -> str: ...
        def public_dir(self) -> str: ...
        def runtime_dir(self) -> str: ...
        def template_dir(self) -> str: ...
        def video_dir(self) -> str: ...
        def resource_dir(self) -> str: ...
        def app_config_dir(self) -> str: ...
        def app_data_dir(self) -> str: ...
        def app_local_data_dir(self) -> str: ...
        def app_cache_dir(self) -> str: ...
        def app_log_dir(self) -> str: ...
        def temp_dir(self) -> str: ...

else:
    PathResolver = _path_mod.PathResolver
