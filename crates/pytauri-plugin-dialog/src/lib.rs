//! # Example
/*!
```rust
use pyo3::prelude::*;

#[pymodule(gil_used = false)]
#[pyo3(name = "_ext_mod")]
pub mod _ext_mod {
    #[pymodule_export]
    use pytauri_plugin_dialog::dialog;
}
```
*/

mod ext_mod_impl;

use pyo3::prelude::*;

/// You can access this module in Python via `pytuari.EXT_MOD.dialog`.
///
/// Please refer to the Python-side documentation.
///
/// See also: [tauri_plugin_dialog]
#[pymodule(submodule, gil_used = false)]
pub mod dialog {
    use super::*;

    #[pymodule_export]
    pub use ext_mod_impl::{DialogExt, MessageDialogBuilder};

    pub use ext_mod_impl::ImplDialogExt;
}

pub use dialog as ext_mod;
