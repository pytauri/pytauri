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
        rect::{Position, Rect, Size},
        run_event::{
            CloseRequestApi, DragDropEvent, ExitRequestApi, RunEvent, WebviewEvent, WindowEvent,
        },
        theme::Theme,
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

    pub(crate) use ext_mod_impl::lib::{
        app::TauriApp,
        app_handle::{debug_assert_app_handle_py_is_rs, TauriAppHandle},
        assets::PyAssets,
        manager::{manager_method_impl, StateManager},
        rect::{PhysicalPositionF64, PhysicalPositionI32, PhysicalSizeU32},
    };

    /// see also: [tauri::ipc]
    #[pymodule]
    pub mod ipc {
        use super::*;

        #[pymodule_export]
        pub use ext_mod_impl::ipc::{Channel, Invoke, InvokeResolver, JavaScriptChannelId};
    }

    /// see also: [tauri::webview]
    #[pymodule]
    pub mod webview {
        use super::*;

        #[pymodule_export]
        pub use ext_mod_impl::webview::{Webview, WebviewWindow};

        pub(crate) use ext_mod_impl::webview::TauriWebviewWindow;
    }

    /// see also: [tauri::menu]
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

    /// see also: [tauri::image]
    #[pymodule]
    pub mod image {
        use super::*;

        #[pymodule_export]
        pub use ext_mod_impl::image::Image;
    }

    /// see also: [tauri::window]
    #[pymodule]
    pub mod window {
        use super::*;

        #[pymodule_export]
        pub use ext_mod_impl::window::Window;
    }

    /// see also: [tauri::tray]
    #[pymodule]
    pub mod tray {
        use super::*;

        #[pymodule_export]
        pub use ext_mod_impl::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconEvent};

        pub use ext_mod_impl::tray::TrayIconId;
    }

    /// see also: [tauri::path]
    #[pymodule]
    pub mod path {
        use super::*;

        #[pymodule_export]
        pub use ext_mod_impl::path::PathResolver;
    }

    /// see also: [tauri::plugin]
    #[pymodule]
    pub mod plugin {
        use super::*;

        #[pymodule_export]
        pub use ext_mod_impl::plugin::Plugin;

        #[cfg(feature = "__private")]
        pub use ext_mod_impl::plugin::BoxedPluginWrapper;
    }
}
