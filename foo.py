from typing import Any, Awaitable, Callable, Generic, Union, cast

from anyio import (
    TASK_STATUS_IGNORED,
    Event,
    connect_tcp,
    create_memory_object_stream,
    create_task_group,
    create_tcp_listener,
    run,
)
from anyio.abc import TaskStatus
from pytauri_utils.async_tools import AsyncTools
from typing_extensions import (
    Concatenate,
    ParamSpec,
    Protocol,
    TypeVar,
    TypeVarTuple,
    Unpack,
)

T = TypeVar("T", infer_variance=True)
T1 = TypeVar("T1", infer_variance=True)
Ts = TypeVarTuple("Ts")
P = ParamSpec("P")
P1 = ParamSpec("P1")


Exception_T = TypeVar(
    "Exception_T", bound=BaseException, infer_variance=True, default=BaseException
)

Catch = Callable[[Exception_T], None]

Then = Callable[[T], None]


async_tools = cast(AsyncTools, ...)


async def callback_tool(
    func: Callable[[Unpack[Ts], Then[T], Catch], None], *args: Unpack[Ts]
) -> T:
    send, recv = create_memory_object_stream[Union[T, BaseException]](max_buffer_size=0)

    @async_tools.to_sync
    async def then(ret: T) -> None:
        async with send:
            send.send_nowait(ret)

    @async_tools.to_sync
    async def catch(e: BaseException) -> None:
        async with send:
            send.send_nowait(e)

    func(*args, then, catch)

    async with recv:
        ret = await recv.receive()

    if isinstance(ret, BaseException):
        raise ret
    return ret


def chenge(
    func: Callable[[Unpack[Ts], Then[T], Catch], None],
) -> Callable[[Unpack[Ts]], Awaitable[T]]:
    return lambda *args: callback_tool(func, *args)


def foo(
    a: int, then: Callable[[str], None], catch: Callable[[Exception], None]
) -> None: ...


class Foo:
    def foo(
        self, a: int, then: Callable[[str], None], catch: Callable[[Exception], None]
    ) -> None: ...


foo1_0 = chenge(foo)


async def main() -> None:
    foo_instance = Foo()
    a = await callback_tool(foo, 1)
    b = await callback_tool(foo_instance.foo, 1)

from concurrent.futures import Future
from anyio.from_thread import start_blocking_portal