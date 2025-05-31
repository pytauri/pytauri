from os import PathLike
from pathlib import Path
from typing import Union

from typing_extensions import TypeAlias

__all__ = ["Pyo3Path", "StrictPyo3Path"]

# NOTE: only `PathLike[str]`, not `PathLike[bytes]`,
# ref: <https://github.com/PyO3/pyo3/blob/daa3ee7897011c95775ce7617b4c1c7cadd8db49/src/conversions/std/path.rs#L17>
Pyo3Path: TypeAlias = Union[str, PathLike[str], Path]
"""The input type of `std::path::PathBuf`"""

StrictPyo3Path: TypeAlias = Path
"""The return type of `std::path::PathBuf`"""
