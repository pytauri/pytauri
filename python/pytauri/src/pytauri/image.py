"""[tauri::image](https://docs.rs/tauri/latest/tauri/image/index.html)"""

from io import BytesIO
from os import PathLike
from typing import (
    Union,
    final,
)

from PIL import Image as PILImage
from typing_extensions import Self

from pytauri.ffi.image import Image as _Image

__all__ = [
    "Image",
]


@final
class Image(_Image):
    """[tauri::image::Image](https://docs.rs/tauri/latest/tauri/image/struct.Image.html)"""

    _MODE = "RGBA"
    """Tauri requires images to be in RGBA mode.

    See: <https://docs.rs/tauri/latest/tauri/image/struct.Image.html#method.new_owned>
    """

    @classmethod
    def from_pil(cls, image: PILImage.Image) -> Self:
        """Creates a new image using the provided `PIL` image.

        The original `tauri::image::Image::from_bytes` only supports `.ico` and `.png` formats.
        But this method supports **all formats supported by `PIL`**.

        !!! note
            `Tauri` requires images to be in `RGBA` mode.
            If the provided image is not in `RGBA` mode, it will be converted to `RGBA` mode as a copy.
        """
        if image.mode != cls._MODE:
            image = image.convert(cls._MODE)
        return cls(image.tobytes(), *image.size)

    @classmethod
    def from_bytes(cls, bytes_: Union[bytes, bytearray, memoryview], /) -> Self:
        """Create an image from bytes.

        This method calls [pytauri.image.Image.from_pil][] internally.
        """
        return cls.from_pil(PILImage.open(BytesIO(bytes_)))

    @classmethod
    def from_path(
        cls, path: Union[str, bytes, PathLike[str], PathLike[bytes]], /
    ) -> Self:
        """Create an image from a file path.

        This method calls [pytauri.image.Image.from_pil][] internally.
        """
        return cls.from_pil(PILImage.open(path))
