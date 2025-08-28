from os import PathLike
from pathlib import Path
from typing import Union

from typing_extensions import TypeAliasType, TypeVar

__all__ = ["PySerdeFrom", "PySerdeInto", "Pyo3PathFrom", "Pyo3PathInto"]

_T = TypeVar("_T", infer_variance=True)


# NOTE: only `PathLike[str]`, not `PathLike[bytes]`,
# ref: <https://github.com/PyO3/pyo3/blob/daa3ee7897011c95775ce7617b4c1c7cadd8db49/src/conversions/std/path.rs#L17>
Pyo3PathFrom = TypeAliasType("Pyo3PathFrom", Union[str, PathLike[str], Path])
"""The input type of `std::path::PathBuf`"""
Pyo3PathInto = TypeAliasType("Pyo3PathInto", Path)
"""The return type of `std::path::PathBuf`"""

# TODO: move these to `pyo3-utils` package
PySerdeFrom = TypeAliasType("PySerdeFrom", Union[str, bytes, _T], type_params=(_T,))
"""If `str` or `bytes` is provided, it will be JSON-deserialized into object `T`."""
PySerdeInto = TypeAliasType("PySerdeInto", _T, type_params=(_T,))
"""The symmetric type of `PySerdeFrom` (i.e., the `T` in `PySerdeFrom[T]`)."""
