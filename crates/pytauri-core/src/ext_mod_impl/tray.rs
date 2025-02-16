use std::path::PathBuf;

use pyo3::{
    exceptions::PyNotImplementedError,
    prelude::*,
    types::{PyString, PyTuple},
    FromPyObject, IntoPyObject,
};
use pyo3_utils::{
    py_wrapper::{PyWrapper, PyWrapperT0},
    ungil::UnsafeUngilExt,
};
use tauri::tray;

use crate::{
    context_menu_impl, delegate_inner,
    ext_mod_impl::{self, menu::ImplContextMenu, ImplManager, PyAppHandleExt as _, Rect},
    manager_method_impl,
    tauri_runtime::Runtime,
    utils::TauriError,
};

type TauriTrayIcon = tray::TrayIcon<Runtime>;

/// see also: [tauri::tray::TrayIconId]
///
/// Remember use [TrayIconId::intern] to create a new instance.
pub type TrayIconId = PyString;

/// see also: [tauri::tray::TrayIcon]
#[pyclass(frozen)]
#[non_exhaustive]
pub struct TrayIcon(pub PyWrapper<PyWrapperT0<TauriTrayIcon>>);

impl TrayIcon {
    pub(crate) fn new(tray_icon: TauriTrayIcon) -> Self {
        Self(PyWrapper::new0(tray_icon))
    }

    #[inline]
    fn new_impl(
        py: Python<'_>,
        manager: &impl tauri::Manager<Runtime>,
        id: Option<impl Into<tray::TrayIconId> + Send>,
    ) -> PyResult<Self> {
        unsafe {
            py.allow_threads_unsend(manager, |manager| {
                let tray_icon_builder = if let Some(id) = id {
                    tray::TrayIconBuilder::with_id(id)
                } else {
                    tray::TrayIconBuilder::new()
                };
                let tray_icon = tray_icon_builder.build(manager)?;

                tauri::Result::Ok(Self::new(tray_icon))
            })
        }
        .map_err(TauriError::from)
        .map_err(PyErr::from)
    }
}

#[pymethods]
impl TrayIcon {
    #[new]
    fn __new__(py: Python<'_>, manager: ImplManager) -> PyResult<Self> {
        manager_method_impl!(py, &manager, |py, manager| {
            Self::new_impl(py, manager, None::<&str>)
        })?
    }

    #[staticmethod]
    fn with_id(py: Python<'_>, manager: ImplManager, id: String) -> PyResult<Self> {
        let id = tray::TrayIconId(id);
        manager_method_impl!(py, &manager, |py, manager| {
            Self::new_impl(py, manager, Some(id))
        })?
    }

    fn app_handle(&self, py: Python<'_>) -> Py<ext_mod_impl::AppHandle> {
        let tray_icon = self.0.inner_ref();
        // TODO, PERF: release the GIL?
        let app_handle = tray_icon.app_handle().py_app_handle().clone_ref(py);
        app_handle
    }

    fn on_menu_event(&self, py: Python<'_>, handler: PyObject) {
        // Delegate to [ext_mod_impl::AppHandle::on_menu_event] as their implementation is the same:
        // - <https://docs.rs/tauri/2.2.5/tauri/tray/struct.TrayIcon.html#method.on_menu_event>
        // - <https://docs.rs/tauri/2.2.5/tauri/struct.AppHandle.html#method.on_menu_event>
        let app_handle = self.app_handle(py);
        ext_mod_impl::AppHandle::on_menu_event(app_handle, py, handler);
    }

    fn on_tray_icon_event(slf: Py<Self>, py: Python<'_>, handler: PyObject) {
        let moved_slf = slf.clone_ref(py);
        py.allow_threads(|| {
            slf.get()
                .0
                .inner_ref()
                .on_tray_icon_event(move |_tray_icon, tray_icon_event| {
                    Python::with_gil(|py| {
                        // See: <https://github.com/tauri-apps/tauri/blob/8e9339e8807338597132ffd8688fb9da00f4102b/crates/tauri/src/app.rs#L2185-L2205>,
                        // The `tray_icon` argument is always the `TrayIcon` instance that calls this method,
                        // so we can directly use the same PyObject.
                        let tray_icon: &Py<Self> = &moved_slf;
                        debug_assert_eq!(tray_icon.get().0.inner_ref().id(), _tray_icon.id());
                        let tray_icon_event: TrayIconEvent =
                            TrayIconEvent::from_tauri(py, tray_icon_event)
                                // TODO: maybe we should only `write_unraisable` and log it instead of `panic` here?
                                .expect("Failed to convert rust `TrayIconEvent` to pyobject");

                        let handler = handler.bind(py);
                        let result = handler.call1((tray_icon, tray_icon_event));
                        if let Err(e) = result {
                            e.write_unraisable(py, Some(handler));
                            panic!(
                                "Python exception occurred in `TrayIcon::on_tray_icon_event` handler"
                            )
                        }
                    })
                })
        })
    }

    fn id<'py>(&self, py: Python<'py>) -> Bound<'py, TrayIconId> {
        let tray_icon = self.0.inner_ref();
        TrayIconId::intern(py, &tray_icon.id().0)
    }

    #[pyo3(signature = (icon))]
    fn set_icon(
        &self,
        py: Python<'_>,
        icon: Option<Py<ext_mod_impl::image::Image>>,
    ) -> PyResult<()> {
        let icon = icon.as_ref().map(|icon| icon.get().to_tauri(py));
        py.allow_threads(|| delegate_inner!(self, set_icon, icon))
    }

    #[pyo3(signature = (menu))]
    fn set_menu(&self, py: Python<'_>, menu: Option<ImplContextMenu>) -> PyResult<()> {
        py.allow_threads(|| match menu {
            Some(menu) => context_menu_impl!(&menu, |menu| {
                delegate_inner!(self, set_menu, Some(menu.to_owned()))
            }),
            None => delegate_inner!(self, set_menu, None::<tauri::menu::Menu<Runtime>>),
        })
    }

    #[pyo3(signature = (tooltip))]
    fn set_tooltip(&self, py: Python<'_>, tooltip: Option<&str>) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, set_tooltip, tooltip))
    }

    #[pyo3(signature = (title))]
    fn set_title(&self, py: Python<'_>, title: Option<&str>) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, set_title, title))
    }

    fn set_visible(&self, py: Python<'_>, visible: bool) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, set_visible, visible))
    }

    // PERF: `pyo3` didn't implement `FromPyObject` for `&path`,
    // see: <https://github.com/PyO3/pyo3/blob/2c732a7ab42af4b11c2a9a8da9f838b592712d95/src/conversions/std/path.rs#L22>
    #[pyo3(signature = (path))]
    fn set_temp_dir_path(&self, py: Python<'_>, path: Option<PathBuf>) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, set_temp_dir_path, path))
    }

    fn set_icon_as_template(&self, py: Python<'_>, is_template: bool) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, set_icon_as_template, is_template))
    }

    fn set_show_menu_on_left_click(&self, py: Python<'_>, enable: bool) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, set_show_menu_on_left_click, enable))
    }

    fn rect(&self, py: Python<'_>) -> PyResult<Option<Rect>> {
        let rect = py.allow_threads(|| delegate_inner!(self, rect,))?;
        match rect {
            Some(rect) => Ok(Some(Rect::from_tauri(py, rect)?)),
            None => Ok(None),
        }
    }
}

/// see also: [tauri::tray::TrayIconEvent::Click::position]
///
/// `tuple[x: float, y: float]`
pub struct PyPhysicalPositionF64(Py<PyTuple>);

impl PyPhysicalPositionF64 {
    pub(crate) fn from_tauri(
        py: Python<'_>,
        position: tauri::PhysicalPosition<f64>,
    ) -> PyResult<Self> {
        let x_y: (f64, f64) = (position.x, position.y);
        Ok(Self(x_y.into_pyobject(py)?.unbind()))
    }
}

impl FromPyObject<'_> for PyPhysicalPositionF64 {
    fn extract_bound(_: &Bound<'_, PyAny>) -> PyResult<Self> {
        unimplemented!("[TrayIconEvent::position] can't be constructed from Python")
    }
}

impl<'py> IntoPyObject<'py> for PyPhysicalPositionF64 {
    type Target = PyTuple;
    type Output = Bound<'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

impl<'a, 'py> IntoPyObject<'py> for &'a PyPhysicalPositionF64 {
    type Target = PyTuple;
    type Output = Borrowed<'a, 'py, Self::Target>;
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.bind_borrowed(py))
    }
}

/// see also: [tauri::tray::TrayIconEvent]
#[pyclass(frozen)]
#[non_exhaustive]
pub enum TrayIconEvent {
    // use `Py<T>` to avoid creating new obj every time visiting the field,
    // see: <https://pyo3.rs/v0.23.4/faq.html#pyo3get-clones-my-field>
    Click {
        id: Py<TrayIconId>,
        position: PyPhysicalPositionF64,
        rect: Py<Rect>,
        button: Py<MouseButton>,
        button_state: Py<MouseButtonState>,
    },
    DoubleClick {
        id: Py<TrayIconId>,
        position: PyPhysicalPositionF64,
        rect: Py<Rect>,
        button: Py<MouseButton>,
    },
    Enter {
        id: Py<TrayIconId>,
        position: PyPhysicalPositionF64,
        rect: Py<Rect>,
    },
    Move {
        id: Py<TrayIconId>,
        position: PyPhysicalPositionF64,
        rect: Py<Rect>,
    },
    Leave {
        id: Py<TrayIconId>,
        position: PyPhysicalPositionF64,
        rect: Py<Rect>,
    },
}

impl TrayIconEvent {
    pub(crate) fn from_tauri(py: Python<'_>, event: tray::TrayIconEvent) -> PyResult<Self> {
        fn from_rs_id(py: Python<'_>, id: tray::TrayIconId) -> Py<TrayIconId> {
            TrayIconId::intern(py, &id.0).unbind()
        }
        fn from_rs_position(
            py: Python<'_>,
            position: tauri::PhysicalPosition<f64>,
        ) -> PyResult<PyPhysicalPositionF64> {
            PyPhysicalPositionF64::from_tauri(py, position)
        }
        fn from_rs_rect(py: Python<'_>, rect: tauri::Rect) -> PyResult<Py<Rect>> {
            Ok(Rect::from_tauri(py, rect)?.into_pyobject(py)?.unbind())
        }
        fn from_rs_button(py: Python<'_>, button: tray::MouseButton) -> PyResult<Py<MouseButton>> {
            Ok(MouseButton::from(button).into_pyobject(py)?.unbind())
        }
        fn from_rs_button_state(
            py: Python<'_>,
            button_state: tray::MouseButtonState,
        ) -> PyResult<Py<MouseButtonState>> {
            Ok(MouseButtonState::from(button_state)
                .into_pyobject(py)?
                .unbind())
        }

        let event = match event {
            tray::TrayIconEvent::Click {
                id,
                position,
                rect,
                button,
                button_state,
            } => Self::Click {
                id: from_rs_id(py, id),
                position: from_rs_position(py, position)?,
                rect: from_rs_rect(py, rect)?,
                button: from_rs_button(py, button)?,
                button_state: from_rs_button_state(py, button_state)?,
            },
            tray::TrayIconEvent::DoubleClick {
                id,
                position,
                rect,
                button,
            } => Self::DoubleClick {
                id: from_rs_id(py, id),
                position: from_rs_position(py, position)?,
                rect: from_rs_rect(py, rect)?,
                button: from_rs_button(py, button)?,
            },
            tray::TrayIconEvent::Enter { id, position, rect } => Self::Enter {
                id: from_rs_id(py, id),
                position: from_rs_position(py, position)?,
                rect: from_rs_rect(py, rect)?,
            },
            tray::TrayIconEvent::Move { id, position, rect } => Self::Move {
                id: from_rs_id(py, id),
                position: from_rs_position(py, position)?,
                rect: from_rs_rect(py, rect)?,
            },
            tray::TrayIconEvent::Leave { id, position, rect } => Self::Leave {
                id: from_rs_id(py, id),
                position: from_rs_position(py, position)?,
                rect: from_rs_rect(py, rect)?,
            },
            event => {
                return Err(PyNotImplementedError::new_err(format!(
                    "Please make a issue for unimplemented TrayIconEvent: {event:?}",
                )))
            }
        };
        Ok(event)
    }
}

macro_rules! mouse_button_impl {
    ($ident:ident => : $($variant:ident),*) => {
        /// see also: [tauri::tray::MouseButton]
        #[pyclass(frozen, eq, eq_int)]
        #[derive(PartialEq, Clone, Copy)]
        pub enum $ident {
            $($variant,)*
        }

        impl From<tauri::tray::MouseButton> for $ident {
            fn from(val: tauri::tray::MouseButton) -> Self {
                match val {
                    $(tauri::tray::MouseButton::$variant => $ident::$variant,)*
                }
            }
        }

        impl From<$ident> for tauri::tray::MouseButton {
            fn from(val: $ident) -> Self {
                match val {
                    $($ident::$variant => tauri::tray::MouseButton::$variant,)*
                }
            }
        }
    };
}

mouse_button_impl! {
    MouseButton => :
    Left,
    Right,
    Middle
}

macro_rules! mouse_button_state_impl {
    ($ident:ident => : $($variant:ident),*) => {
        /// see also: [tauri::tray::MouseButtonState]
        #[pyclass(frozen, eq, eq_int)]
        #[derive(PartialEq, Clone, Copy)]
        pub enum $ident {
            $($variant,)*
        }

        impl From<tauri::tray::MouseButtonState> for $ident {
            fn from(val: tauri::tray::MouseButtonState) -> Self {
                match val {
                    $(tauri::tray::MouseButtonState::$variant => $ident::$variant,)*
                }
            }
        }

        impl From<$ident> for tauri::tray::MouseButtonState {
            fn from(val: $ident) -> Self {
                match val {
                    $($ident::$variant => tauri::tray::MouseButtonState::$variant,)*
                }
            }
        }
    };
}

mouse_button_state_impl! {
    MouseButtonState => :
    Up,
    Down
}
