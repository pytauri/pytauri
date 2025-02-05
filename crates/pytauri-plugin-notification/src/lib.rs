//! # Example
/*!
```rust
use pyo3::prelude::*;

#[pymodule(gil_used = false)]
#[pyo3(name = "_ext_mod")]
pub mod _ext_mod {
    #[pymodule_export]
    use pytauri_plugin_notification::notification;
}
```
*/

mod ext_mod_impl;

use pyo3::prelude::*;

/// You can access this module in Python via `pytuari.EXT_MOD.notification`.
///
/// Please refer to the Python-side documentation.
///
/// See also: [tauri_plugin_notification]
#[pymodule(submodule, gil_used = false)]
pub mod notification {
    use super::*;

    #[pymodule_export]
    pub use ext_mod_impl::{NotificationBuilder, NotificationExt};

    pub use ext_mod_impl::ImplNotificationExt;
}

pub use notification as ext_mod;
