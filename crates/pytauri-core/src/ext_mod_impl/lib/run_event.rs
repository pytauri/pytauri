use std::path::PathBuf;

use pyo3::{
    prelude::*,
    types::{PyBool, PyFloat, PyInt, PyList, PyString},
    BoundObject as _, IntoPyObject,
};
use pyo3_utils::py_wrapper::{PyWrapper, PyWrapperT0};

use crate::ext_mod::{
    menu::MenuEvent, tray::TrayIconEvent, PhysicalPositionF64, PhysicalPositionI32,
    PhysicalSizeU32, Theme,
};

/// See also: [tauri::RunEvent]
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
        event: Py<WindowEvent>,
    },
    #[non_exhaustive]
    WebviewEvent {
        label: Py<PyString>,
        event: Py<WebviewEvent>,
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
            tauri::RunEvent::WindowEvent { label, event, .. } => Self::WindowEvent {
                // PERF: if `label` is immutable, we can intern it to save memory.
                label: PyString::intern(py, &label).unbind(),
                event: WindowEvent::from_tauri(py, &event)?
                    .into_pyobject(py)?
                    .unbind(),
            },
            tauri::RunEvent::WebviewEvent { label, event, .. } => Self::WebviewEvent {
                // PERF: if `label` is immutable, we can intern it to save memory.
                label: PyString::intern(py, &label).unbind(),
                event: WebviewEvent::from_tauri(py, &event)?
                    .into_pyobject(py)?
                    .unbind(),
            },
            tauri::RunEvent::Ready => Self::Ready(),
            tauri::RunEvent::Resumed => Self::Resumed(),
            tauri::RunEvent::MainEventsCleared => Self::MainEventsCleared(),
            tauri::RunEvent::MenuEvent(event) => {
                Self::MenuEvent(MenuEvent::intern(py, &event.id.0).unbind())
            }
            tauri::RunEvent::TrayIconEvent(event) => Self::TrayIconEvent(
                TrayIconEvent::from_tauri(py, &event)?
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

/// See also: [tauri::DragDropEvent::Enter::paths]
///
/// `list[pathlib.Path]`
#[derive(FromPyObject, IntoPyObject, IntoPyObjectRef)]
#[pyo3(transparent)]
struct VecPathBuf(Py<PyList>);

impl VecPathBuf {
    #[inline]
    fn from_tauri(py: Python<'_>, paths: &Vec<PathBuf>) -> PyResult<Self> {
        let lst = PyList::new(py, paths)?;
        Ok(Self(lst.unbind()))
    }
}

/// See also: [tauri::DragDropEvent]
#[pyclass(frozen)]
#[non_exhaustive]
pub enum DragDropEvent {
    // use `Py<T>` to avoid creating new obj every time visiting the field,
    // see: <https://pyo3.rs/v0.23.4/faq.html#pyo3get-clones-my-field>
    Enter {
        #[expect(private_interfaces)]
        paths: VecPathBuf,
        #[expect(private_interfaces)]
        position: PhysicalPositionF64,
    },
    Over {
        #[expect(private_interfaces)]
        position: PhysicalPositionF64,
    },
    Drop {
        #[expect(private_interfaces)]
        paths: VecPathBuf,
        #[expect(private_interfaces)]
        position: PhysicalPositionF64,
    },
    Leave(),
    _NonExhaustive(),
}

impl DragDropEvent {
    fn from_tauri(py: Python<'_>, value: &tauri::DragDropEvent) -> PyResult<Self> {
        let ret = match value {
            tauri::DragDropEvent::Enter { paths, position } => Self::Enter {
                paths: VecPathBuf::from_tauri(py, paths)?,
                position: PhysicalPositionF64::from_tauri(py, *position)?,
            },
            tauri::DragDropEvent::Over { position } => Self::Over {
                position: PhysicalPositionF64::from_tauri(py, *position)?,
            },
            tauri::DragDropEvent::Drop { paths, position } => Self::Drop {
                paths: VecPathBuf::from_tauri(py, paths)?,
                position: PhysicalPositionF64::from_tauri(py, *position)?,
            },
            tauri::DragDropEvent::Leave => Self::Leave(),
            _ => Self::_NonExhaustive(),
        };
        Ok(ret)
    }
}

/// See also: [tauri::WebviewEvent]
#[pyclass(frozen)]
#[non_exhaustive]
pub enum WebviewEvent {
    // use `Py<T>` to avoid creating new obj every time visiting the field,
    // see: <https://pyo3.rs/v0.23.4/faq.html#pyo3get-clones-my-field>
    DragDrop(Py<DragDropEvent>),
    _NonExhaustive(),
}

impl WebviewEvent {
    pub(crate) fn from_tauri(py: Python<'_>, value: &tauri::WebviewEvent) -> PyResult<Self> {
        let ret = match value {
            tauri::WebviewEvent::DragDrop(event) => Self::DragDrop(
                DragDropEvent::from_tauri(py, event)?
                    .into_pyobject(py)?
                    .unbind(),
            ),
            _ => Self::_NonExhaustive(),
        };
        Ok(ret)
    }
}

/// See also: [tauri::WindowEvent]
#[pyclass(frozen)]
#[non_exhaustive]
pub enum WindowEvent {
    // use `Py<T>` to avoid creating new obj every time visiting the field,
    // see: <https://pyo3.rs/v0.23.4/faq.html#pyo3get-clones-my-field>
    #[expect(private_interfaces)]
    Resized(PhysicalSizeU32),
    #[expect(private_interfaces)]
    Moved(PhysicalPositionI32),
    #[non_exhaustive]
    CloseRequested {
        api: Py<CloseRequestApi>,
    },
    Destroyed(),
    Focused(Py<PyBool>),
    #[non_exhaustive]
    ScaleFactorChanged {
        scale_factor: Py<PyFloat>,
        #[expect(private_interfaces)]
        new_inner_size: PhysicalSizeU32,
    },
    DragDrop(Py<DragDropEvent>),
    ThemeChanged(Py<Theme>),
    _NonExhaustive(),
}

impl WindowEvent {
    // NOTE: Because the parameter of [tauri::webview::WebviewWindow::on_window_event] is `&WindowEvent`,
    // and we do not want to clone [tauri::WindowEvent::DragDrop] (since it contains [Vec<PathBuf>]),
    // we use `&tauri::WindowEvent` as the parameter here.
    pub(crate) fn from_tauri(py: Python<'_>, value: &tauri::WindowEvent) -> PyResult<Self> {
        let ret = match value {
            tauri::WindowEvent::Resized(size) => {
                Self::Resized(PhysicalSizeU32::from_tauri(py, *size)?)
            }
            tauri::WindowEvent::Moved(pos) => {
                Self::Moved(PhysicalPositionI32::from_tauri(py, *pos)?)
            }
            tauri::WindowEvent::CloseRequested { api, .. } => {
                let api = CloseRequestApi::new(api.clone())
                    .into_pyobject(py)?
                    .unbind();
                Self::CloseRequested { api }
            }
            tauri::WindowEvent::Destroyed => Self::Destroyed(),
            tauri::WindowEvent::Focused(focused) => {
                Self::Focused(PyBool::new(py, *focused).unbind())
            }
            tauri::WindowEvent::ScaleFactorChanged {
                scale_factor,
                new_inner_size,
                ..
            } => Self::ScaleFactorChanged {
                scale_factor: PyFloat::new(py, *scale_factor).unbind(),
                new_inner_size: PhysicalSizeU32::from_tauri(py, *new_inner_size)?,
            },
            tauri::WindowEvent::DragDrop(event) => Self::DragDrop(
                DragDropEvent::from_tauri(py, event)?
                    .into_pyobject(py)?
                    .unbind(),
            ),
            tauri::WindowEvent::ThemeChanged(theme) => {
                Self::ThemeChanged(Theme::from(*theme).into_pyobject(py)?.unbind())
            }
            _ => Self::_NonExhaustive(),
        };
        Ok(ret)
    }
}
