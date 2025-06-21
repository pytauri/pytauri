from functools import cache
from typing import Any, Optional, cast

from pydantic import BaseModel, RootModel

__all__ = ["to_model"]


def to_model(type_: Any) -> Optional[type[RootModel[Any]]]:
    if type_ is bytes:
        return
    if isinstance(type_, type) and issubclass(type_, BaseModel):
        return

    type_ = cast(Any, type_)  # make pyright happy
    return _to_model_cache(type_)


@cache
def _to_model_cache(type_: Any) -> type[RootModel[Any]]:
    return RootModel[type_]
