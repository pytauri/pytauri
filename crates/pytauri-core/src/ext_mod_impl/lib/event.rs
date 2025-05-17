use pyo3::{prelude::*, types::PyString};

/// See also: [tauri::EventId].
pub use tauri::EventId;

/// See also: [tauri::Event].
#[pyclass(frozen)]
#[non_exhaustive]
pub struct Event {
    #[pyo3(get)]
    pub id: EventId, // TODO, PERF: use `Py<PyInt>` instead of `u32` for getter performance.
    #[pyo3(get)]
    pub payload: Py<PyString>,
}

/// See also: [tauri::EventTarget].
//
// NOTE: use `Py<T>` instead of `String` for getter performance.
#[pyclass(frozen)]
#[non_exhaustive]
pub enum EventTarget {
    Any(),
    AnyLabel { label: Py<PyString> },
    App(),
    Window { label: Py<PyString> },
    Webview { label: Py<PyString> },
    WebviewWindow { label: Py<PyString> },
    _NonExhaustive(),
}

impl EventTarget {
    pub(crate) fn from_tauri(py: Python<'_>, value: &tauri::EventTarget) -> Self {
        match value {
            tauri::EventTarget::Any => Self::Any(),
            tauri::EventTarget::AnyLabel { label } => Self::AnyLabel {
                label: PyString::new(py, label).unbind(),
            },
            tauri::EventTarget::App => Self::App(),
            tauri::EventTarget::Window { label } => Self::Window {
                label: PyString::new(py, label).unbind(),
            },
            tauri::EventTarget::Webview { label } => Self::Webview {
                label: PyString::new(py, label).unbind(),
            },
            tauri::EventTarget::WebviewWindow { label } => Self::WebviewWindow {
                label: PyString::new(py, label).unbind(),
            },
            _ => Self::_NonExhaustive(),
        }
    }

    pub(crate) fn to_tauri(&self, py: Python<'_>) -> PyResult<tauri::EventTarget> {
        // TODO, PERF: once we drop py39, we can use [PyStringMethods::to_str] instead of [PyStringMethods::to_cow]
        let value = match self {
            Self::Any() => tauri::EventTarget::Any,
            Self::AnyLabel { label } => tauri::EventTarget::AnyLabel {
                label: label.bind(py).to_cow()?.into_owned(),
            },
            Self::App() => tauri::EventTarget::App,
            Self::Window { label } => tauri::EventTarget::Window {
                label: label.bind(py).to_cow()?.into_owned(),
            },
            Self::Webview { label } => tauri::EventTarget::Webview {
                label: label.bind(py).to_cow()?.into_owned(),
            },
            Self::WebviewWindow { label } => tauri::EventTarget::WebviewWindow {
                label: label.bind(py).to_cow()?.into_owned(),
            },
            Self::_NonExhaustive() => panic!("NonExhaustive is reserved for `#[non_exhaustive]`"),
        };
        Ok(value)
    }
}
