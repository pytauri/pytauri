from functools import cache
from typing import Any, Optional, cast

from pydantic import BaseModel, TypeAdapter

__all__ = ["to_type_adapter"]


def to_type_adapter(type_: Any) -> Optional[TypeAdapter[Any]]:
    if type_ is bytes:
        return
    if isinstance(type_, type) and issubclass(type_, BaseModel):
        return

    type_ = cast(Any, type_)  # make pyright happy
    return _to_type_adapter(type_)


@cache
def _to_type_adapter(type_: Any) -> TypeAdapter[Any]:
    return TypeAdapter(type_)
