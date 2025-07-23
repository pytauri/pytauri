from collections.abc import Coroutine
from concurrent import futures
from contextlib import (
    AbstractAsyncContextManager,
    AbstractContextManager,
    AsyncExitStack,
    ExitStack,
)
from functools import partial
from threading import get_ident
from typing import Any, Awaitable, Callable, Generic, Optional, Union, cast

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


class BlockingPortalPoolExecutor(AbstractContextManager["BlockingPortalPoolExecutor"]):
    def __init__(self, portal: BlockingPortal) -> None:
        self._portal = portal
        self._exit_stack = ExitStack()

    def __enter__(self) -> Self:
        portal = self._portal
        exit_stack = self._exit_stack

        self._cancelled_exc_class = portal.call(get_cancelled_exc_class)
        self._task_group: TaskGroup = exit_stack.enter_context(
            portal.wrap_async_context_manager(portal.call(create_task_group))
        )

        return self

    def __exit__(self, *args: Any) -> Optional[bool]:
        return self._exit_stack.__exit__(*args)

    def submit(
        self, fn: Callable[P, Awaitable[T]], *args: P.args, **kwargs: P.kwargs
    ) -> futures.Future[T]:
        future = futures.Future[T]()

        async def task() -> None:
            async def wrapper() -> None:
                try:
                    result = await fn(*args, **kwargs)
                    future.set_result(result)
                except self._cancelled_exc_class as exc:
                    future.set_exception(futures.CancelledError(exc))
                    raise
                except BaseException as exc:
                    future.set_exception(exc)
                    if not isinstance(exc, Exception):
                        raise

            self._task_group.start_soon(wrapper)

        self._portal.start_task_soon(task).add_done_callback(
            # NOTE: log
            lambda f: f.result(0)
        )
        return future

    def shutdown(self, wait: bool = True) -> None:
        portal = self._portal
        exit_stack = self._exit_stack

        if wait:
            portal.call(exit_stack.close)
        else:
            portal.start_task_soon(exit_stack.close).add_done_callback(
                # NOTE: log
                lambda f: f.result(0)
            )


class TaskGroupPoolExecutor(AbstractAsyncContextManager["TaskGroupPoolExecutor"]):
    def __init__(self, task_group: TaskGroup) -> None:
        self._outer_task_group = task_group
        self._task_group = create_task_group()
        self._exit_stack = AsyncExitStack()
        self._cancelled_exc_class = get_cancelled_exc_class()

    async def __aenter__(self) -> Self:
        await self._exit_stack.enter_async_context(self._task_group)
        return self

    async def __aexit__(self, *args: Any) -> Optional[bool]:
        return await self._exit_stack.__aexit__(*args)

    async def submit(
        self, fn: Callable[P, Awaitable[T]], *args: P.args, **kwargs: P.kwargs
    ) -> futures.Future[T]:
        future = futures.Future[T]()

        async def wrapper() -> None:
            try:
                result = await fn(*args, **kwargs)
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

    async def shutdown(self, wait: bool = True) -> None:
        exit_stack = self._exit_stack

        if wait:
            await exit_stack.aclose()
        else:
            self._outer_task_group.start_soon(exit_stack.aclose)


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


# def future_to_coroutine(
#     future: futures.Future[T],
#     executor: Union[BlockingPortal, TaskGroup],
# ) -> Coroutine[None, None, T]:
#     sender, receiver = create_memory_object_stream[Union[T, BaseException]](1)

#     async def acallback(future: futures.Future[T]) -> None:
#         with CancelScope(shield=True), sender:  # noqa: ASYNC100
#             if future.cancelled():
#                 sender.send_nowait(get_cancelled_exc_class()())
#                 return

#             exception = future.exception()
#             if exception is not None:
#                 sender.send_nowait(exception)
#                 return

#             ret = future.result()
#             sender.send_nowait(ret)

#     def callback(future: futures.Future[T]) -> None:
#         # TODO: HOW to ensure `acallback` will not be cancelled in any case?
#         if isinstance(executor, BlockingPortal):
#             executor.start_task_soon(acallback, future).add_done_callback(
#                 # NOTE: log
#                 lambda f: f.result(0)
#             )
#         else:
#             executor.start_soon(acallback, future)

#     future.add_done_callback(callback)

#     async def wait_for_future() -> T:
#         with receiver:
#             ret = await receiver.receive()
#             if isinstance(ret, BaseException):
#                 raise ret
#             return ret

#     return wait_for_future()


async def future_to_coroutine(
    future: futures.Future[T],
    executor: Union[BlockingPortal, TaskGroup],
) -> T:
    event_loop_ident = get_ident()
    sender, receiver = create_memory_object_stream[Union[T, BaseException]](1)

    def _callback(future: futures.Future[T]) -> None:
        with CancelScope(shield=True), sender:  # noqa: ASYNC100
            if future.cancelled():
                sender.send_nowait(get_cancelled_exc_class()())
                return

            exception = future.exception()
            if exception is not None:
                sender.send_nowait(exception)
                return

            ret = future.result()
            sender.send_nowait(ret)

    async def _async_callback(future: futures.Future[T]) -> None:
        _callback(future)

    def callback(future: futures.Future[T]) -> None:
        if get_ident() == event_loop_ident:
            _callback(future)
        else:
            if isinstance(executor, BlockingPortal):
                # TODO: HOW to ensure `acallback` will not be cancelled in any case?
                executor.start_task_soon(_callback, future).add_done_callback(
                    # NOTE: log
                    lambda f: f.result(0)
                )
            else:
                executor.start_soon(_async_callback, future)

    future.add_done_callback(callback)

    with receiver:
        ret = await receiver.receive()
        if isinstance(ret, BaseException):
            raise ret
        return ret


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
        BlockingPortalPoolExecutor(portal) as portal_executor,
    ):
        future = portal_executor.submit(foo)

        async def main() -> None:
            async with (
                create_task_group() as tg,
                TaskGroupPoolExecutor(tg) as executor,
                BlockingPortal() as blocking_portal,
                FutureNursery() as future_nursery,
            ):
                future1 = await executor.submit(foo)
                result = await future_nursery.wait(future1)
                print(result)

                result = await future_nursery.wait(future)
                print("Result from future:", result)

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
