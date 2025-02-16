# ruff: noqa: D102

"""[tauri::menu](https://docs.rs/tauri/latest/tauri/menu/index.html)"""

from collections.abc import Sequence
from enum import Enum, auto
from typing import (
    TYPE_CHECKING,
    Optional,
    Union,
    final,
)

from typing_extensions import LiteralString, Self, TypeAliasType

from pytauri.ffi._ext_mod import pytauri_mod

__all__ = [
    "AboutMetadata",
    "CheckMenuItem",
    "ContextMenu",
    "IconMenuItem",
    "ImplContextMenu",
    "Menu",
    "MenuEvent",
    "MenuID",
    "MenuItem",
    "MenuItemKind",
    "NativeIcon",
    "PredefinedMenuItem",
    "Submenu",
]

_menu_mod = pytauri_mod.menu


MenuID = TypeAliasType("MenuID", str)
"""[tauri::menu::MenuID](https://docs.rs/tauri/latest/tauri/menu/struct.MenuId.html)"""
MenuEvent = TypeAliasType("MenuEvent", MenuID)
"""[tauri::menu::MenuEvent](https://docs.rs/tauri/latest/tauri/menu/struct.MenuEvent.html)"""

if TYPE_CHECKING:
    from pytauri.ffi.image import Image
    from pytauri.ffi.lib import AppHandle, ImplManager, PositionType
    from pytauri.ffi.window import Window

    @final
    class Menu:
        """[tauri::menu::Menu](https://docs.rs/tauri/latest/tauri/menu/struct.Menu.html)"""

        def __new__(cls, manager: ImplManager, /) -> Self: ...
        @staticmethod
        def with_id(manager: ImplManager, id: MenuID, /) -> "Menu": ...  # noqa: A002
        @staticmethod
        def with_items(
            manager: ImplManager, items: Sequence["MenuItemKind"], /
        ) -> "Menu": ...
        @staticmethod
        def with_id_and_items(
            manager: ImplManager,
            id: MenuID,  # noqa: A002
            items: Sequence["MenuItemKind"],
            /,
        ) -> "Menu": ...
        @staticmethod
        def default(app_handle: AppHandle, /) -> "Menu": ...
        def app_handle(self, /) -> AppHandle: ...
        def id(self, /) -> MenuID: ...
        def append(self, item: "MenuItemKind", /) -> None: ...
        def append_items(self, items: Sequence["MenuItemKind"], /) -> None: ...
        def prepend(self, item: "MenuItemKind", /) -> None: ...
        def prepend_items(self, items: Sequence["MenuItemKind"], /) -> None: ...
        def insert(self, item: "MenuItemKind", position: int, /) -> None: ...
        def insert_items(
            self, items: Sequence["MenuItemKind"], position: int, /
        ) -> None: ...
        def remove(self, item: "MenuItemKind", /) -> None: ...
        def remove_at(self, position: int, /) -> Optional["MenuItemKind"]: ...
        def get(self, id: MenuID, /) -> Optional["MenuItemKind"]: ...  # noqa: A002
        def items(self, /) -> list["MenuItemKind"]: ...
        def set_as_app_menu(self, /) -> None: ...
        def set_as_window_menu(self, window: Window, /) -> None: ...

    @final
    class MenuItem:
        """[tauri::menu::MenuItem](https://docs.rs/tauri/latest/tauri/menu/struct.MenuItem.html)"""

        def __new__(
            cls,
            manager: ImplManager,
            text: str,
            enabled: bool,
            accelerator: Optional[str] = None,
            /,
        ) -> Self: ...
        @staticmethod
        def with_id(
            manager: ImplManager,
            id: MenuID,  # noqa: A002
            text: str,
            enabled: bool,
            accelerator: Optional[str] = None,
            /,
        ) -> "MenuItem": ...
        def app_handle(self, /) -> AppHandle: ...
        def id(self, /) -> MenuID: ...
        def text(self, /) -> str: ...
        def set_text(self, text: str, /) -> None: ...
        def is_enabled(self, /) -> bool: ...
        def set_enabled(self, enabled: bool, /) -> None: ...
        def set_accelerator(self, accelerator: Optional[str], /) -> None: ...

    @final
    class Submenu:
        """[tauri::menu::Submenu](https://docs.rs/tauri/latest/tauri/menu/struct.Submenu.html)"""

        def __new__(cls, manager: ImplManager, text: str, enabled: bool, /) -> Self: ...
        @staticmethod
        def with_id(
            manager: ImplManager,
            id: MenuID,  # noqa: A002
            text: str,
            enabled: bool,
            /,
        ) -> "Submenu": ...
        @staticmethod
        def with_items(
            manager: ImplManager,
            text: str,
            enabled: bool,
            items: Sequence["MenuItemKind"],
            /,
        ) -> "Submenu": ...
        @staticmethod
        def with_id_and_items(
            manager: ImplManager,
            id: MenuID,  # noqa: A002
            text: str,
            enabled: bool,
            items: Sequence["MenuItemKind"],
            /,
        ) -> "Submenu": ...
        def app_handle(self, /) -> AppHandle: ...
        def id(self, /) -> MenuID: ...
        def append(self, item: "MenuItemKind", /) -> None: ...
        def append_items(self, items: Sequence["MenuItemKind"], /) -> None: ...
        def prepend(self, item: "MenuItemKind", /) -> None: ...
        def prepend_items(self, items: Sequence["MenuItemKind"], /) -> None: ...
        def insert(self, item: "MenuItemKind", position: int, /) -> None: ...
        def insert_items(
            self, items: Sequence["MenuItemKind"], position: int, /
        ) -> None: ...
        def remove(self, item: "MenuItemKind", /) -> None: ...
        def remove_at(self, position: int, /) -> Optional["MenuItemKind"]: ...
        def get(self, id: MenuID, /) -> Optional["MenuItemKind"]: ...  # noqa: A002
        def items(self, /) -> list["MenuItemKind"]: ...
        def text(self, /) -> str: ...
        def set_text(self, text: str, /) -> None: ...
        def is_enabled(self, /) -> bool: ...
        def set_enabled(self, enabled: bool, /) -> None: ...

    @final
    class PredefinedMenuItem:
        """[tauri::menu::PredefinedMenuItem](https://docs.rs/tauri/latest/tauri/menu/struct.PredefinedMenuItem.html)"""

        @staticmethod
        def separator(manager: ImplManager, /) -> "PredefinedMenuItem": ...
        @staticmethod
        def copy(
            manager: ImplManager, text: Optional[str] = None, /
        ) -> "PredefinedMenuItem": ...
        @staticmethod
        def cut(
            manager: ImplManager, text: Optional[str] = None, /
        ) -> "PredefinedMenuItem": ...
        @staticmethod
        def paste(
            manager: ImplManager, text: Optional[str] = None, /
        ) -> "PredefinedMenuItem": ...
        @staticmethod
        def select_all(
            manager: ImplManager, text: Optional[str] = None, /
        ) -> "PredefinedMenuItem": ...
        @staticmethod
        def undo(
            manager: ImplManager, text: Optional[str] = None, /
        ) -> "PredefinedMenuItem": ...
        @staticmethod
        def redo(
            manager: ImplManager, text: Optional[str] = None, /
        ) -> "PredefinedMenuItem": ...
        @staticmethod
        def minimize(
            manager: ImplManager, text: Optional[str] = None, /
        ) -> "PredefinedMenuItem": ...
        @staticmethod
        def maximize(
            manager: ImplManager, text: Optional[str] = None, /
        ) -> "PredefinedMenuItem": ...
        @staticmethod
        def fullscreen(
            manager: ImplManager, text: Optional[str] = None, /
        ) -> "PredefinedMenuItem": ...
        @staticmethod
        def hide(
            manager: ImplManager, text: Optional[str] = None, /
        ) -> "PredefinedMenuItem": ...
        @staticmethod
        def hide_others(
            manager: ImplManager, text: Optional[str] = None, /
        ) -> "PredefinedMenuItem": ...
        @staticmethod
        def show_all(
            manager: ImplManager, text: Optional[str] = None, /
        ) -> "PredefinedMenuItem": ...
        @staticmethod
        def close_window(
            manager: ImplManager, text: Optional[str] = None, /
        ) -> "PredefinedMenuItem": ...
        @staticmethod
        def quit(
            manager: ImplManager, text: Optional[str] = None, /
        ) -> "PredefinedMenuItem": ...
        @staticmethod
        def about(
            manager: ImplManager,
            text: Optional[str] = None,
            metadata: Optional["AboutMetadata"] = None,
            /,
        ) -> "PredefinedMenuItem": ...
        @staticmethod
        def services(
            manager: ImplManager, text: Optional[str] = None, /
        ) -> "PredefinedMenuItem": ...
        def app_handle(self, /) -> AppHandle: ...
        def id(self, /) -> MenuID: ...
        def text(self, /) -> str: ...
        def set_text(self, text: str, /) -> None: ...

    @final
    class CheckMenuItem:
        """[tauri::menu::CheckMenuItem](https://docs.rs/tauri/latest/tauri/menu/struct.CheckMenuItem.html)"""

        def __new__(
            cls,
            manager: ImplManager,
            text: str,
            enabled: bool,
            checked: bool,
            accelerator: Optional[str] = None,
            /,
        ) -> Self: ...
        @staticmethod
        def with_id(
            manager: ImplManager,
            id: MenuID,  # noqa: A002
            text: str,
            enabled: bool,
            checked: bool,
            accelerator: Optional[str] = None,
            /,
        ) -> "CheckMenuItem": ...
        def app_handle(self, /) -> AppHandle: ...
        def id(self, /) -> MenuID: ...
        def text(self, /) -> str: ...
        def set_text(self, text: str, /) -> None: ...
        def is_enabled(self, /) -> bool: ...
        def set_enabled(self, enabled: bool, /) -> None: ...
        def set_accelerator(self, accelerator: Optional[str], /) -> None: ...
        def is_checked(self, /) -> bool: ...
        def set_checked(self, checked: bool, /) -> None: ...

    class AboutMetadata:
        """[tauri::menu::AboutMetadata](https://docs.rs/tauri/latest/tauri/menu/struct.AboutMetadata.html)"""

        def __new__(
            cls,
            /,
            *,
            name: Optional[str] = None,
            version: Optional[str] = None,
            short_version: Optional[str] = None,
            authors: Optional[Sequence[str]] = None,
            comments: Optional[str] = None,
            copyright: Optional[str] = None,  # noqa: A002
            license: Optional[str] = None,  # noqa: A002
            website: Optional[str] = None,
            website_label: Optional[str] = None,
            credits: Optional[str] = None,  # noqa: A002
            icon: Optional["Image"] = None,
        ) -> Self: ...

    @final
    class IconMenuItem:
        """[tauri::menu::IconMenuItem](https://docs.rs/tauri/latest/tauri/menu/struct.IconMenuItem.html)"""

        def __new__(
            cls,
            manager: ImplManager,
            text: str,
            enabled: bool,
            icon: Optional["Image"] = None,
            accelerator: Optional[str] = None,
            /,
        ) -> Self: ...
        @staticmethod
        def with_id(
            manager: ImplManager,
            id: MenuID,  # noqa: A002
            text: str,
            enabled: bool,
            icon: Optional["Image"] = None,
            accelerator: Optional[str] = None,
            /,
        ) -> "IconMenuItem": ...
        @staticmethod
        def with_native_icon(
            manager: ImplManager,
            text: str,
            enabled: bool,
            native_icon: Optional["NativeIcon"] = None,
            accelerator: Optional[str] = None,
            /,
        ) -> "IconMenuItem": ...
        @staticmethod
        def with_id_and_native_icon(
            manager: ImplManager,
            id: MenuID,  # noqa: A002
            text: str,
            enabled: bool,
            native_icon: Optional["NativeIcon"] = None,
            accelerator: Optional[str] = None,
            /,
        ) -> "IconMenuItem": ...
        def app_handle(self, /) -> AppHandle: ...
        def id(self, /) -> MenuID: ...
        def text(self, /) -> str: ...
        def set_text(self, text: str, /) -> None: ...
        def is_enabled(self, /) -> bool: ...
        def set_enabled(self, enabled: bool, /) -> None: ...
        def set_accelerator(self, accelerator: Optional[str], /) -> None: ...
        def set_icon(self, icon: Optional["Image"], /) -> None: ...
        def set_native_icon(self, native_icon: Optional["NativeIcon"], /) -> None: ...

    @final
    class NativeIcon(Enum):
        """[tauri::menu::NativeIcon](https://docs.rs/tauri/latest/tauri/menu/enum.NativeIcon.html)

        !!! warning
            This is actually a `Class` disguised as an `Enum`. The order of fields is not guaranteed.
            See also: <https://pyo3.rs/v0.23.4/class.html#pyclass-enums>.
        """

        Add = auto()
        Advanced = auto()
        Bluetooth = auto()
        Bookmarks = auto()
        Caution = auto()
        ColorPanel = auto()
        ColumnView = auto()
        Computer = auto()
        EnterFullScreen = auto()
        Everyone = auto()
        ExitFullScreen = auto()
        FlowView = auto()
        Folder = auto()
        FolderBurnable = auto()
        FolderSmart = auto()
        FollowLinkFreestanding = auto()
        FontPanel = auto()
        GoLeft = auto()
        GoRight = auto()
        Home = auto()
        IChatTheater = auto()
        IconView = auto()
        Info = auto()
        InvalidDataFreestanding = auto()
        LeftFacingTriangle = auto()
        ListView = auto()
        LockLocked = auto()
        LockUnlocked = auto()
        MenuMixedState = auto()
        MenuOnState = auto()
        MobileMe = auto()
        MultipleDocuments = auto()
        Network = auto()
        Path = auto()
        PreferencesGeneral = auto()
        QuickLook = auto()
        RefreshFreestanding = auto()
        Refresh = auto()
        Remove = auto()
        RevealFreestanding = auto()
        RightFacingTriangle = auto()
        Share = auto()
        Slideshow = auto()
        SmartBadge = auto()
        StatusAvailable = auto()
        StatusNone = auto()
        StatusPartiallyAvailable = auto()
        StatusUnavailable = auto()
        StopProgressFreestanding = auto()
        StopProgress = auto()
        TrashEmpty = auto()
        TrashFull = auto()
        User = auto()
        UserAccounts = auto()
        UserGroup = auto()
        UserGuest = auto()

    HELP_SUBMENU_ID: LiteralString
    """[tauri::menu::HELP_SUBMENU_ID](https://docs.rs/tauri/latest/tauri/menu/constant.HELP_SUBMENU_ID.html)"""
    WINDOW_SUBMENU_ID: LiteralString
    """[tauri::menu::WINDOW_SUBMENU_ID](https://docs.rs/tauri/latest/tauri/menu/constant.WINDOW_SUBMENU_ID.html)"""

    class ContextMenu:
        """[tauri::menu::ContextMenu](https://docs.rs/tauri/latest/tauri/menu/trait.ContextMenu.html)"""

        @staticmethod
        def popup(slf: "ImplContextMenu", window: Window, /) -> None: ...

        @staticmethod
        def popup_at(
            slf: "ImplContextMenu", window: Window, position: PositionType, /
        ) -> None: ...

else:
    Menu = _menu_mod.Menu
    Submenu = _menu_mod.Submenu
    MenuItem = _menu_mod.MenuItem
    CheckMenuItem = _menu_mod.CheckMenuItem
    PredefinedMenuItem = _menu_mod.PredefinedMenuItem
    AboutMetadata = _menu_mod.AboutMetadata
    IconMenuItem = _menu_mod.IconMenuItem
    NativeIcon = _menu_mod.NativeIcon
    HELP_SUBMENU_ID = _menu_mod.HELP_SUBMENU_ID
    WINDOW_SUBMENU_ID = _menu_mod.WINDOW_SUBMENU_ID
    ContextMenu = _menu_mod.ContextMenu


MenuItemKind = TypeAliasType(
    "MenuItemKind",
    Union[MenuItem, Submenu, PredefinedMenuItem, CheckMenuItem, IconMenuItem],
)
"""[tauri::menu::MenuItemKind](https://docs.rs/tauri/latest/tauri/menu/enum.MenuItemKind.html)"""

ImplContextMenu = TypeAliasType("ImplContextMenu", Union[Menu, Submenu])
