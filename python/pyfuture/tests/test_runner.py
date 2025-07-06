# TODO:
# ruff: noqa: D100, D103

from collections.abc import Awaitable
from typing import Optional

import pytest
from anyio import Event
from pyfuture import (
    RunnerBuilder,
    _PyRunnerProto,  # pyright: ignore[reportPrivateUsage]
)
from typing_extensions import Self


@pytest.mark.anyio
async def test_runner_builder() -> None:
    class Result:
        pass

    mock_result = Result()

    class MockPyFuture:
        result: Optional[Result] = None

        @property
        def awaitable(self) -> Awaitable[Result]:
            async def _awaitable() -> Result:
                return mock_result

            return _awaitable()

        def set_result(self, result: Result) -> None:
            self.result = result
            self.is_done.set()

        def set_exception(self, _exception: BaseException) -> None:
            self.is_done.set()
            raise NotImplementedError()  # TODO: test awaitable exception

        def __init__(self) -> None:
            self.is_done = Event()

    class MockRunner:
        closed = False

        def __new__(cls, _py_runner: _PyRunnerProto, /) -> Self:
            return super().__new__(cls)

        def __init__(self, py_runner: _PyRunnerProto, /) -> None:
            self.py_runner = py_runner

        def close(self) -> None:
            self.closed = True

    async def main():
        runner = None
        pyfuture = None
        async with RunnerBuilder() as builder:
            runner = builder.build(MockRunner)

            # running test
            pyfuture = MockPyFuture()
            runner.py_runner(pyfuture)

            # wait for `pyfuture.awaitable` done before exiting `RunnerBuilder`,
            # or exiting `RunnerBuilder` will close the runner,
            # then the `awaitable` will be cancelled.
            await pyfuture.is_done.wait()

        assert runner, "`builder.build` error"
        assert runner.closed, "`RunnerBuilder` didn't call `runner.close`"

        # check result
        assert pyfuture, "unreachable"
        assert pyfuture.result is mock_result, (
            "runner didn't call `py_future.set_result`"
        )

    await main()
