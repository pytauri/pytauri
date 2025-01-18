"""[tauri::ipc](https://docs.rs/tauri/latest/tauri/ipc/index.html)"""

from collections import UserDict
from collections.abc import Awaitable
from functools import partial, wraps
from inspect import signature
from logging import getLogger
from typing import (
    Any,
    Callable,
    NamedTuple,
    Optional,
    Union,
    cast,
)

from anyio.from_thread import BlockingPortal
from pydantic import BaseModel, ValidationError
from typing_extensions import TypeVar

from pytauri.ffi.ipc import ArgumentsType, Invoke, InvokeResolver, ParametersType
from pytauri.ffi.lib import (
    AppHandle,
    _InvokeHandlerProto,  # pyright: ignore[reportPrivateUsage]
)
from pytauri.ffi.webview import WebviewWindow

__all__ = [
    "ArgumentsType",
    "Commands",
    "Invoke",
    "InvokeException",
    "InvokeResolver",
    "ParametersType",
]

_logger = getLogger(__name__)

_InvokeResponse = Union[bytes, bytearray]

_PyHandlerType = Callable[..., Awaitable[_InvokeResponse]]

_WrappablePyHandlerType = Callable[..., Awaitable[Union[_InvokeResponse, BaseModel]]]

_WrappablePyHandlerTypeVar = TypeVar(
    "_WrappablePyHandlerTypeVar", bound=_WrappablePyHandlerType, infer_variance=True
)

_RegisterType = Callable[[_WrappablePyHandlerTypeVar], _WrappablePyHandlerTypeVar]


class _PyInvokHandleData(NamedTuple):
    parameters: ParametersType
    handler: _PyHandlerType
    """The `handler` can receive the parameters specified by `parameters`"""


class InvokeException(Exception):  # noqa: N818
    """Indicates that an exception occurred in a `command`. Similar to Rust's `Result::Err`.

    When this exception is raised in a `command`,
    pytauri will return it to the frontend through `Invoke.reject(value)`
    and will not log the exception on the python side.
    """

    value: str
    """The error message that will be returned to the frontend."""

    def __init__(self, value: str) -> None:  # noqa: D107
        self.value = value


class Commands(UserDict[str, _PyInvokHandleData]):
    """This class provides features similar to [tauri::generate_handler](https://docs.rs/tauri/latest/tauri/macro.generate_handler.html).

    Typically, you would use [Commands.command][pytauri.Commands.command] to register a command handler function.
    Then, use [Commands.generate_handler][pytauri.Commands.generate_handler] to get an `invoke_handler`
    for use with [BuilderArgs][pytauri.BuilderArgs].
    """

    def __init__(self) -> None:  # noqa: D107
        super().__init__()

        data = self.data

        async def _async_invoke_handler(invoke: Invoke) -> None:
            # NOTE:
            # - the implementer of this function must not raise exceptions
            # - and must ensure to fulfill `invoke/resolver`
            resolver = None
            try:
                command = invoke.command
                handler_data = data.get(command)
                if handler_data is None:
                    invoke.reject(f"no python handler `{command}` found")
                    return

                parameters = handler_data.parameters
                handler = handler_data.handler

                resolver = invoke.bind_to(parameters)
                if resolver is None:
                    # `invoke` has already been rejected
                    return

                try:
                    resp = await handler(**resolver.arguments)
                    # TODO, PERF: idk if this will block?
                except InvokeException as e:
                    resolver.reject(e.value)
                except Exception as e:
                    # # TODO: Should we return the traceback to the frontend?
                    # # It might leak information.
                    # from traceback import format_exc
                    # resolver.reject(format_exc())
                    _logger.exception(
                        f"invoke_handler {handler}: `{handler.__name__}` raised an exception",
                        exc_info=e,
                    )
                    resolver.reject(repr(e))
                else:
                    resolver.resolve(resp)

            except Exception as e:
                msg = f"{_async_invoke_handler} implementation raised an exception, please report this as a pytauri bug"

                _logger.critical(msg, exc_info=e)
                if resolver is not None:
                    resolver.reject(msg)
                else:
                    invoke.reject(msg)
                raise

        self._async_invoke_handler = _async_invoke_handler

    def generate_handler(self, portal: BlockingPortal, /) -> _InvokeHandlerProto:
        """This method is similar to [tauri::generate_handler](https://docs.rs/tauri/latest/tauri/macro.generate_handler.html).

        You can use this method to get `invoke_handler` for use with [BuilderArgs][pytauri.BuilderArgs].

        Examples:
            ```py
            from anyio.from_thread import start_blocking_portal

            commands = Commands()

            with start_blocking_portal(backend) as portal:
                invoke_handler = commands.generate_handler(portal)
                ...
            ```

        !!! warning
            The `portal` must remain valid while the returned `invoke_handler` is being used.
        """
        async_invoke_handler = self._async_invoke_handler

        def invoke_handler(invoke: Invoke) -> None:
            # NOTE:
            # - `invoke_handler` must not raise exception
            # - must not block

            # this func will be call in extern thread, so it's ok to use `start_task_soon`
            portal.start_task_soon(async_invoke_handler, invoke)

        return invoke_handler

    @staticmethod
    def wrap_pyfunc(  # noqa: C901  # TODO: simplify the method
        pyfunc: _WrappablePyHandlerType,
    ) -> _PyHandlerType:
        """Wrap a `Callable` to conform to the definition of PyHandlerType.

        Specifically:

        - If `pyfunc` has a `KEYWORD_ONLY` parameter named `body`, will check if `issubclass(body, BaseModel)` is true,
          and if so, wrap it as a new function with `body: bytearray` parameter.
        - If `pyfunc` conforms to `issubclass(return_annotation, BaseModel)`,
          wrap it as a new function with `return_annotation: bytes` return type.
        - If not, will return the original `pyfunc`.

        The `pyfunc` will be decorated using [functools.wraps][], and its `__signature__` will also be updated.
        """
        serializer = None
        deserializer = None

        body_key = "body"

        sig = signature(pyfunc)
        parameters = sig.parameters
        return_annotation = sig.return_annotation

        body_param = parameters.get(body_key)
        if body_param is not None:
            if body_param.kind not in {
                body_param.KEYWORD_ONLY,
                body_param.POSITIONAL_OR_KEYWORD,
            }:
                raise ValueError(f"Expected `{body_key}` to be KEYWORD_ONLY")
            body_type = body_param.annotation
            if issubclass(body_type, BaseModel):
                serializer = body_type.model_validate_json
            else:
                if not issubclass(body_type, bytearray):
                    raise ValueError(
                        f"Expected `{body_key}` to be subclass of {BaseModel} or {bytearray}, "
                        f"got {body_type}"
                    )

        if issubclass(return_annotation, BaseModel):
            deserializer = return_annotation.__pydantic_serializer__.to_json
        else:
            if not issubclass(return_annotation, (bytes, bytearray)):
                raise ValueError(
                    f"Expected `return_annotation` to be subclass of {BaseModel}, {bytes} or {bytearray}, "
                    f"got {return_annotation}"
                )

        if not serializer and not deserializer:
            return cast(_PyHandlerType, pyfunc)  # `cast` make typing happy

        @wraps(pyfunc)
        async def wrapper(*args: Any, **kwargs: Any) -> bytes:
            nonlocal serializer, deserializer

            if serializer is not None:
                body_bytearray = kwargs[body_key]
                assert isinstance(body_bytearray, bytearray)  # PERF
                try:
                    body_model = serializer(body_bytearray)
                except ValidationError as e:
                    raise InvokeException(str(e)) from e
                kwargs[body_key] = body_model

            resp = await pyfunc(*args, **kwargs)

            if deserializer is not None:
                assert isinstance(resp, BaseModel)  # PERF
                return deserializer(resp)
            else:
                assert isinstance(resp, bytes)  # PERF
                return resp

        new_parameters = None
        if serializer is not None:
            new_parameters = parameters.copy()
            new_parameters[body_key] = parameters[body_key].replace(
                annotation=bytearray
            )

        # see: <https://docs.python.org/3.13/library/inspect.html#inspect.signature>
        wrapper.__signature__ = (  # pyright: ignore[reportAttributeAccessIssue]
            sig.replace(
                parameters=list(new_parameters.values()),
                return_annotation=bytes,
            )
            if new_parameters
            else sig.replace(return_annotation=bytes)
        )
        return wrapper

    @staticmethod
    def parse_parameters(
        pyfunc: _PyHandlerType, /, check_signature: bool = True
    ) -> ParametersType:
        """Check the signature of a `Callable` and return the parameters.

        Check if the [Signature][inspect.Signature] of `pyfunc` conforms to [ArgumentsType][pytauri.ipc.ArgumentsType],
        and if the return value is a subclass of [bytes][] or [bytearray][].

        Args:
            pyfunc: The `Callable` to check.
            check_signature: Whether to check the signature of `pyfunc`.
                Set it to `False` only if you are sure that the signature conforms to the expected pattern.

        Returns:
            The parameters of the `pyfunc`. You can use it with [Invoke.bind_to][pytauri.ipc.Invoke.bind_to].

        Raises:
            ValueError: If the signature does not conform to the expected pattern.
        """
        sig = signature(pyfunc)
        parameters = sig.parameters
        if not check_signature:
            # `cast` make typing happy
            return cast(ParametersType, parameters)

        return_annotation = sig.return_annotation

        arguments_type = {
            "body": bytearray,
            "app_handle": AppHandle,
            "webview_window": WebviewWindow,
        }

        for name, param in parameters.items():
            # check if the `parameters` type hint conforms to [pytauri.ipc.ArgumentsType][]

            correct_anna = arguments_type.get(name)
            if correct_anna is None:
                raise ValueError(
                    f"Unexpected parameter `{name}`, expected one of {list(arguments_type.keys())}"
                )
            if not issubclass(param.annotation, correct_anna):
                raise ValueError(
                    f"Expected `{name}` to be subclass of `{correct_anna}`, got `{param.annotation}`"
                )
        else:
            # after checking, we make sure the `parameters` are valid
            parameters = cast(ParametersType, parameters)

        if not issubclass(return_annotation, (bytes, bytearray)):
            raise ValueError(
                f"Expected return_annotation to be subclass of {bytes} or {bytearray}, got `{return_annotation}`"
            )

        return parameters

    def set_command(
        self,
        command: str,
        handler: _WrappablePyHandlerType,
        /,
        check_signature: bool = True,
    ) -> None:
        """Set a command handler.

        This method internally calls [parse_parameters][pytauri.Commands.parse_parameters]
        and [wrap_pyfunc][pytauri.Commands.wrap_pyfunc], `parse_parameters(wrap_pyfunc(handler))`.
        """
        new_handler = self.wrap_pyfunc(handler)
        parameters = self.parse_parameters(new_handler, check_signature=check_signature)
        self.data[command] = _PyInvokHandleData(parameters, new_handler)

    def _register(
        self,
        handler: _WrappablePyHandlerTypeVar,
        /,
        *,
        command: Optional[str] = None,
    ) -> _WrappablePyHandlerTypeVar:
        command = command or handler.__name__
        if command in self.data:
            raise ValueError(
                f"Command `{command}` already exists. If it's expected, use `set_command` instead."
            )

        self.set_command(command, handler, check_signature=True)
        return handler

    def command(
        self, command: Optional[str] = None, /
    ) -> _RegisterType[_WrappablePyHandlerTypeVar]:
        """A [decorator](https://docs.python.org/3/glossary.html#term-decorator) to register a command handler.

        Examples:
            ```py
            commands = Commands()


            @commands.command()
            async def my_command(body: FooModel, app_handle: AppHandle) -> BarModel: ...


            @commands.command("foo_command")
            async def my_command2(body: FooModel, app_handle: AppHandle) -> BarModel: ...
            ```

        This method internally calls [set_command][pytauri.Commands.set_command],
        which means the function signature must conform to [ArgumentsType][pytauri.ipc.ArgumentsType].

        Args:
            command: The name of the command. If not provided, the `__name__` of `callable` will be used.

        Raises:
            ValueError: If a command with the same name already exists.
                If it's expected, use [set_command][pytauri.Commands.set_command] instead.
        """
        if command is None:
            return self._register
        else:
            return partial(self._register, command=command)
