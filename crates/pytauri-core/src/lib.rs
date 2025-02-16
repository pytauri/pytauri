// See: <https://doc.rust-lang.org/rustdoc/unstable-features.html#extensions-to-the-doc-attribute>
#![cfg_attr(
    docsrs,
    feature(doc_cfg, doc_auto_cfg, doc_cfg_hide),
    doc(cfg_hide(doc))
)]

mod ext_mod_impl;
pub mod tauri_runtime;
pub mod utils;

use pyo3::{prelude::*, types::PyString};

/// You can access this module in Python via `pytuari.EXT_MOD.pytuari`.
///
/// See also: [tauri]
#[pymodule(submodule, gil_used = false, name = "pytauri")]
pub mod ext_mod {
    use super::*;

    #[pymodule_export]
    pub use ext_mod_impl::{
        App, AppHandle, Context, Event, Listener, Manager, Position, Rect, RunEvent, Size,
    };

    pub use ext_mod_impl::{
        EventId, ImplListener, ImplManager, PyAppHandleExt, PyAppHandleStateError,
        PyAppHandleStateResult,
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

        pub use ext_mod_impl::menu::{
            ImplContextMenu, MenuEvent, MenuID, MenuItemKind, HELP_SUBMENU_ID, WINDOW_SUBMENU_ID,
        };

        // TODO: see also <https://github.com/PyO3/pyo3/issues/3900#issue-2153617797> to export `const &str` to python.
        macro_rules! intern_var_to_mod {
            ($mod:expr, $py:expr, $var:ident) => {
                // NOTE: use [PyString::intern] instead of [pyo3::intern!] to avoid retaining a reference to [PyString] in Rust
                // ([pyo3::intern!] will create a new `static` variable in Rust).
                // TODO, PERF: intern the `name` also?
                $mod.add(stringify!($var), PyString::intern($py, $var))
            };
        }

        #[pymodule_init]
        fn module_init(m: &Bound<'_, PyModule>) -> PyResult<()> {
            let py = m.py();
            intern_var_to_mod!(m, py, HELP_SUBMENU_ID)?;
            intern_var_to_mod!(m, py, WINDOW_SUBMENU_ID)?;
            Ok(())
        }
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
}
