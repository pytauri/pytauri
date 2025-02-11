# ruff: noqa: D102

"""[tauri::image](https://docs.rs/tauri/latest/tauri/image/index.html)"""

from typing import (
    TYPE_CHECKING,
)

from typing_extensions import Self

from pytauri.ffi._ext_mod import pytauri_mod

__all__ = [
    "Image",
]

_image_mod = pytauri_mod.image

if TYPE_CHECKING:

    class Image:
        """[tauri::image::Image](https://docs.rs/tauri/latest/tauri/image/struct.Image.html)"""

        def __new__(cls, rgba: bytes, width: int, height: int, /) -> Self: ...
        @property
        def rgba(self) -> bytes: ...
        @property
        def width(self) -> int: ...
        @property
        def height(self) -> int: ...

else:
    Image = _image_mod.Image
