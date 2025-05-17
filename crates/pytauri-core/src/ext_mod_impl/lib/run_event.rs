use pyo3::{
    prelude::*,
    types::{PyInt, PyString},
    IntoPyObject,
};
use pyo3_utils::py_wrapper::{PyWrapper, PyWrapperT0};

use crate::ext_mod::{menu::MenuEvent, tray::TrayIconEvent};

/// see also: [tauri::RunEvent]
#[pyclass(frozen)]
#[non_exhaustive]
pub enum RunEvent {
    // use `Py<T>` to avoid creating new obj every time visiting the field,
    // see: <https://pyo3.rs/v0.23.4/faq.html#pyo3get-clones-my-field>
    Exit(),
    #[non_exhaustive]
    ExitRequested {
        code: Option<Py<PyInt>>,
        api: Py<ExitRequestApi>,
    },
    // TODO, PERF: maybe we should remove `WindowEvent` and `WebviewEvent` fields,
    // use `on_window_event` and `on_webview_event` instead.
    #[non_exhaustive]
    WindowEvent {
        label: Py<PyString>,
        // TODO:
        // event: WindowEvent,
    },
    #[non_exhaustive]
    WebviewEvent {
        label: Py<PyString>,
        // TODO:
        // event: WebviewEvent,
    },
    Ready(),
    Resumed(),
    MainEventsCleared(),
    // TODO, PERF: maybe we should remove `MenuEvent` and `TrayIconEvent` fields,
    // use `on_menu_event` and `on_tray_icon_event` instead.
    MenuEvent(Py<MenuEvent>),
    TrayIconEvent(Py<TrayIconEvent>),
    _NonExhaustive(),
}

impl RunEvent {
    pub(crate) fn from_tauri(py: Python<'_>, value: tauri::RunEvent) -> PyResult<Self> {
        let ret = match value {
            tauri::RunEvent::Exit => Self::Exit(),
            tauri::RunEvent::ExitRequested { code, api, .. } => {
                let code = code.map(|code| {
                    let Ok(code) = code.into_pyobject(py);
                    code.unbind()
                });
                let api = ExitRequestApi::new(api).into_pyobject(py)?.unbind();
                Self::ExitRequested { code, api }
            }
            tauri::RunEvent::WindowEvent {
                label, /* TODO */ ..
            } => Self::WindowEvent {
                // if `label` is immutable, we can intern it to save memory.
                label: PyString::intern(py, &label).unbind(),
            },
            tauri::RunEvent::WebviewEvent {
                label, /* TODO */ ..
            } => Self::WebviewEvent {
                label: PyString::intern(py, &label).unbind(),
            },
            tauri::RunEvent::Ready => Self::Ready(),
            tauri::RunEvent::Resumed => Self::Resumed(),
            tauri::RunEvent::MainEventsCleared => Self::MainEventsCleared(),
            tauri::RunEvent::MenuEvent(event) => {
                Self::MenuEvent(MenuEvent::intern(py, &event.id.0).unbind())
            }
            tauri::RunEvent::TrayIconEvent(event) => Self::TrayIconEvent(
                TrayIconEvent::from_tauri(py, event)?
                    .into_pyobject(py)?
                    .unbind(),
            ),
            _ => Self::_NonExhaustive(),
        };
        Ok(ret)
    }
}

/// See also: [tauri::CloseRequestApi]
#[pyclass(frozen)]
#[non_exhaustive]
pub struct CloseRequestApi(pub PyWrapper<PyWrapperT0<tauri::CloseRequestApi>>);

impl CloseRequestApi {
    fn new(value: tauri::CloseRequestApi) -> Self {
        Self(PyWrapper::new0(value))
    }
}

#[pymethods]
impl CloseRequestApi {
    // PERF: [Sender::send] is quick enough and never blocks,
    // so we don't need to release the GIL.
    fn prevent_close(&self) {
        self.0.inner_ref().prevent_close();
    }
}

/// See also: [tauri::ExitRequestApi]
#[pyclass(frozen)]
#[non_exhaustive]
pub struct ExitRequestApi(pub PyWrapper<PyWrapperT0<tauri::ExitRequestApi>>);

impl ExitRequestApi {
    fn new(value: tauri::ExitRequestApi) -> Self {
        Self(PyWrapper::new0(value))
    }
}

#[pymethods]
impl ExitRequestApi {
    // PERF: [Sender::send] is quick enough and never blocks,
    // so we don't need to release the GIL.
    fn prevent_exit(&self) {
        self.0.inner_ref().prevent_exit();
    }
}
