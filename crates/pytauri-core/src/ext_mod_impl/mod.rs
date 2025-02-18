pub mod image;
pub mod ipc;
pub mod menu;
pub mod tray;
pub mod webview;
pub mod window;

use std::{
    borrow::Cow,
    collections::HashMap,
    convert::Infallible,
    error::Error,
    fmt::{Debug, Display},
};

use pyo3::{
    exceptions::PyNotImplementedError,
    exceptions::PyRuntimeError,
    marker::Ungil,
    prelude::*,
    types::{PyInt, PyString},
    IntoPyObject,
};
use pyo3_utils::{
    py_wrapper::{PyWrapper, PyWrapperT0, PyWrapperT2},
    ungil::UnsafeUngilExt,
};
use tauri::{Listener as _, Manager as _};

use crate::{
    delegate_inner,
    ext_mod_impl::{
        image::Image,
        menu::{Menu, MenuEvent},
        tray::{TrayIcon, TrayIconEvent},
        webview::{TauriWebviewWindow, WebviewWindow},
    },
    tauri_runtime::Runtime,
    utils::TauriError,
};

type TauriApp = tauri::App<Runtime>;
type TauriAppHandle = tauri::AppHandle<Runtime>;
type TauriContext = tauri::Context<Runtime>;

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
    fn new(py: Python<'_>, value: tauri::RunEvent) -> PyResult<Self> {
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

fn debug_assert_app_handle_py_is_rs(py_app_handle: &Py<AppHandle>, rs_app_handle: &TauriAppHandle) {
    if cfg!(debug_assertions) {
        let py_app_handle = py_app_handle.get().0.inner_ref();
        let py_app_handle_global = py_app_handle.py_app_handle();
        let rs_app_handle_global = rs_app_handle.py_app_handle();
        debug_assert!(
            py_app_handle_global.is(rs_app_handle_global),
            "AppHandle pyobject instance is not the same as the rust instance"
        );
    }
}

/// You can get the global singleton [Py]<[AppHandle]> using [PyAppHandleExt].
#[pyclass(frozen)]
#[non_exhaustive]
// NOTE: Do not use [PyWrapperT2], otherwise the global singleton [PyAppHandle]
// will be consumed and cannot be used;
// If you really need ownership of [tauri::AppHandle], you can use [tauri::AppHandle::clone].
pub struct AppHandle(pub PyWrapper<PyWrapperT0<TauriAppHandle>>);

impl AppHandle {
    /// NOTE: use [PyAppHandleExt] instead.
    fn new(app_handle: TauriAppHandle) -> Self {
        Self(PyWrapper::new0(app_handle))
    }
}

#[pymethods]
impl AppHandle {
    fn run_on_main_thread(&self, py: Python<'_>, handler: PyObject) -> PyResult<()> {
        py.allow_threads(|| {
            delegate_inner!(self, run_on_main_thread, move || {
                Python::with_gil(|py| {
                    let handler = handler.bind(py);
                    let result = handler.call0();
                    if let Err(e) = result {
                        e.write_unraisable(py, Some(handler));
                        panic!("Python exception occurred in `AppHandle::run_on_main_thread`")
                    }
                })
            })
        })
    }

    fn exit(&self, py: Python<'_>, exit_code: i32) {
        py.allow_threads(|| self.0.inner_ref().exit(exit_code))
    }

    /// NoReturn
    fn restart(&self, py: Python<'_>) {
        let _: Infallible = py.allow_threads(|| self.0.inner_ref().restart());
    }

    fn on_menu_event(slf: Py<Self>, py: Python<'_>, handler: PyObject) {
        let moved_slf = slf.clone_ref(py);
        py.allow_threads(|| {
            slf.get()
                .0
                .inner_ref()
                .on_menu_event(move |_app_handle, menu_event| {
                    Python::with_gil(|py| {
                        let app_handle: &Py<Self> = &moved_slf;
                        debug_assert_app_handle_py_is_rs(app_handle, _app_handle);
                        let menu_event: Bound<'_, MenuEvent> =
                            MenuEvent::intern(py, &menu_event.id.0);

                        let handler = handler.bind(py);
                        let result = handler.call1((app_handle, menu_event));
                        if let Err(e) = result {
                            e.write_unraisable(py, Some(handler));
                            panic!(
                                "Python exception occurred in `AppHandle::on_menu_event` handler"
                            )
                        }
                    })
                })
        })
    }

    fn on_tray_icon_event(slf: Py<Self>, py: Python<'_>, handler: PyObject) {
        let moved_slf = slf.clone_ref(py);
        py.allow_threads(|| {
            slf.get()
                .0
                .inner_ref()
                .on_tray_icon_event(move |_app_handle, tray_icon_event| {
                    Python::with_gil(|py| {
                        let app_handle: &Py<Self> = &moved_slf;
                        debug_assert_app_handle_py_is_rs(app_handle, _app_handle);
                        let tray_icon_event: TrayIconEvent =
                            TrayIconEvent::from_tauri(py, tray_icon_event)
                                // TODO: maybe we should only `write_unraisable` and log it instead of `panic` here?
                                .expect("Failed to convert rust `TrayIconEvent` to pyobject");

                        let handler = handler.bind(py);
                        let result = handler.call1((app_handle, tray_icon_event));
                        if let Err(e) = result {
                            e.write_unraisable(py, Some(handler));
                            panic!(
                                "Python exception occurred in `AppHandle::on_tray_icon_event` handler"
                            )
                        }
                    })
                })
        })
    }

    fn tray_by_id(&self, py: Python<'_>, id: &str) -> Option<TrayIcon> {
        py.allow_threads(|| self.0.inner_ref().tray_by_id(id).map(TrayIcon::new))
    }

    fn remove_tray_by_id(&self, py: Python<'_>, id: &str) -> Option<TrayIcon> {
        py.allow_threads(|| self.0.inner_ref().remove_tray_by_id(id).map(TrayIcon::new))
    }

    fn default_window_icon(&self, py: Python<'_>) -> Option<Image> {
        self.0
            .inner_ref()
            // this is not a blocking operation, so we don't need to release the GIL
            .default_window_icon()
            .map(|icon| Image::from_tauri(py, icon))
    }

    fn menu(&self, py: Python<'_>) -> Option<Menu> {
        py.allow_threads(|| self.0.inner_ref().menu().map(Menu::new))
    }

    fn set_menu(&self, py: Python<'_>, menu: Py<Menu>) -> PyResult<Option<Menu>> {
        py.allow_threads(|| {
            let menu = menu.get().0.inner_ref().clone();
            let returned_menu = delegate_inner!(self, set_menu, menu)?;
            PyResult::Ok(returned_menu.map(Menu::new))
        })
    }

    fn remove_menu(&self, py: Python<'_>) -> PyResult<Option<Menu>> {
        py.allow_threads(|| {
            let returned_menu = delegate_inner!(self, remove_menu,)?;
            PyResult::Ok(returned_menu.map(Menu::new))
        })
    }

    fn hide_menu(&self, py: Python<'_>) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, hide_menu,))
    }

    fn show_menu(&self, py: Python<'_>) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, show_menu,))
    }

    fn invoke_key<'py>(&self, py: Python<'py>) -> Bound<'py, PyString> {
        // if `invoke_key` is immutable, we can intern it to save memory.
        PyString::intern(py, self.0.inner_ref().invoke_key())
    }
}

/// This error indicates that the app was not initialized using [App::try_build],
/// i.e. it was not created by pytauri.
#[derive(Debug)]
pub struct PyAppHandleStateError;

impl Display for PyAppHandleStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to get `PyAppHandle` from state, maybe this app was not created by pytauri"
        )
    }
}

impl Error for PyAppHandleStateError {}

impl From<PyAppHandleStateError> for PyErr {
    fn from(value: PyAppHandleStateError) -> Self {
        PyRuntimeError::new_err(format!("{value}"))
    }
}

pub type PyAppHandleStateResult<T> = Result<T, PyAppHandleStateError>;

mod sealed {
    use super::*;

    pub trait SealedPyAppHandleExt {}
    impl<T: tauri::Manager<Runtime>> SealedPyAppHandleExt for T {}

    pub(super) struct PyAppHandle(pub(crate) Py<AppHandle>);
}

use sealed::{PyAppHandle, SealedPyAppHandleExt};

/// You can use this trait to get the global singleton [Py]<[AppHandle]>.
//
// NOTE: due to the unsoundness of [Manager::unmanage], do not allow to unmanage `PyAppHandle`,
// see: <https://github.com/tauri-apps/tauri/issues/12721>.
pub trait PyAppHandleExt: tauri::Manager<Runtime> + SealedPyAppHandleExt {
    /// See [PyAppHandleExt::try_py_app_handle] for details.
    ///
    /// # Panics
    ///
    /// Panics if [PyAppHandleExt::try_py_app_handle] returns an error.
    fn py_app_handle(&self) -> &Py<AppHandle> {
        self.try_py_app_handle().unwrap()
    }

    /// Get the global singleton [Py]<[AppHandle]>.
    ///
    /// If it has not been initialized, it will return an error.
    /// Use [PyAppHandleExt::get_or_init_py_app_handle] to initialize.
    fn try_py_app_handle(&self) -> PyAppHandleStateResult<&Py<AppHandle>> {
        let state = self
            .try_state::<PyAppHandle>()
            .ok_or(PyAppHandleStateError)?;
        Ok(&state.inner().0)
    }

    /// Get or initialize the global singleton [Py]<[AppHandle]>.
    ///
    /// It may return an error only during the first initialization.
    /// Once successfully called for the first time, subsequent calls will always return [Ok].
    ///
    /// [App::try_build] will call this method, which means if you already have an [App] instance,
    /// the [AppHandle] has also been initialized.
    fn get_or_init_py_app_handle(&self, py: Python<'_>) -> PyResult<&Py<AppHandle>> {
        match self.try_py_app_handle() {
            Ok(py_app_handle) => Ok(py_app_handle),
            Err(_) => {
                let py_app_handle = AppHandle::new(self.app_handle().to_owned());
                let py_app_handle = py_app_handle.into_pyobject(py)?.unbind();
                let not_yet_managed = self.manage::<PyAppHandle>(PyAppHandle(py_app_handle));
                debug_assert!(
                    not_yet_managed,
                    "`PyAppHandle` is private, so it is impossible for other crates to manage it, \
                    and for self crate, it should be initialized only once."
                );
                Ok(self
                    .try_py_app_handle()
                    .expect("`PyAppHandle` has already been initialized, so this never fail"))
            }
        }
    }
}

impl<T: tauri::Manager<Runtime>> PyAppHandleExt for T {}

#[pyclass(frozen, unsendable)]
#[non_exhaustive]
pub struct App(pub PyWrapper<PyWrapperT2<TauriApp>>);

impl App {
    #[cfg(feature = "__private")]
    pub fn try_build(py: Python<'_>, app: TauriApp) -> PyResult<Self> {
        // remember to initialize the global singleton [PyAppHandle], see it's doc
        app.get_or_init_py_app_handle(py)?;
        Ok(Self(PyWrapper::new2(app)))
    }

    fn py_cb_to_rs_cb(
        callback: PyObject,
        app_handle: Py<AppHandle>,
    ) -> impl FnMut(&TauriAppHandle, tauri::RunEvent) {
        move |_app_handle, run_event| {
            let py_app_handle: &Py<AppHandle> = &app_handle;
            debug_assert_app_handle_py_is_rs(&app_handle, _app_handle);

            Python::with_gil(|py| {
                let py_run_event: RunEvent = RunEvent::new(py, run_event)
                    // TODO: maybe we should only `write_unraisable` and log it instead of `panic` here?
                    .expect("Failed to convert rust `RunEvent` to pyobject");

                let callback = callback.bind(py);
                let result = callback.call1((py_app_handle, py_run_event));
                if let Err(e) = result {
                    // Use [write_unraisable] instead of [restore]:
                    // - Because we are about to panic, Python might abort
                    // - [restore] will not be handled in this case, so it will not be printed to stderr
                    e.write_unraisable(py, Some(callback));
                    // `panic` allows Python to exit `app.run()`,
                    // otherwise the Python main thread will be blocked by `app.run()`
                    // and unable to raise an error
                    panic!("Python exception occurred in callback")
                }
            })
        }
    }

    fn noop_callback(_: &TauriAppHandle, _: tauri::RunEvent) {}
}

#[pymethods]
impl App {
    #[pyo3(signature = (callback = None, /))]
    fn run(&self, py: Python<'_>, callback: Option<PyObject>) -> PyResult<()> {
        let app = self.0.try_take_inner()??;
        let py_app_handle = app.py_app_handle().clone_ref(py);
        unsafe {
            // `tauri::App` does not hold the GIL, so this is safe
            py.allow_threads_unsend(app, move |app| {
                match callback {
                    Some(callback) => app.run(Self::py_cb_to_rs_cb(callback, py_app_handle)),
                    None => app.run(Self::noop_callback),
                }
                Ok(())
            })
        }
    }

    #[pyo3(signature = (callback = None, /))]
    fn run_iteration(&self, py: Python<'_>, callback: Option<PyObject>) -> PyResult<()> {
        let app = self.0.try_lock_inner_mut()??;
        let py_app_handle = app.py_app_handle().clone_ref(py);
        unsafe {
            // `&mut tauri::App` does not hold the GIL, so this is safe
            py.allow_threads_unsend(app, |mut app| {
                match callback {
                    Some(callback) => {
                        app.run_iteration(Self::py_cb_to_rs_cb(callback, py_app_handle))
                    }
                    None => app.run_iteration(Self::noop_callback),
                }
                Ok(())
            })
        }
    }

    fn cleanup_before_exit(&self, py: Python<'_>) -> PyResult<()> {
        // `self: &App` does not hold the GIL, so this is safe
        unsafe {
            py.allow_threads_unsend(self, |slf| {
                let app = slf.0.try_lock_inner_ref()??;
                app.cleanup_before_exit();
                Ok(())
            })
        }
    }

    fn handle(&self, py: Python<'_>) -> PyResult<Py<AppHandle>> {
        let app = self.0.try_lock_inner_ref()??;
        // TODO, PERF: release the GIL?
        let app_handle = app.py_app_handle().clone_ref(py);
        Ok(app_handle)
    }
}

#[pyclass(frozen)]
#[non_exhaustive]
pub struct Context(pub PyWrapper<PyWrapperT2<TauriContext>>);

impl Context {
    pub fn new(context: TauriContext) -> Self {
        Self(PyWrapper::new2(context))
    }
}

/// The Implementers of [tauri::Manager].
#[derive(FromPyObject, IntoPyObject, IntoPyObjectRef)]
#[non_exhaustive]
// TODO: more types
pub enum ImplManager {
    App(Py<App>),
    AppHandle(Py<AppHandle>),
    WebviewWindow(Py<WebviewWindow>),
}

impl ImplManager {
    #[inline]
    pub(crate) fn _delegate_app<'py, R>(
        py: Python<'py>,
        app: &Py<App>,
        f: impl FnOnce(Python<'py>, &TauriApp) -> R,
    ) -> PyResult<R> {
        let py_app = app.borrow(py);
        let rs_app = py_app.0.try_lock_inner_ref()??;
        Ok(f(py, &rs_app))
    }

    #[inline]
    pub(crate) fn _delegate_app_handle<'py, R>(
        py: Python<'py>,
        app_handle: &Py<AppHandle>,
        f: impl FnOnce(Python<'py>, &TauriAppHandle) -> R,
    ) -> PyResult<R> {
        let app_handle = app_handle.get().0.inner_ref();
        Ok(f(py, &app_handle))
    }

    #[inline]
    pub(crate) fn _delegate_webview_window<'py, R>(
        py: Python<'py>,
        webview_window: &Py<WebviewWindow>,
        f: impl FnOnce(Python<'py>, &TauriWebviewWindow) -> R,
    ) -> PyResult<R> {
        let webview_window = webview_window.get().0.inner_ref();
        Ok(f(py, &webview_window))
    }

    #[inline]
    pub(crate) fn _delegate_manager_ungil<M, F, R>(py: Python<'_>, manager: &M, f: F) -> R
    where
        M: tauri::Manager<Runtime>,
        F: FnOnce(&M) -> R + Ungil + Send,
        R: Ungil,
    {
        unsafe {
            // safety: `tauri::Manager` does not hold the GIL, so this is safe
            py.allow_threads_unsend(manager, f)
        }
    }
}

/**
```ignore
fn manager_method_impl(py: Python<'_>, manager: &ImplManager) -> PyResult<()> {
    manager_method_impl!(py, manager, |_py, manager| {
        // here the `manager` is equivalent to `&impl tauri::Manager<Runtime>`
        manager.get_webview_window("main")
    })?;

    manager_method_impl!(py, manager, [ungil], |manager| {
        // here equivalent to `Python::allow_threads_unsend`
        manager.get_webview_window("main")
    })?;

    Ok(())
}
```
*/
#[doc(hidden)] // if export this macro, remember to enable doctest 👆
#[macro_export]
macro_rules! manager_method_impl {
    // impl
    ($py:expr, $manager:expr, $f0:expr, $f1:expr, $f2:expr) => {{
        use $crate::ext_mod_impl::ImplManager;

        let manager: &ImplManager = $manager;
        match manager {
            ImplManager::App(v) => {
                ImplManager::_delegate_app($py, v, $f0)
            }
            ImplManager::AppHandle(v) => {
                ImplManager::_delegate_app_handle($py, v, $f1)
            }
            ImplManager::WebviewWindow(v) => {
                ImplManager::_delegate_webview_window($py, v, $f2)
            }
        }
    }};

    // entry1 -> entry0
    ($py:expr, $manager:expr, [ungil], $($f:tt)*) => {{
        manager_method_impl!($py, $manager, |py, manager| {
            $crate::ext_mod_impl::ImplManager::_delegate_manager_ungil(py, manager, $($f)*)
        })
    }};
    // entry0
    ($py:expr, $manager:expr, $($f:tt)*) => {
        manager_method_impl!($py, $manager, $($f)*, $($f)*, $($f)*)
    };
}

/// See also: [tauri::Manager].
#[pyclass(frozen)]
#[non_exhaustive]
pub struct Manager;

#[pymethods]
impl Manager {
    #[staticmethod]
    fn app_handle(py: Python<'_>, slf: ImplManager) -> PyResult<Py<AppHandle>> {
        manager_method_impl!(py, &slf, |py, manager| manager
            // TODO, PERF: release the GIL?
            .py_app_handle()
            .clone_ref(py))
    }

    #[staticmethod]
    fn get_webview_window(
        py: Python<'_>,
        slf: ImplManager,
        label: &str,
    ) -> PyResult<Option<WebviewWindow>> {
        manager_method_impl!(py, &slf, [ungil], |manager| {
            manager.get_webview_window(label).map(WebviewWindow::new)
        })
    }

    #[staticmethod]
    fn webview_windows(
        py: Python<'_>,
        slf: ImplManager,
    ) -> PyResult<HashMap<String, WebviewWindow>> {
        manager_method_impl!(py, &slf, [ungil], |manager| {
            manager
                .webview_windows()
                .into_iter()
                .map(|(label, window)| (label, WebviewWindow::new(window)))
                .collect::<_>()
        })
    }
}

/// See also: [tauri::EventId].
pub use tauri::EventId;

/// See also: [tauri::Event].
#[pyclass(frozen)]
#[non_exhaustive]
pub struct Event {
    #[pyo3(get)]
    pub id: EventId,
    #[pyo3(get)]
    pub payload: Py<PyString>,
}

/// The Implementers of [tauri::Listener].
pub type ImplListener = ImplManager;

/// See also: [tauri::Listener].
#[pyclass(frozen)]
#[non_exhaustive]
pub struct Listener;

impl Listener {
    fn pyobj_to_handler(pyobj: PyObject) -> impl Fn(tauri::Event) + Send + 'static {
        move |event| {
            Python::with_gil(|py| {
                let event = Event {
                    id: event.id(),
                    payload: PyString::new(py, event.payload()).unbind(),
                };
                let pyobj = pyobj.bind(py);
                let result = pyobj.call1((event,));
                if let Err(e) = result {
                    e.write_unraisable(py, Some(pyobj));
                    panic!("Python exception occurred in Listener handler")
                }
            })
        }
    }
}

#[pymethods]
impl Listener {
    #[staticmethod]
    fn listen(
        py: Python<'_>,
        slf: ImplListener,
        event: Cow<'_, str>,
        handler: PyObject,
    ) -> PyResult<EventId> {
        manager_method_impl!(py, &slf, [ungil], |manager| manager
            .listen(event, Self::pyobj_to_handler(handler)))
    }

    #[staticmethod]
    fn once(
        py: Python<'_>,
        slf: ImplListener,
        event: Cow<'_, str>,
        handler: PyObject,
    ) -> PyResult<EventId> {
        manager_method_impl!(py, &slf, [ungil], |manager| manager
            .once(event, Self::pyobj_to_handler(handler)))
    }

    #[staticmethod]
    fn unlisten(py: Python<'_>, slf: ImplListener, id: EventId) -> PyResult<()> {
        manager_method_impl!(py, &slf, [ungil], |manager| manager.unlisten(id))
    }

    #[staticmethod]
    fn listen_any(
        py: Python<'_>,
        slf: ImplListener,
        event: Cow<'_, str>,
        handler: PyObject,
    ) -> PyResult<EventId> {
        manager_method_impl!(py, &slf, [ungil], |manager| manager
            .listen_any(event, Self::pyobj_to_handler(handler)))
    }

    #[staticmethod]
    fn once_any(
        py: Python<'_>,
        slf: ImplListener,
        event: Cow<'_, str>,
        handler: PyObject,
    ) -> PyResult<EventId> {
        manager_method_impl!(py, &slf, [ungil], |manager| manager
            .once_any(event, Self::pyobj_to_handler(handler)))
    }
}

/// see also: [tauri::Position]
#[derive(Clone, Copy)]
#[pyclass(frozen)]
pub enum Position {
    /// `x, y`
    Physical(i32, i32),
    /// `x, y`
    Logical(f64, f64),
}

impl From<Position> for tauri::Position {
    fn from(val: Position) -> Self {
        match val {
            Position::Physical(x, y) => tauri::PhysicalPosition::new(x, y).into(),
            Position::Logical(x, y) => tauri::LogicalPosition::new(x, y).into(),
        }
    }
}

impl From<tauri::Position> for Position {
    fn from(val: tauri::Position) -> Self {
        match val {
            tauri::Position::Physical(tauri::PhysicalPosition { x, y }) => Position::Physical(x, y),
            tauri::Position::Logical(tauri::LogicalPosition { x, y }) => Position::Logical(x, y),
        }
    }
}

/// see also: [tauri::Size]
#[derive(Clone, Copy)]
#[pyclass(frozen)]
pub enum Size {
    /// `width, height`
    Physical(u32, u32),
    /// `width, height`
    Logical(f64, f64),
}

impl From<Size> for tauri::Size {
    fn from(val: Size) -> Self {
        match val {
            Size::Physical(width, height) => tauri::PhysicalSize::new(width, height).into(),
            Size::Logical(width, height) => tauri::LogicalSize::new(width, height).into(),
        }
    }
}

impl From<tauri::Size> for Size {
    fn from(val: tauri::Size) -> Self {
        match val {
            tauri::Size::Physical(tauri::PhysicalSize { width, height }) => {
                Size::Physical(width, height)
            }
            tauri::Size::Logical(tauri::LogicalSize { width, height }) => {
                Size::Logical(width, height)
            }
        }
    }
}

/// see also: [tauri::Rect]
#[pyclass(frozen)]
pub struct Rect {
    // use `Py<T>` to avoid creating new obj every time visiting the field,
    // see: <https://pyo3.rs/v0.23.4/faq.html#pyo3get-clones-my-field>
    #[pyo3(get)]
    pub position: Py<Position>,
    #[pyo3(get)]
    pub size: Py<Size>,
}

impl Rect {
    #[expect(dead_code)] // TODO
    pub(crate) fn to_tauri(&self) -> tauri::Rect {
        tauri::Rect {
            position: (*self.position.get()).into(),
            size: (*self.size.get()).into(),
        }
    }

    pub(crate) fn from_tauri(py: Python<'_>, rect: tauri::Rect) -> PyResult<Self> {
        Ok(Self {
            position: Position::from(rect.position).into_pyobject(py)?.unbind(),
            size: Size::from(rect.size).into_pyobject(py)?.unbind(),
        })
    }
}

#[pymethods]
impl Rect {
    #[new]
    #[pyo3(signature = (*, position, size))]
    fn __new__(position: Py<Position>, size: Py<Size>) -> Self {
        Self { position, size }
    }
}
