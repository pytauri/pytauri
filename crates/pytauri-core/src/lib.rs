// See: <https://doc.rust-lang.org/rustdoc/unstable-features.html#extensions-to-the-doc-attribute>
#![cfg_attr(
    docsrs,
    feature(doc_cfg, doc_auto_cfg, doc_cfg_hide),
    doc(cfg_hide(doc))
)]

mod ext_mod_impl;
mod plugins;
pub mod tauri_runtime;
pub mod utils;

use pyo3::prelude::*;

pub use plugins::pytauri_plugins;

/// See also: [tauri]
///
/// You can access this module in Python via `pytuari.EXT_MOD.pytuari`.
#[pymodule(submodule, gil_used = false, name = "pytauri")]
pub mod ext_mod {
    use super::*;

    #[pymodule_export]
    pub use ext_mod_impl::lib::{
        app::App,
        app_handle::AppHandle,
        context::Context,
        emitter::Emitter,
        event::{Event, EventTarget},
        listener::Listener,
        manager::Manager,
        rect::{LogicalRect, PhysicalRect, Position, Rect, Size},
        run_event::{
            CloseRequestApi, DragDropEvent, ExitRequestApi, RunEvent, WebviewEvent, WindowEvent,
        },
        runtime::{CursorIcon, Theme, UserAttentionType},
        webview_version,
    };
    // TODO: constants defined outside a module and then re-exported are not supported,
    // see <https://github.com/PyO3/pyo3/pull/5150#issuecomment-2889031243>.
    #[pymodule_export]
    pub const RESTART_EXIT_CODE: i32 = ext_mod_impl::lib::RESTART_EXIT_CODE;
    #[pymodule_export]
    pub const VERSION: &str = ext_mod_impl::lib::VERSION;
    #[pymodule_export]
    pub const IS_DEV: bool = ext_mod_impl::lib::IS_DEV;

    pub use ext_mod_impl::lib::{
        app_handle::{PyAppHandleExt, PyAppHandleStateError, PyAppHandleStateResult},
        emitter::ImplEmitter,
        event::EventId,
        listener::ImplListener,
        manager::ImplManager,
        url::Url,
    };

    #[expect(unused_imports)] // TODO
    pub(crate) use ext_mod_impl::lib::rect::{TauriLogicalRect, TauriPhysicalRect};
    pub(crate) use ext_mod_impl::lib::{
        app::TauriApp,
        app_handle::{debug_assert_app_handle_py_is_rs, TauriAppHandle},
        assets::PyAssets,
        manager::{manager_method_impl, StateManager},
        rect::{PhysicalPositionF64, PhysicalPositionI32, PhysicalSizeU32},
    };

    /// See also: [tauri::ipc]
    #[pymodule]
    pub mod ipc {
        use super::*;

        #[pymodule_export]
        pub use ext_mod_impl::ipc::{Channel, Invoke, InvokeResolver, JavaScriptChannelId};
    }

    /// See also: [tauri::webview]
    #[pymodule]
    pub mod webview {
        use super::*;

        #[pymodule_export]
        pub use ext_mod_impl::webview::{SameSite, Webview, WebviewWindow};

        pub use ext_mod_impl::webview::{Color, Cookie};

        pub(crate) use ext_mod_impl::webview::TauriWebviewWindow;
    }

    /// See also: [tauri::menu]
    #[pymodule]
    pub mod menu {
        use super::*;

        #[pymodule_export]
        pub use ext_mod_impl::menu::{
            AboutMetadata, CheckMenuItem, ContextMenu, IconMenuItem, Menu, MenuItem, NativeIcon,
            PredefinedMenuItem, Submenu,
        };

        // TODO: constants defined outside a module and then re-exported are not supported,
        // see <https://github.com/PyO3/pyo3/pull/5150#issuecomment-2889031243>.
        #[pymodule_export]
        pub const HELP_SUBMENU_ID: &str = ext_mod_impl::menu::HELP_SUBMENU_ID;
        #[pymodule_export]
        pub const WINDOW_SUBMENU_ID: &str = ext_mod_impl::menu::WINDOW_SUBMENU_ID;

        pub use ext_mod_impl::menu::{ImplContextMenu, MenuEvent, MenuID, MenuItemKind};

        pub(crate) use ext_mod_impl::menu::context_menu_impl;
    }

    /// See also: [tauri::image]
    #[pymodule]
    pub mod image {
        use super::*;

        #[pymodule_export]
        pub use ext_mod_impl::image::Image;
    }

    /// See also: [tauri::window]
    #[pymodule]
    pub mod window {
        use super::*;

        #[pymodule_export]
        pub use ext_mod_impl::window::{
            Effect, EffectState, Monitor, ProgressBarStatus, TitleBarStyle, Window,
        };

        pub use ext_mod_impl::window::{Effects, ProgressBarState};
    }

    /// See also: [tauri::tray]
    #[pymodule]
    pub mod tray {
        use super::*;

        #[pymodule_export]
        pub use ext_mod_impl::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconEvent};

        pub use ext_mod_impl::tray::TrayIconId;
    }

    /// See also: [tauri::path]
    #[pymodule]
    pub mod path {
        use super::*;

        #[pymodule_export]
        pub use ext_mod_impl::path::PathResolver;
    }

    /// See also: [tauri::plugin]
    #[pymodule]
    pub mod plugin {
        use super::*;

        #[pymodule_export]
        pub use ext_mod_impl::plugin::Plugin;
    }
}
