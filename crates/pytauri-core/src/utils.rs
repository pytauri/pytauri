use std::{
    any::Any,
    error::Error,
    fmt::{Display, Formatter},
    panic::panic_any,
};

use pyo3::{exceptions::PyRuntimeError, prelude::*};

/// Utility for converting [tauri::Error] to [pyo3::PyErr].
///
/// See also: <https://pyo3.rs/v0.23.2/function/error-handling.html#foreign-rust-error-types>.
///
/// # Example
///
/**
```rust
use pyo3::prelude::*;
use pytauri_core::utils::{TauriError, TauriResult};

fn tauri_result() -> tauri::Result<()> {
    Ok(())
}

#[pyfunction]
fn foo() -> PyResult<()> {
    tauri_result().map_err(Into::<TauriError>::into)?;
    Ok(())
}

#[pyfunction]
fn bar() -> TauriResult<()> {
    tauri_result()?;
    Ok(())
}
```
*/

#[derive(Debug)]
pub struct TauriError(tauri::Error);

impl Display for TauriError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Error for TauriError {}

impl From<TauriError> for PyErr {
    fn from(value: TauriError) -> Self {
        PyRuntimeError::new_err(value.0.to_string())
    }
}

impl From<tauri::Error> for TauriError {
    fn from(value: tauri::Error) -> Self {
        Self(value)
    }
}

pub type TauriResult<T> = Result<T, TauriError>;

// keep it private, maybe we will refactor it in the future
pub(crate) trait PyResultExt {
    type Output;

    fn unwrap_unraisable_py_result<M>(
        self,
        py: Python<'_>,
        obj: Option<&Bound<'_, PyAny>>,
        msg: impl FnOnce() -> M,
    ) -> Self::Output
    where
        M: Any + Send + 'static;
}

impl<T> PyResultExt for PyResult<T> {
    type Output = T;

    #[inline] // `inline` to allow optimize the `FnOnce` lazy closure
    fn unwrap_unraisable_py_result<M>(
        self,
        py: Python<'_>,
        obj: Option<&Bound<'_, PyAny>>,
        msg: impl FnOnce() -> M,
    ) -> Self::Output
    where
        M: Any + Send + 'static,
    {
        match self {
            Ok(v) => v,
            Err(err) => {
                // Use [write_unraisable] instead of [restore]:
                // - Because we are about to panic, Python might abort
                // - [restore] will not be handled in this case, so it will not be printed to stderr
                err.write_unraisable(py, obj);
                // `panic` allows Python to exit `app.run()`,
                // otherwise the Python main thread will be blocked by `app.run()`
                // and unable to raise an error
                panic_any(msg());
            }
        }
    }
}

macro_rules! delegate_inner {
    ($slf:expr, $func:ident, $($arg:expr),*) => {
        $slf.0
            .inner_ref()
            .$func($($arg),*)
            .map_err($crate::utils::TauriError::from)
            .map_err(pyo3::PyErr::from)
    };
}

pub(crate) use delegate_inner;
