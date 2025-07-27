from collections.abc import AsyncIterable, AsyncIterator, Coroutine, Generator, Iterator
from concurrent import futures
from contextlib import (
    AbstractAsyncContextManager,
    AbstractContextManager,
    AsyncExitStack,
    ExitStack,
    contextmanager,
)
from enum import Enum, auto
from functools import partial
from threading import get_ident
from typing import Any, Awaitable, Callable, Generic, Optional, Union, cast

import anyio
from anyio import (
    BrokenResourceError,
    CancelScope,
    Event,
    create_memory_object_stream,
    create_task_group,
    create_tcp_listener,
    get_cancelled_exc_class,
    run,
)
from anyio.abc import TaskGroup
from anyio.from_thread import BlockingPortal, start_blocking_portal
from pytauri_utils.async_tools import AsyncTools
from typing_extensions import (
    Concatenate,
    ParamSpec,
    Protocol,
    Self,
    TypeVar,
    TypeVarTuple,
    Unpack,
)

T = TypeVar("T", infer_variance=True)
T1 = TypeVar("T1", infer_variance=True)
Ts = TypeVarTuple("Ts")
P = ParamSpec("P")
P1 = ParamSpec("P1")


class _CancelFutures(Enum):
    NoSet = auto()
    True_ = auto()
    False_ = auto()


class TaskGroupPoolExecutor(AbstractAsyncContextManager["TaskGroupPoolExecutor"]):
    def __init__(self, portal: BlockingPortal) -> None:
        self._portal = portal
        self._task_group: TaskGroup = create_task_group()
        self._event_loop_ident = get_ident()
        self._exit_stack = AsyncExitStack()
        self._cancelled_exc_class = get_cancelled_exc_class()
        self._shutdown = _CancelFutures.NoSet

    async def __aenter__(self) -> Self:
        await self._exit_stack.enter_async_context(self._task_group)
        return self

    async def __aexit__(self, *args: Any) -> Optional[bool]:
        return await self._exit_stack.__aexit__(*args)

    def submit(
        self, fn: Callable[[Unpack[Ts]], Awaitable[T]], *args: Unpack[Ts]
    ) -> futures.Future[T]:
        if self._shutdown is not _CancelFutures.NoSet:
            raise RuntimeError("Already shutdown")

        future = futures.Future[T]()

        async def wrapper() -> None:
            # NOTE: We don't need a lock here, see following comment in `def shutdown`.
            if self._shutdown is _CancelFutures.True_:
                future.cancel()
                return

            if not future.set_running_or_notify_cancel():
                # Future was already cancelled
                return

            try:
                result = await fn(*args)
                future.set_result(result)
            except self._cancelled_exc_class as exc:
                future.set_exception(futures.CancelledError(exc))
                raise
            except BaseException as exc:
                future.set_exception(exc)
                if not isinstance(exc, Exception):
                    raise

        self._task_group.start_soon(wrapper)
        return future

    def submit_stream(self, stream: AsyncIterator[T]) -> Iterator[futures.Future[T]]:
        def _iterator() -> Iterator[futures.Future[T]]:
            while True:
                if get_ident() == self._event_loop_ident:
                    yield self.submit(stream.__anext__)
                else:
                    yield self._portal.start_task_soon(stream.__anext__)

        return _iterator()

    async def shutdown(
        self, wait: bool = True, *, cancel_futures: bool = False
    ) -> None:
        # NOTE: We don't need a lock here because there are no `await` points in this section,
        # which makes it atomic for Python's single-threaded async event loop.
        #
        # Also, trio/anyio does not have an RWLock suitable for this scenario:
        #     <https://github.com/python-trio/trio/issues/528>.
        if self._shutdown is not _CancelFutures.NoSet:
            return
        self._shutdown = (
            _CancelFutures.True_ if cancel_futures else _CancelFutures.False_
        )

        if wait:
            await self._exit_stack.aclose()


class BlockingPortalPoolExecutor(
    AbstractAsyncContextManager["BlockingPortalPoolExecutor"]
):
    def __init__(self, task_group_executor: TaskGroupPoolExecutor) -> None:
        self._task_group_executor = task_group_executor
        self._portal = task_group_executor._portal  # pyright: ignore[reportPrivateUsage]  # pub(crate)

    async def __aenter__(self) -> Self:
        return self

    async def __aexit__(self, *args: Any) -> Optional[bool]:
        return

    def submit(
        self, fn: Callable[[Unpack[Ts]], Awaitable[T]], *args: Unpack[Ts]
    ) -> futures.Future[T]:
        return self._portal.call(self._task_group_executor.submit, fn, *args)

    def submit_stream(self, stream: AsyncIterator[T]) -> Iterator[futures.Future[T]]:
        return self._portal.call(self._task_group_executor.submit_stream, stream)

    def shutdown(self, wait: bool = True, *, cancel_futures: bool = False) -> None:
        return self._portal.call(
            partial(
                self._task_group_executor.shutdown, wait, cancel_futures=cancel_futures
            )
        )


@contextmanager
def start_blocking_portal_executor(
    portal: BlockingPortal,
) -> Generator[BlockingPortalPoolExecutor]:
    with (
        portal.wrap_async_context_manager(
            portal.call(TaskGroupPoolExecutor, portal)
        ) as task_group_executor,
        portal.wrap_async_context_manager(
            portal.call(BlockingPortalPoolExecutor, task_group_executor)
        ) as blocking_portal_executor,
    ):
        yield blocking_portal_executor


class FutureNursery(AbstractAsyncContextManager["FutureNursery"]):
    def __init__(self) -> None:
        self._exit_stack = AsyncExitStack()
        self._portal = BlockingPortal()
        self._task_group: TaskGroup = create_task_group()
        self._event_loop_ident = get_ident()
        self._cancelled_exc_class = get_cancelled_exc_class()

    async def __aenter__(self) -> Self:
        await self._exit_stack.enter_async_context(self._portal)
        await self._exit_stack.enter_async_context(self._task_group)
        return self

    async def __aexit__(self, *args: Any) -> Optional[bool]:
        return await self._exit_stack.__aexit__(*args)

    async def wait(self, future: futures.Future[T]) -> T:
        sender, receiver = create_memory_object_stream[Union[T, BaseException]](1)

        def _callback(future: futures.Future[T]) -> None:
            with CancelScope(shield=True), sender:  # noqa: ASYNC100
                try:
                    if future.cancelled():
                        sender.send_nowait(futures.CancelledError())
                        return

                    exception = future.exception()
                    if exception is not None:
                        sender.send_nowait(exception)
                        return

                    ret = future.result()
                    sender.send_nowait(ret)
                except BrokenResourceError:
                    # maybe cancelled
                    pass

        def callback(future: futures.Future[T]) -> None:
            if get_ident() == self._event_loop_ident:
                _callback(future)
            else:
                # TODO: HOW to ensure `acallback` will not be cancelled in any case?
                self._portal.start_task_soon(_callback, future).add_done_callback(
                    # NOTE: log
                    lambda f: f.result(0)
                )

        future.add_done_callback(callback)

        try:
            with receiver:
                ret = await receiver.receive()
                if isinstance(ret, BaseException):
                    raise ret
                return ret
        except self._cancelled_exc_class:
            future.cancel()
            raise

    async def wait_stream(
        self, stream: Iterator[futures.Future[T]]
    ) -> AsyncIterator[T]:
        while True:
            try:
                future = next(stream)
            except StopIteration:
                return

            try:
                yield await self.wait(future)
            except StopAsyncIteration:
                return


if __name__ == "__main__":

    async def foo() -> int:
        from anyio import sleep

        for i in range(2):
            print(f"Task running iteration {i}")
            await sleep(1)
        # raise ValueError("An error occurred in the task")
        return 42

    with (
        start_blocking_portal(backend="trio") as portal,
        start_blocking_portal_executor(portal) as portal_executor,
    ):
        # future = portal_executor.submit(foo)

        async def main() -> None:
            async with (
                create_task_group() as tg,
                BlockingPortal() as blocking_portal,
                TaskGroupPoolExecutor(blocking_portal) as executor,
                FutureNursery() as future_nursery,
            ):
                # future1 = executor.submit(foo)
                # result = await future_nursery.wait(future1)
                # print(result)

                # result = await future_nursery.wait(future)
                # print("Result from future:", result)

                async def stream_iterator() -> AsyncIterator[int]:
                    from anyio import sleep

                    for i in range(3):
                        yield i
                        await sleep(0.001)

                stream = executor.submit_stream(stream_iterator())
                py_stream = future_nursery.wait_stream(stream)

                async for item in py_stream:
                    print("Stream item:", item)

        run(main, backend="trio")

    # from anyio import sleep

    # async def main() -> None:
    #     async with create_task_group() as tg, TaskGroupPoolExecutor(tg) as executor:

    #         async def foo() -> None:
    #             print("Task group started, ready to accept tasks")

    #         print("1")
    #         tg.cancel_scope.cancel()
    #         print("2")
    #         tg.start_soon(foo)
    #         print("3")
    #         await sleep(1)
    #         print("4")

    # run(main, backend="trio")
