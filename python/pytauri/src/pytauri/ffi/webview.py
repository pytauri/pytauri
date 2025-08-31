# ruff: noqa: D102

"""[tauri::webview](https://docs.rs/tauri/latest/tauri/webview/index.html)"""

import datetime
import sys
from collections.abc import Mapping
from enum import Enum, auto
from typing import (
    TYPE_CHECKING,
    Any,
    Callable,
    Optional,
    final,
)

from typing_extensions import NotRequired, Self, TypeAliasType, TypedDict, Unpack

from pytauri.ffi._ext_mod import pytauri_mod
from pytauri.ffi._typing import Pyo3PathInto, PySerdeFrom

__all__ = [
    "Color",
    "Cookie",
    "SameSite",
    "Webview",
    "WebviewWindow",
    "WebviewWindowBuilder",
    "WebviewWindowBuilderArgs",
]

_webview_mod = pytauri_mod.webview


Color = TypeAliasType("Color", tuple[int, int, int, int])
"""[tauri::webview::Color](https://docs.rs/tauri/latest/tauri/webview/struct.Color.html)

`(r, g, b, a): u8`
"""

_WindowConfigFrom = TypeAliasType("_WindowConfigFrom", Mapping[str, Any])
"""[tauri::utils::config::WindowConfig](https://docs.rs/tauri-utils/latest/tauri_utils/config/struct.WindowConfig.html)"""
_WindowConfigInto = TypeAliasType("_WindowConfigInto", dict[str, Any])
"""[tauri::utils::config::WindowConfig](https://docs.rs/tauri-utils/latest/tauri_utils/config/struct.WindowConfig.html)"""

if TYPE_CHECKING:
    from pytauri.ffi.image import Image
    from pytauri.ffi.lib import (
        CursorIcon,
        ImplManager,
        PositionType,
        SizeType,
        Theme,
        Url,
        UserAttentionType,
        WebviewEventType,
        WebviewUrlType,
        WindowEventType,
        _PhysicalPositionF64,  # pyright: ignore[reportPrivateUsage]
        _PhysicalPositionI32,  # pyright: ignore[reportPrivateUsage]
        _PhysicalSizeU32,  # pyright: ignore[reportPrivateUsage]
    )
    from pytauri.ffi.menu import ImplContextMenu, Menu, MenuEvent
    from pytauri.ffi.window import (
        Effects,
        Monitor,
        ProgressBarState,
        TitleBarStyle,
        Window,
    )

    @final
    class WebviewWindow:
        """[tauri::webview::WebviewWindow](https://docs.rs/tauri/latest/tauri/webview/struct.WebviewWindow.html)"""

        def __new__(
            cls,
            manager: ImplManager,
            label: str,
            url: WebviewUrlType,
            /,
            **kwargs: Unpack["WebviewWindowBuilderArgs"],
        ) -> Self: ...

        def run_on_main_thread(self, handler: Callable[[], object], /) -> None:
            """Runs the given closure on the main thread.

            !!! warning
                `handler` has the same restrictions as [App.run][pytauri.App.run].
            """
            ...

        def label(self) -> str: ...
        def on_window_event(
            self, handler: Callable[[WindowEventType], None], /
        ) -> None:
            """Registers a window event listener.

            !!! warning
                `handler` has the same restrictions as [App.run][pytauri.App.run].
            """

        def on_webview_event(
            self, handler: Callable[[WebviewEventType], None], /
        ) -> None:
            """Registers a window event listener.

            !!! warning
                `handler` has the same restrictions as [App.run][pytauri.App.run].
            """

        def on_menu_event(
            self, handler: Callable[["Self", "MenuEvent"], None], /
        ) -> None:
            """Registers a global menu event listener.

            !!! warning
                `handler` has the same restrictions as [App.run][pytauri.App.run].
            """

        def menu(self) -> Optional[Menu]: ...
        def set_menu(self, menu: Menu, /) -> Optional[Menu]: ...
        def remove_menu(self) -> Optional[Menu]: ...
        def hide_menu(self) -> None: ...
        def show_menu(self) -> None: ...
        def is_menu_visible(self) -> bool: ...
        def popup_menu(self, menu: ImplContextMenu, /) -> None: ...
        def popup_menu_at(
            self, menu: ImplContextMenu, position: PositionType, /
        ) -> None: ...
        def scale_factor(self) -> float: ...
        def inner_position(self) -> "_PhysicalPositionI32": ...
        def outer_position(self) -> "_PhysicalPositionI32": ...
        def inner_size(self) -> "_PhysicalSizeU32": ...
        def outer_size(self) -> "_PhysicalSizeU32": ...
        def is_fullscreen(self) -> bool: ...
        def is_minimized(self) -> bool: ...
        def is_maximized(self) -> bool: ...
        def is_focused(self) -> bool: ...
        def is_decorated(self) -> bool: ...
        def is_resizable(self) -> bool: ...
        def is_enabled(self) -> bool: ...
        def is_always_on_top(self) -> bool: ...
        def is_maximizable(self) -> bool: ...
        def is_minimizable(self) -> bool: ...
        def is_closable(self) -> bool: ...
        def is_visible(self) -> bool: ...
        def title(self) -> str: ...
        def current_monitor(self) -> Optional["Monitor"]: ...
        def primary_monitor(self) -> Optional["Monitor"]: ...
        def monitor_from_point(self, x: float, y: float, /) -> Optional["Monitor"]: ...
        def available_monitors(self) -> list["Monitor"]: ...
        def theme(self) -> Theme: ...
        def cursor_position(self) -> "_PhysicalPositionF64": ...
        def center(self) -> None: ...
        def request_user_attention(
            self, attention_type: Optional["UserAttentionType"], /
        ) -> None: ...
        def set_resizable(self, resizable: bool, /) -> None: ...
        def set_enabled(self, enabled: bool, /) -> None: ...
        def set_maximizable(self, maximizable: bool, /) -> None: ...
        def set_minimizable(self, minimizable: bool, /) -> None: ...
        def set_closable(self, closable: bool, /) -> None: ...
        def set_title(self, title: str, /) -> None: ...
        def maximize(self) -> None: ...
        def unmaximize(self) -> None: ...
        def minimize(self) -> None: ...
        def unminimize(self) -> None: ...
        def show(self) -> None: ...
        def hide(self) -> None: ...
        def close(self) -> None: ...
        def destroy(self) -> None: ...
        def set_decorations(self, decorations: bool, /) -> None: ...
        def set_shadow(self, shadow: bool, /) -> None: ...
        def set_effects(self, effects: Optional[Effects], /) -> None: ...
        def set_always_on_bottom(self, always_on_bottom: bool, /) -> None: ...
        def set_always_on_top(self, always_on_top: bool, /) -> None: ...
        def set_visible_on_all_workspaces(
            self, visible_on_all_workspaces: bool, /
        ) -> None: ...
        def set_content_protected(self, protected: bool, /) -> None: ...
        def set_size(self, size: SizeType, /) -> None: ...
        def set_min_size(self, size: Optional[SizeType], /) -> None: ...
        def set_max_size(self, size: Optional[SizeType], /) -> None: ...
        def set_position(self, position: PositionType, /) -> None: ...
        def set_fullscreen(self, fullscreen: bool, /) -> None: ...
        def set_focus(self) -> None: ...
        def set_icon(self, icon: Image, /) -> None: ...
        def set_background_color(self, color: Optional[Color], /) -> None: ...
        def set_skip_taskbar(self, skip: bool, /) -> None: ...
        def set_cursor_grab(self, grab: bool, /) -> None: ...
        def set_cursor_visible(self, visible: bool, /) -> None: ...
        def set_cursor_icon(self, icon: CursorIcon, /) -> None: ...
        def set_cursor_position(self, position: PositionType, /) -> None: ...
        def set_ignore_cursor_events(self, ignore: bool, /) -> None: ...
        def start_dragging(self) -> None: ...

        if sys.platform == "win32":

            def set_overlay_icon(self, icon: Optional[Image], /) -> None: ...

        def set_badge_count(self, count: Optional[int], /) -> None: ...

        if sys.platform == "darwin":

            def set_badge_label(self, label: Optional[str], /) -> None: ...

        def set_progress_bar(self, progress_state: ProgressBarState, /) -> None: ...
        def set_title_bar_style(self, style: TitleBarStyle, /) -> None: ...
        def set_theme(self, theme: Optional[Theme], /) -> None: ...
        def print(self) -> None: ...
        def url(self) -> Url: ...
        def navigate(self, url: Url, /) -> None: ...
        def reload(self) -> None: ...
        def eval(self, js: str, /) -> None: ...
        def open_devtools(self) -> None: ...
        def close_devtools(self) -> None: ...
        def is_devtools_open(self) -> bool: ...
        def set_zoom(self, scale_factor: float, /) -> None: ...
        def clear_all_browsing_data(self) -> None: ...
        def cookies_for_url(self, url: Url, /) -> list["Cookie"]: ...
        def cookies(self) -> list["Cookie"]: ...
        def set_cookie(self, cookie: "Cookie", /) -> None: ...
        def delete_cookie(self, cookie: "Cookie", /) -> None: ...
        def as_ref_webview(self) -> "Webview": ...

    @final
    class WebviewWindowBuilder:
        """[tauri::webview::WebviewWindowBuilder](https://docs.rs/tauri/latest/tauri/webview/struct.WebviewWindowBuilder.html)"""

        @staticmethod
        def build(
            manager: ImplManager,
            label: str,
            url: WebviewUrlType,
            /,
            **kwargs: Unpack["WebviewWindowBuilderArgs"],
        ) -> "WebviewWindow": ...

        @staticmethod
        def from_config(
            manager: ImplManager,
            config: PySerdeFrom[_WindowConfigFrom],
            /,
            **kwargs: Unpack["WebviewWindowBuilderArgs"],
        ) -> "WebviewWindow": ...

    @final
    class Webview:
        """[tauri::webview::Webview](https://docs.rs/tauri/latest/tauri/webview/struct.Webview.html)"""

        def window(self) -> "Window": ...

    @final
    class SameSite(Enum):
        """[cookie::SameSite](https://docs.rs/cookie/latest/cookie/enum.SameSite.html)"""

        Strict = auto()
        Lax = auto()
        None_ = auto()

else:
    WebviewWindow = _webview_mod.WebviewWindow
    WebviewWindowBuilder = _webview_mod.WebviewWindowBuilder
    Webview = _webview_mod.Webview
    SameSite = _webview_mod.SameSite


class Cookie(TypedDict, closed=True):
    """[cookie::Cookie](https://docs.rs/cookie/latest/cookie/struct.Cookie.html)"""

    key: str
    value: str
    max_age: NotRequired[Optional[int]]
    expires: NotRequired[Optional[datetime.datetime]]
    path: NotRequired[Optional[str]]
    domain: NotRequired[Optional[str]]
    secure: NotRequired[Optional[bool]]
    httponly: NotRequired[Optional[bool]]
    samesite: NotRequired[Optional[SameSite]]
    partitioned: NotRequired[Optional[bool]]


if sys.platform == "win32":

    class _BaseWebviewWindowBuilderArgs(TypedDict):
        owner: NotRequired["WebviewWindow"]
        drag_and_drop: NotRequired[bool]

elif sys.platform == "darwin":

    class _BaseWebviewWindowBuilderArgs(TypedDict):
        title_bar_style: NotRequired["TitleBarStyle"]
        traffic_light_position: NotRequired["PositionType"]
        allow_link_preview: NotRequired[bool]
        hidden_title: NotRequired[bool]
        tabbing_identifier: NotRequired[str]

elif sys.platform == "linux" or sys.platform.startswith(
    ("dragonfly", "freebsd", "netbsd", "openbsd")
):

    class _BaseWebviewWindowBuilderArgs(TypedDict):
        transient_for: NotRequired["WebviewWindow"]

else:

    class _BaseWebviewWindowBuilderArgs(TypedDict):
        pass


class WebviewWindowBuilderArgs(_BaseWebviewWindowBuilderArgs, closed=True):
    """[tauri::webview::WebviewWindowBuilder](https://docs.rs/tauri/latest/tauri/webview/struct.WebviewWindowBuilder.html)"""

    on_navigation: NotRequired[Callable[["Url"], bool]]
    on_document_title_changed: NotRequired[Callable[["WebviewWindow", str], None]]
    menu: NotRequired["Menu"]
    center: NotRequired[bool]
    position: NotRequired[tuple[float, float]]
    inner_size: NotRequired[tuple[float, float]]
    min_inner_size: NotRequired[tuple[float, float]]
    max_inner_size: NotRequired[tuple[float, float]]
    prevent_overflow: NotRequired[bool]
    prevent_overflow_with_margin: NotRequired["SizeType"]
    resizable: NotRequired[bool]
    maximizable: NotRequired[bool]
    minimizable: NotRequired[bool]
    closable: NotRequired[bool]
    title: NotRequired[str]
    fullscreen: NotRequired[bool]
    focusable: NotRequired[bool]
    focused: NotRequired[bool]
    maximized: NotRequired[bool]
    visible: NotRequired[bool]
    theme: NotRequired[Optional["Theme"]]
    decorations: NotRequired[bool]
    always_on_bottom: NotRequired[bool]
    always_on_top: NotRequired[bool]
    visible_on_all_workspaces: NotRequired[bool]
    content_protected: NotRequired[bool]
    icon: NotRequired["Image"]
    skip_taskbar: NotRequired[bool]
    window_classname: NotRequired[str]
    shadow: NotRequired[bool]
    parent: NotRequired["WebviewWindow"]
    effects: NotRequired["Effects"]
    accept_first_mouse: NotRequired[bool]
    initialization_script: NotRequired[str]
    initialization_script_for_all_frames: NotRequired[str]
    user_agent: NotRequired[str]
    additional_browser_args: NotRequired[str]
    data_directory: NotRequired[Pyo3PathInto]
    disable_drag_drop_handler: NotRequired[bool]
    enable_clipboard_access: NotRequired[bool]
    incognito: NotRequired[bool]
    auto_resize: NotRequired[bool]
    proxy_url: NotRequired["Url"]
    transparent: NotRequired[bool]
    zoom_hotkeys_enabled: NotRequired[bool]
    browser_extensions_enabled: NotRequired[bool]
    extensions_path: NotRequired[Pyo3PathInto]
    use_https_scheme: NotRequired[bool]
    devtools: NotRequired[bool]
    background_color: NotRequired["Color"]
    disable_javascript: NotRequired[bool]
