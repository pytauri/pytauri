use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
};

use pyo3::{exceptions::PyRuntimeError, prelude::*};
use tauri_plugin_autostart::{self as plugin};

use crate::{ext_mod::plugin::Plugin, tauri_runtime::Runtime};

#[derive(Debug)]
struct PluginError(plugin::Error);

impl Display for PluginError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Error for PluginError {}

impl From<PluginError> for PyErr {
    fn from(value: PluginError) -> Self {
        match value.0 {
            plugin::Error::Io(e) => e.into(),
            e @ plugin::Error::Anyhow(_) => PyRuntimeError::new_err(e.to_string()),
        }
    }
}

impl From<plugin::Error> for PluginError {
    fn from(value: plugin::Error) -> Self {
        Self(value)
    }
}

macro_rules! macos_launcher_impl {
    ($ident:ident => : $($variant:ident),*) => {
        /// See also: [tauri_plugin_autostart::MacosLauncher]
        #[pyclass(frozen, eq, eq_int)]
        #[derive(PartialEq, Clone, Copy)]
        pub enum $ident {
            $($variant,)*
        }

        impl From<$ident> for tauri_plugin_autostart::MacosLauncher {
            fn from(val: $ident) -> Self {
                match val {
                    $($ident::$variant => tauri_plugin_autostart::MacosLauncher::$variant,)*
                }
            }
        }

        impl From<tauri_plugin_autostart::MacosLauncher> for $ident {
            fn from(val: tauri_plugin_autostart::MacosLauncher) -> Self {
                match val {
                    $(tauri_plugin_autostart::MacosLauncher::$variant => $ident::$variant,)*
                }
            }
        }

    };
}

macos_launcher_impl! (
    MacosLauncher => :
    LaunchAgent,
    AppleScript
);

/// See also: [tauri_plugin_autostart::init]
#[pyfunction]
#[pyo3(signature = (macos_launcher = MacosLauncher::LaunchAgent, args = None))]
pub fn init(
    #[cfg_attr(not(target_os = "macos"), expect(unused_variables))] macos_launcher: MacosLauncher,
    args: Option<Vec<String>>,
) -> Plugin {
    // TODO: `tauri_plugin_autostart::init` requires `'static`,
    // so we have to use `Builder` instead, see: <https://github.com/tauri-apps/plugins-workspace/pull/2909>.
    // We should deprecate this function in favor of the `Builder` binding.
    let mut builder = plugin::Builder::new();

    if let Some(args) = args {
        builder = builder.args(args);
    }
    #[cfg(target_os = "macos")]
    {
        builder = builder.macos_launcher(macos_launcher.into());
    }

    Plugin::new(Box::new(move || Box::new(builder.build::<Runtime>())))
}

/// See also: [tauri_plugin_autostart]
#[pymodule(submodule, gil_used = false)]
pub mod autostart {
    #[pymodule_export]
    pub use super::{init, MacosLauncher};
}
