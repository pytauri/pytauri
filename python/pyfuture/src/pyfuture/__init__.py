# TODO:
# ruff: noqa: D104, D101, D107, D105

import threading
from collections.abc import Awaitable
from concurrent.futures import CancelledError, Future
from enum import Enum, auto
from functools import partial
from typing import Callable, Union

from anyio import get_cancelled_exc_class
from anyio.abc import TaskGroup
from anyio.from_thread import BlockingPortal
from typing_extensions import ParamSpec, TypeVar, TypeVarTuple, Unpack

_P = ParamSpec("_P")
_T_Retval = TypeVar("_T_Retval", infer_variance=True)
_PosArgsT = TypeVarTuple("_PosArgsT")


class _CancelFutures(Enum):
    NoSet = auto()
    True_ = auto()
    False_ = auto()


_CANCELLED_EXC_CLASS = get_cancelled_exc_class()


class BlockingPortalPoolExecutor:
    def __init__(self, portal: BlockingPortal) -> None:
        self._portal = portal
        self._cancel_futures: _CancelFutures = _CancelFutures.NoSet
        self._shutdown_lock = threading.Lock()

    def submit(
        self,
        fn: Callable[_P, Awaitable[_T_Retval]],
        /,
        *args: _P.args,
        **kwargs: _P.kwargs,
    ) -> Future[_T_Retval]:
        if isinstance(executor, BlockingPortal):
            return executor.start_task_soon(fn, *args)

        future = Future[_T_Retval]()

        async def _wrapper() -> None:
            if self._cancel_futures is _CancelFutures.True_:
                future.cancel()
                return

            if not future.set_running_or_notify_cancel():
                return

            try:
                result = await fn(*args)
            except self._cancelled_exc_class as e:
                future.set_exception(CancelledError(e))
                raise
            except BaseException as e:
                future.set_exception(e)
                if not isinstance(e, Exception):
                    raise
            else:
                future.set_result(result)

        executor.start_soon(_wrapper)
        return future

    def shutdown(self, wait: bool = True, *, cancel_futures: bool = False) -> None:
        with self._shutdown_lock:
            if self._cancel_futures is not _CancelFutures.NoSet:
                return

            self._cancel_futures = (
                _CancelFutures.True_ if cancel_futures else _CancelFutures.False_
            )

        self
