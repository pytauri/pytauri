use pyo3::{
    exceptions::PyNotImplementedError,
    prelude::*,
    types::{PyInt, PyString},
    IntoPyObject,
};

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
        // TODO, XXX, FIXME: `ExitRequestApi` is a private type in `tauri`,
        // we need create a issue to `tauri`, or we cant implement this.
        // See: <https://github.com/tauri-apps/tauri/pull/12701>
        // api: ExitRequestApi,
    },
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
}

impl RunEvent {
    pub(crate) fn new(py: Python<'_>, value: tauri::RunEvent) -> PyResult<Self> {
        let ret = match value {
            tauri::RunEvent::Exit => Self::Exit(),
            tauri::RunEvent::ExitRequested {
                code, /* TODO */ ..
            } => {
                let code = code.map(|code| {
                    let Ok(code) = code.into_pyobject(py);
                    code.unbind()
                });
                Self::ExitRequested { code }
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
            event => {
                return Err(PyNotImplementedError::new_err(format!(
                    "Please make a issue for unimplemented RunEvent: {event:?}",
                )))
            }
        };
        Ok(ret)
    }
}
