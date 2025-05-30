# ruff: noqa: D102, D106

"""Original FFI interface module.

!!! warning
    All APIs under this module should not be considered stable.
    You should use the re-exported APIs under the top-level module.
"""

from enum import Enum, auto
from types import ModuleType
from typing import TYPE_CHECKING, Callable, Union, final

from pytauri import ImplManager
from pytauri.webview import WebviewWindow
from typing_extensions import Self, TypeAlias, TypeAliasType, TypedDict, Unpack

from pytauri_plugins import (
    PLUGIN_DIALOG,
    _pytauri_plugins_mod,  # pyright: ignore[reportPrivateUsage]
)

__all__ = [
    "DialogExt",
    "ImplDialogExt",
    "MessageDialogBuilder",
    "MessageDialogBuilderArgs",
    "MessageDialogButtons",
    "MessageDialogButtonsType",
    "MessageDialogKind",
]

if PLUGIN_DIALOG:
    _dialog_mod: ModuleType = _pytauri_plugins_mod.dialog
else:
    raise ImportError(
        "Enable the `plugin-dialog` feature for `pytauri` crate to use this plugin."
    )

_HasWindowHandleAndHasDisplayHandle: TypeAlias = WebviewWindow


if TYPE_CHECKING:

    @final
    class MessageDialogButtons:
        """[tauri_plugin_dialog::MessageDialogButtons](https://docs.rs/tauri-plugin-dialog/latest/tauri_plugin_dialog/enum.MessageDialogButtons.html)"""

        @final
        class Ok: ...

        @final
        class OkCancel: ...

        @final
        class YesNo: ...

        @final
        class OkCustom(tuple[str]):
            _0: str
            __match_args__ = ("_0",)

            def __new__(cls, _0: str, /) -> Self: ...

        @final
        class OkCancelCustom(tuple[str, str]):
            _0: str
            _1: str
            __match_args__ = ("_0", "_1")

            def __new__(cls, _0: str, _1: str, /) -> Self: ...

        # When adding new variants, remember to update `MessageDialogButtonsType`.

    class MessageDialogKind(Enum):
        """[tauri_plugin_dialog::MessageDialogKind](https://docs.rs/tauri-plugin-dialog/latest/tauri_plugin_dialog/enum.MessageDialogKind.html)"""

        Info = auto()
        Warning = auto()
        Error = auto()

    @final
    class MessageDialogBuilder:
        """[tauri_plugin_dialog::MessageDialogBuilder](https://docs.rs/tauri-plugin-dialog/latest/tauri_plugin_dialog/struct.MessageDialogBuilder.html)"""

        def blocking_show(
            self, /, **kwargs: Unpack["MessageDialogBuilderArgs"]
        ) -> bool: ...

        def show(
            self,
            handler: Callable[[bool], object],
            /,
            **kwargs: Unpack["MessageDialogBuilderArgs"],
        ) -> None: ...

    @final
    class DialogExt:
        """[tauri_plugin_dialog::DialogExt](https://docs.rs/tauri-plugin-dialog/latest/tauri_plugin_dialog/trait.DialogExt.html)"""

        @staticmethod
        def message(slf: "ImplDialogExt", message: str, /) -> MessageDialogBuilder: ...

else:
    MessageDialogButtons = _dialog_mod.MessageDialogButtons
    MessageDialogKind = _dialog_mod.MessageDialogKind
    MessageDialogBuilder = _dialog_mod.MessageDialogBuilder
    DialogExt = _dialog_mod.DialogExt

MessageDialogButtonsType = TypeAliasType(
    "MessageDialogButtonsType",
    Union[
        MessageDialogButtons.Ok,
        MessageDialogButtons.OkCancel,
        MessageDialogButtons.YesNo,
        MessageDialogButtons.OkCustom,
        MessageDialogButtons.OkCancelCustom,
    ],
)
"""See [MessageDialogButtons][pytauri_plugins.dialog.MessageDialogButtons] for details."""


class MessageDialogBuilderArgs(TypedDict, total=False):
    """[tauri_plugin_dialog::MessageDialogBuilder](https://docs.rs/tauri-plugin-dialog/latest/tauri_plugin_dialog/struct.MessageDialogBuilder.html)"""

    title: str
    parent: _HasWindowHandleAndHasDisplayHandle
    buttons: MessageDialogButtonsType
    kind: MessageDialogKind


ImplDialogExt: TypeAlias = ImplManager
"""The implementers of `DialogExt`."""
