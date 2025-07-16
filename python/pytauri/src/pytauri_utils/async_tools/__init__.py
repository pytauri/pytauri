"""This module provides a set of asynchronous utilities for pytauri."""

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
    """A utility class for asynchronous tasks in pytauri.

    # Example:

    ```python
    from anyio.from_thread import start_blocking_portal

    with (
        start_blocking_portal("asyncio") as portal,  # or `trio`
        AsyncTools(portal) as async_tools,
    ):
        pass
    ```
    """

    portal: BlockingPortal
    """The `BlockingPortal` that you pass to the `AsyncTools` constructor.

    You can use it **in another thread** to interact with the asynchronous event loop
    (because the asynchronous event loop is not thread-safe).
    But usually you prefer to use the `AsyncTools.to_sync` or `AsyncTools.to_async`.

    Thread safety references:
       - <https://anyio.readthedocs.io/en/stable/threads.html>
       - <https://docs.python.org/3/library/asyncio-task.html#asyncio.run_coroutine_threadsafe>

    """
    task_group: TaskGroup
    """A `TaskGroup` instantiated in the `portal` context.

    You can use it **in the asynchronous event loop** to spawn a background task.

    Note:
        You can NOT enter/exit the `task_group` context manager directly,
        its lifecycle is managed by the `AsyncTools` context manager.

    # Example:

    ```python
    async def command_handler(async_tools: AsyncTools) -> None:
        async_tools.task_group.start_soon(...)  # run in the background
        return
    ```
    """

    def __init__(self, portal: BlockingPortal):
        """Initialize the AsyncTools with a BlockingPortal.

        Args:
            portal:
                The portal to use for asynchronous operations.

                !!! Note
                    You need to ensure that the lifecycle of `portal` is longer than
                    the lifecycle of this `AsyncTools` instance.
                    I.e., You cannot exit the `portal`'s context manager before exiting the
                    `AsyncTools` instance.
        """
        self.portal = portal
        self._exit_stack = ExitStack()

    def __enter__(self) -> Self:
        """Enter and initialize the context of this `AsyncTools`.

        Note:
            Most APIs are only available after calling this method.
        """
        portal = self.portal

        self.task_group = self._exit_stack.enter_context(
            portal.wrap_async_context_manager(portal.call(create_task_group))
        )
        return self

    def __exit__(self, *args: Any) -> Optional[None]:
        """Exit and finalize the context of this `AsyncTools`.

        Note:
            After exiting this context, you cannot use the `AsyncTools` instance anymore.
            And this will not exit the `portal` context manager (you should do it yourself).
        """
        self._exit_stack.__exit__(*args)

    def to_sync(self, cb: Callable[_P, Awaitable[_T]]) -> Callable[_P, _T]:
        """Convert an asynchronous callable to a synchronous one.

        This is useful when you need to call an async function in a synchronous context,
        such as a Tauri callback handler.

        Note:
            The returned function can NOT be called in the thread (asynchronous event loop)
            where the `portal` is located.

        # Example:

        ```python
        from anyio import create_memory_object_stream
        from pytauri import AppHandle, Event, Listener


        async def command_handler(
            app_handle: AppHandle, async_tools: AsyncTools
        ) -> None:
            send_stream, receive_stream = create_memory_object_stream[str]()

            @async_tools.to_sync
            async def listener(event: Event) -> None:
                async with send_stream:
                    await send_stream.send(event.payload)

            Listener.once(app_handle, "foo-event", listener)

            async with receive_stream:
                print("Received: ", await receive_stream.receive())
        ```
        """

        @wraps(cb)
        def wrapped(*args: _P.args, **kwargs: _P.kwargs) -> _T:
            # Why need `partial`?
            # see: <https://github.com/python-trio/trio/issues/470>.
            #
            # PERF: maybe we should we `lambda` instead of `partial`? And cache?
            return self.portal.start_task_soon(partial(cb, *args, **kwargs)).result()

        return wrapped

    def to_async(self, cb: Callable[_P, _T]) -> Callable[_P, Coroutine[None, None, _T]]:
        """Convert a synchronous callable to an asynchronous one.

        This is useful when you want to run IO/CPU-bound synchronous code
        in an asynchronous context.

        IO/CPU-bound references:
            - <https://anyio.readthedocs.io/en/stable/threads.html#working-with-threads>
            - <https://docs.python.org/3/library/asyncio-task.html#asyncio.to_thread>

        Note:
            The returned function can only be called in the thread (asynchronous event loop)
            where the `portal` is located, but it will run the synchronous code in a separate thread.

        # Example:

        ```python
        from time import sleep


        async def command_handler(async_tools: AsyncTools) -> None:
            await async_tools.to_async(sleep)(1.0)  # Run sleep in a separate thread
        ```
        """

        @wraps(cb)
        async def wrapped(*args: _P.args, **kwargs: _P.kwargs) -> _T:
            return await run_sync(partial(cb, *args, **kwargs))

        return wrapped
