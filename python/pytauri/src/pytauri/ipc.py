"""[tauri::ipc](https://docs.rs/tauri/latest/tauri/ipc/index.html)"""

from collections import UserDict
from collections.abc import Awaitable
from functools import cache, partial, wraps
from inspect import Parameter, Signature, signature
from logging import getLogger
from os import PathLike
from typing import (
    Annotated,
    Any,
    Callable,
    Generic,
    Optional,
    Union,
    cast,
)
from warnings import warn

from anyio.from_thread import BlockingPortal
from pydantic import (
    BaseModel,
    GetPydanticSchema,
    RootModel,
    ValidationError,
)
from pydantic.alias_generators import to_camel
from pydantic_core.core_schema import (
    any_schema,
    chain_schema,
    json_or_python_schema,
    no_info_plain_validator_function,
    str_schema,
)
from typing_extensions import NamedTuple, Self, TypeVar, overload

from pytauri._gen_ts import CommandInputOutput, InputOutput, NoneType, gen_ts
from pytauri.ffi.ipc import (
    ArgumentsType,
    Headers,
    Invoke,
    InvokeResolver,
    ParametersType,
    _InvokeResponseBody,  # pyright: ignore[reportPrivateUsage]
)
from pytauri.ffi.ipc import Channel as _FFIChannel
from pytauri.ffi.ipc import JavaScriptChannelId as _FFIJavaScriptChannelId
from pytauri.ffi.lib import (
    AppHandle,
    _InvokeHandlerProto,  # pyright: ignore[reportPrivateUsage]
)
from pytauri.ffi.webview import Webview, WebviewWindow

__all__ = [
    "ArgumentsType",
    "Channel",
    "Commands",
    "Headers",
    "Invoke",
    "InvokeException",
    "InvokeResolver",
    "JavaScriptChannelId",
    "ParametersType",
]

_logger = getLogger(__name__)

_PyHandlerType = Callable[..., Awaitable[_InvokeResponseBody]]

_WrappablePyHandlerType = Callable[..., Awaitable[Union[bytes, BaseModel, Any]]]

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


_T = TypeVar("_T", infer_variance=True)

_T_model = TypeVar("_T_model", bound=BaseModel, infer_variance=True)

_Serializer = Callable[[bytes], _T]

_Deserializer = Callable[[_T], str]


class _ModelSerde(NamedTuple, Generic[_T, _T_model]):
    model: type[_T_model]
    serializer: _Serializer[_T]
    deserializer: _Deserializer[_T]


# This second overload is for unsupported special forms (such as Annotated, Union, None, etc.)
# Currently there is no way to type this correctly
# See https://github.com/python/typing/pull/1618
# ref: <https://github.com/pydantic/pydantic/blob/e1f9d15a5ed59d4b6f495154e2410823bdf55a3a/pydantic/type_adapter.py#L182-L184>
@overload
@cache
def _type_to_model_inner(type_: type[_T]) -> _ModelSerde[_T, RootModel[_T]]: ...
@overload
@cache
def _type_to_model_inner(type_: Any) -> _ModelSerde[_T, RootModel[_T]]: ...
@cache
def _type_to_model_inner(type_: Any) -> _ModelSerde[_T, RootModel[_T]]:
    root_model = RootModel[type_]

    # PERF, FIXME: Since we need to access the `root` attribute and
    # require `def serializer` and `def deserializer` as wrapper functions,
    # this may be slightly slower than using `pydantic.TypeAdapter`.
    # However, we need to use `pydantic.json_schema.models_json_schema` or `pydantic.TypeAdapter.json_schemas`
    # to generate json schema, and both only accept `BaseModel` or `TypeAdapter` (not mixed).
    # Therefore, we consistently use `BaseModel` here instead of `TypeAdapter`.
    # TODO: maybe we can submit a feature request to pydantic.

    def serializer(data: bytes) -> _T:
        return root_model.model_validate_json(data).root

    def deserializer(data: _T) -> str:
        return root_model(data).model_dump_json()

    return _ModelSerde(root_model, serializer, deserializer)


@overload
def _type_to_model(type_: type[_T]) -> _ModelSerde[_T, RootModel[_T]]: ...
@overload
def _type_to_model(type_: Any) -> _ModelSerde[_T, RootModel[_T]]: ...
def _type_to_model(type_: Any) -> _ModelSerde[_T, RootModel[_T]]:
    model_serde: _ModelSerde[_T, RootModel[_T]] = _type_to_model_inner(type_)
    # PERF: `cache_info` will require lock and instantiate a new `cache_info` tuple.
    if _type_to_model_inner.cache_info().misses > 128:
        warn(
            f"`{_type_to_model.__qualname__}` cache misses more than 128 times, "
            "please report this to pytauri developers",
            stacklevel=2,
        )
    return model_serde


class Commands(UserDict[str, _PyInvokHandleData]):
    """This class provides features similar to [tauri::generate_handler](https://docs.rs/tauri/latest/tauri/macro.generate_handler.html).

    Typically, you would use [Commands.command][pytauri.Commands.command] to register a command handler function.
    Then, use [Commands.generate_handler][pytauri.Commands.generate_handler] to get an `invoke_handler`
    for use with [BuilderArgs][pytauri.BuilderArgs].
    """

    _experimental_gen_ts: Optional[CommandInputOutput]

    def __init__(self, *, experimental_gen_ts: bool = False) -> None:  # noqa: D107
        super().__init__()

        self._experimental_gen_ts = {} if experimental_gen_ts else None

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

    def wrap_pyfunc(  # noqa: C901, PLR0912, PLR0915  # TODO: simplify this method
        self, pyfunc: _WrappablePyHandlerType, *, _gen_ts_cmd: Optional[str] = None
    ) -> _PyHandlerType:
        """Wrap a `Callable` to conform to the definition of PyHandlerType.

        Specifically:

        - If `pyfunc` has a `KEYWORD_ONLY` parameter named `body`:
            - If `body` is `bytes`:
                do nothing.
            - If `issubclass(body, BaseModel)`:
                wrap this callable as a new function with a `body: bytes` parameter.
            - Otherwise:
                try to convert it to a `BaseModel`/`TypeAdapter`, and proceed as in the `BaseModel` branch.
        - Handle the return type:
            - If the return type is `bytes`:
                do nothing.
            - If `issubclass(return_type, BaseModel)`:
                wrap this callable as a new function with `return: str` value.
            - Otherwise:
                try to convert it to a `BaseModel`/`TypeAdapter`, and proceed as in the `BaseModel` branch.
        - If no wrapping is needed, the original `pyfunc` will be returned.

        The `pyfunc` will be decorated using [functools.wraps][], and its `__signature__` will also be updated.
        """
        serializer: Optional[_Serializer[Union[BaseModel, Any]]] = None
        deserializer: Optional[_Deserializer[Union[BaseModel, Any]]] = None
        input_type = NoneType
        output_type = NoneType

        body_key = "body"

        sig = signature(pyfunc)
        parameters = sig.parameters
        return_annotation = sig.return_annotation

        body_param = parameters.get(body_key)
        if body_param is not None:
            if body_param.kind not in {
                Parameter.KEYWORD_ONLY,
                Parameter.POSITIONAL_OR_KEYWORD,
            }:
                raise ValueError(
                    f"Expected `{body_key}` to be KEYWORD parameter, but got `{body_param.kind}` parameter"
                )

            body_type = body_param.annotation
            if body_type is Parameter.empty:
                raise ValueError(
                    f"Expected `{body_key}` to have type annotation, but got empty"
                )
            elif body_type is bytes:
                serializer = None
                input_type = bytes
            # `Annotated`, `Union`, `None`, etc are not `type`
            elif isinstance(body_type, type) and issubclass(body_type, BaseModel):
                serializer = body_type.model_validate_json
                input_type = body_type
            else:
                # PERF, FIXME: `cast` make pyright happy, it mistakenly thinks this is `Any | type[Unknown]`
                body_type = cast(Any, body_type)
                try:
                    model_serde = _type_to_model(body_type)
                except Exception as e:
                    raise ValueError(
                        f"Failed to convert `{body_type}` type to pydantic Model, "
                        f"please explicitly use {BaseModel} or {bytes} as `{body_key}` type annotation instead."
                    ) from e
                serializer = model_serde.serializer
                input_type = model_serde.model

        if return_annotation is Signature.empty:
            raise ValueError(
                "Expected the return value to have type annotation, but got empty. "
                "Please explicitly use `def foo() -> None:` instead."
            )
        elif return_annotation is bytes:
            deserializer = None
            output_type = bytes
        # `Annotated`, `Union`, `None`, etc are not `type`
        elif isinstance(return_annotation, type) and issubclass(
            return_annotation, BaseModel
        ):
            # PERF: maybe we should cache this closure?
            def _deserializer(data: BaseModel) -> str:
                return data.model_dump_json()

            deserializer = _deserializer
            output_type = return_annotation
        else:
            # PERF, FIXME: `cast` make pyright happy, it mistakenly thinks this is `Any | type[Unknown]`
            return_annotation = cast(Any, return_annotation)
            try:
                model_serde = _type_to_model(return_annotation)
            except Exception as e:
                raise ValueError(
                    f"Failed to convert `{return_annotation}` type to pydantic Model, "
                    f"please explicitly use {BaseModel} or {bytes} as return type annotation instead."
                ) from e
            deserializer = model_serde.deserializer
            output_type = model_serde.model

        if _gen_ts_cmd is not None:
            assert self._experimental_gen_ts is not None
            self._experimental_gen_ts[_gen_ts_cmd] = InputOutput(
                input_type, output_type
            )

        if not serializer and not deserializer:
            return cast(_PyHandlerType, pyfunc)  # `cast` make typing happy

        @wraps(pyfunc)
        async def wrapper(*args: Any, **kwargs: Any) -> _InvokeResponseBody:
            nonlocal serializer, deserializer

            if serializer is not None:
                body_bytes = kwargs[body_key]
                assert isinstance(body_bytes, bytes)  # PERF
                try:
                    body_arg = serializer(body_bytes)
                except ValidationError as e:
                    raise InvokeException(repr(e)) from e
                kwargs[body_key] = body_arg

            resp = await pyfunc(*args, **kwargs)

            if deserializer is not None:
                # - subclass of `BaseModel`
                # - other types that are not `bytes`
                assert not isinstance(resp, bytes)  # PERF
                return deserializer(resp)
            else:
                assert isinstance(resp, bytes)  # PERF
                return resp

        new_parameters = parameters.copy()
        new_return_annotation = return_annotation
        if serializer is not None:
            new_parameters[body_key] = parameters[body_key].replace(annotation=bytes)
        if deserializer is not None:
            new_return_annotation = str

        # see: <https://docs.python.org/3.13/library/inspect.html#inspect.signature>
        wrapper.__signature__ = sig.replace(  # pyright: ignore[reportAttributeAccessIssue]
            parameters=tuple(new_parameters.values()),
            return_annotation=new_return_annotation,
        )
        return wrapper

    @staticmethod
    def parse_parameters(
        pyfunc: _PyHandlerType, /, check_signature: bool = True
    ) -> ParametersType:
        """Check the signature of a `Callable` and return the parameters.

        Check if the [Signature][inspect.Signature] of `pyfunc` conforms to [ArgumentsType][pytauri.ipc.ArgumentsType],
        and if the return value is [bytes][] or [str][].

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
            "body": bytes,
            "app_handle": AppHandle,
            "webview_window": WebviewWindow,
            "headers": Headers,
        }

        for name, param in parameters.items():
            # check if the `parameters` type hint conforms to [pytauri.ipc.ArgumentsType][]

            correct_anna = arguments_type.get(name)
            if correct_anna is None:
                raise ValueError(
                    f"Unexpected parameter `{name}`, expected one of {list(arguments_type.keys())}"
                )
            if param.annotation is not correct_anna:
                raise ValueError(
                    f"Expected `{name}` to be `{correct_anna}`, but got `{param.annotation}`"
                )
            if param.kind not in {
                Parameter.KEYWORD_ONLY,
                Parameter.POSITIONAL_OR_KEYWORD,
            }:
                raise ValueError(
                    f"Expected `{name}` to be KEYWORD parameter, but got `{param.kind}` parameter"
                )
        else:
            # after checking, we are sure that the `parameters` are valid
            parameters = cast(ParametersType, parameters)

        if return_annotation not in {bytes, str}:
            raise ValueError(
                f"Expected `return_annotation` to be {bytes} or {str}, got `{return_annotation}`"
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
        new_handler = self.wrap_pyfunc(
            handler,
            _gen_ts_cmd=command if self._experimental_gen_ts is not None else None,
        )
        parameters = self.parse_parameters(new_handler, check_signature=check_signature)
        self.data[command] = _PyInvokHandleData(parameters, new_handler)

    def _register(
        self,
        handler: _WrappablePyHandlerTypeVar,
        /,
        *,
        command: Optional[str] = None,
    ) -> _WrappablePyHandlerTypeVar:
        # it seems that `handler.__name__` is already `sys._is_interned` in cpython
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

    async def experimental_gen_ts(
        self,
        output_dir: Union[str, PathLike[str]],
        json2ts_cmd: str,
        *,
        cmd_alias: Optional[Callable[[str], str]] = to_camel,
    ) -> None:
        """Generate TypeScript types and API client from the registered commands.

        This method is only available if `experimental_gen_ts` is set to `True`
        when creating the `Commands` instance.

        Args:
            output_dir: The directory to output the generated TypeScript files.
            json2ts_cmd: The command to run [json-schema-to-typescript] to generate TypeScript types.
                [json-schema-to-typescript]: https://github.com/bcherny/json-schema-to-typescript/
            cmd_alias: An optional function to convert command names to TypeScript style.
                By default, it uses [to_camel][pydantic.alias_generators.to_camel].

        Raises:
            RuntimeError: If `experimental_gen_ts` is not enabled when creating the `Commands`
            instance.
        """
        if self._experimental_gen_ts is None:
            raise RuntimeError(
                "Experimental TypeScript generation is not enabled. "
                "Please set `experimental_gen_ts=True` when creating `Commands`."
            )
        cmd_in_out = self._experimental_gen_ts
        del self._experimental_gen_ts  # release memory

        await gen_ts(cmd_in_out, output_dir, json2ts_cmd, cmd_alias=cmd_alias)


# see:
# - <https://docs.pydantic.dev/2.10/concepts/types/#customizing-validation-with-__get_pydantic_core_schema__>
# - <https://docs.pydantic.dev/2.10/concepts/json_schema/#implementing-__get_pydantic_core_schema__>
_FFIJavaScriptChannelIdAnno = Annotated[
    _FFIJavaScriptChannelId,
    GetPydanticSchema(
        lambda _source, _handler: json_or_python_schema(
            json_schema=chain_schema(
                [
                    str_schema(),
                    no_info_plain_validator_function(_FFIJavaScriptChannelId.from_str),
                ]
            ),
            python_schema=any_schema(),
        ),
        lambda _source, handler: handler(str_schema()),
    ),
]

_ModelTypeVar = TypeVar(
    "_ModelTypeVar", bound=BaseModel, default=BaseModel, infer_variance=True
)


class JavaScriptChannelId(
    RootModel[_FFIJavaScriptChannelIdAnno], Generic[_ModelTypeVar]
):
    """This class is a wrapper around [pytauri.ffi.ipc.JavaScriptChannelId][].

    You can use this class as model field in pydantic model directly, or use it as model directly.

    > [pytauri.ffi.ipc.JavaScriptChannelId][] can't be used directly in pydantic model.

    # Examples

    ```py
    from asyncio import Task, create_task, sleep
    from typing import Any

    from pydantic import BaseModel, RootModel
    from pydantic.networks import HttpUrl
    from pytauri import Commands
    from pytauri.ipc import JavaScriptChannelId, WebviewWindow

    commands = Commands()

    Progress = RootModel[int]


    class Download(BaseModel):
        url: HttpUrl
        channel: JavaScriptChannelId[Progress]


    background_tasks: set[Task[Any]] = set()


    @commands.command()
    async def download(body: Download, webview_window: WebviewWindow) -> None:
        channel = body.channel.channel_on(webview_window.as_ref_webview())

        async def task():
            progress = Progress(0)
            while progress.root <= 100:
                channel.send_model(progress)
                await sleep(0.1)
                progress.root += 1

        t = create_task(task())
        background_tasks.add(t)
        t.add_done_callback(background_tasks.discard)


    # Or you can use it as `body` model directly
    @commands.command()
    async def my_command(body: JavaScriptChannelId) -> bytes: ...
    ```
    """

    @classmethod
    def from_str(cls, value: str, /) -> Self:
        """See [pytauri.ffi.ipc.JavaScriptChannelId.from_str][]."""
        ffi_js_channel_id = _FFIJavaScriptChannelId.from_str(value)
        return cls(ffi_js_channel_id)

    def channel_on(self, webview: Webview, /) -> "Channel[_ModelTypeVar]":
        """See [pytauri.ffi.ipc.JavaScriptChannelId.channel_on][]."""
        ffi_channel = self.root.channel_on(webview)
        return Channel(ffi_channel)


class Channel(Generic[_ModelTypeVar]):
    """This class is a wrapper around [pytauri.ffi.ipc.Channel][].

    It adds the following methods:

    - [send_model][pytauri.ipc.Channel.send_model]

    # Examples

    See [JavaScriptChannelId][pytauri.ipc.JavaScriptChannelId--examples]
    """

    def __init__(self, ffi_channel: _FFIChannel, /):  # noqa: D107
        self._ffi_channel = ffi_channel

    def id(self, /) -> int:
        """See [pytauri.ffi.ipc.Channel.id][]."""
        return self._ffi_channel.id()

    def send(self, data: _InvokeResponseBody, /) -> None:
        """See [pytauri.ffi.ipc.Channel.send][]."""
        self._ffi_channel.send(data)

    def send_model(self, model: _ModelTypeVar, /) -> None:
        """Equivalent to `self.send(model.model_dump_json())`."""
        self.send(model.model_dump_json())
