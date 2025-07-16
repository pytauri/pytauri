from collections.abc import Awaitable, Coroutine
from contextlib import ExitStack
from functools import partial, wraps
from typing import Any, Callable, Optional

from anyio import (
    create_task_group,
)
from anyio.abc import TaskGroup
from anyio.from_thread import BlockingPortal
from anyio.to_thread import run_sync
from typing_extensions import ParamSpec, Self, TypeVar

__all__ = ["AsyncTools"]

_P = ParamSpec("_P")
_T = TypeVar("_T", infer_variance=True)


class AsyncTools:
    portal: BlockingPortal
    task_group: TaskGroup

    def __init__(self, portal: BlockingPortal):
        self.portal = portal
        self._exit_stack = ExitStack()

    def __enter__(self) -> Self:
        portal = self.portal

        self.task_group = self._exit_stack.enter_context(
            portal.wrap_async_context_manager(portal.call(create_task_group))
        )
        return self

    def __exit__(self, *args: Any) -> Optional[None]:
        self._exit_stack.__exit__(*args)

    def to_sync(self, cb: Callable[_P, Awaitable[_T]]) -> Callable[_P, _T]:
        @wraps(cb)
        def wrapped(*args: _P.args, **kwargs: _P.kwargs) -> _T:
            # why need `partial`?
            # see: <https://github.com/python-trio/trio/issues/470>.
            return self.portal.start_task_soon(partial(cb, *args, **kwargs)).result()

        return wrapped

    def to_async(self, cb: Callable[_P, _T]) -> Callable[_P, Coroutine[None, None, _T]]:
        @wraps(cb)
        async def wrapped(*args: _P.args, **kwargs: _P.kwargs) -> _T:
            return await run_sync(partial(cb, *args, **kwargs))

        return wrapped
