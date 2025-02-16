use pyo3::{prelude::*, types::PyString};
use pyo3_utils::py_wrapper::{PyWrapper, PyWrapperT0};
use tauri::webview;

use crate::{
    context_menu_impl,
    ext_mod_impl::{
        image::Image,
        menu::{ImplContextMenu, Menu, MenuEvent},
        window::Window,
        Position,
    },
    tauri_runtime::Runtime,
    utils::TauriError,
};

pub(crate) type TauriWebviewWindow = webview::WebviewWindow<Runtime>;
type TauriWebview = webview::Webview<Runtime>;

/// see also: [tauri::webview::WebviewWindow]
#[pyclass(frozen)]
#[non_exhaustive]
pub struct WebviewWindow(pub PyWrapper<PyWrapperT0<TauriWebviewWindow>>);

impl WebviewWindow {
    pub(crate) fn new(webview_window: TauriWebviewWindow) -> Self {
        Self(PyWrapper::new0(webview_window))
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! delegate_inner {
    ($slf:expr, $func:ident, $($arg:expr),*) => {
        $slf.0
            .inner_ref()
            .$func($($arg),*)
            .map_err(|e| PyErr::from(TauriError::from(e)))
    };
}

#[pymethods]
impl WebviewWindow {
    fn run_on_main_thread(&self, py: Python<'_>, handler: PyObject) -> PyResult<()> {
        py.allow_threads(|| {
            delegate_inner!(self, run_on_main_thread, move || {
                Python::with_gil(|py| {
                    let handler = handler.bind(py);
                    let result = handler.call0();
                    if let Err(e) = result {
                        e.write_unraisable(py, Some(handler));
                        panic!("Python exception occurred in `WebviewWindow::run_on_main_thread`")
                    }
                })
            })
        })
    }

    fn label<'py>(&self, py: Python<'py>) -> Bound<'py, PyString> {
        let webview_window = self.0.inner_ref();
        // if `label` is immutable, we can intern it to save memory.
        PyString::intern(py, webview_window.label())
    }

    fn on_menu_event(slf: Py<Self>, py: Python<'_>, handler: PyObject) {
        let moved_slf = slf.clone_ref(py);
        py.allow_threads(|| {
            slf.get()
                .0
                .inner_ref()
                .on_menu_event(move |_window, menu_event| {
                    Python::with_gil(|py| {
                        // See: <https://github.com/tauri-apps/tauri/blob/8e9339e8807338597132ffd8688fb9da00f4102b/crates/tauri/src/app.rs#L2168-L2184>,
                        // The `window` argument is always the `WebviewWindow` instance that calls this method,
                        // so we can directly use the same PyObject.
                        let window: &Py<Self> = &moved_slf;  // TODO, XXX, FIXME: return `Window` instead of `WebviewWindow`?
                        debug_assert_eq!(&*window.get().0.inner_ref().as_ref().window_ref(), _window);
                        let menu_event: Bound<'_, MenuEvent> = MenuEvent::intern(py, &menu_event.id.0);

                        let handler = handler.bind(py);
                        let result = handler.call1((window, menu_event));
                        if let Err(e) = result {
                            e.write_unraisable(py, Some(handler));
                            panic!(
                                "Python exception occurred in `WebviewWindow::on_menu_event` handler"
                            )
                        }
                    })
                })
        })
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

    fn is_menu_visible(&self, py: Python<'_>) -> PyResult<bool> {
        py.allow_threads(|| delegate_inner!(self, is_menu_visible,))
    }

    fn popup_menu(&self, py: Python<'_>, menu: ImplContextMenu) -> PyResult<()> {
        py.allow_threads(|| {
            context_menu_impl!(&menu, |menu| delegate_inner!(self, popup_menu, menu))
        })
    }

    fn popup_menu_at(
        &self,
        py: Python<'_>,
        menu: ImplContextMenu,
        position: Position,
    ) -> PyResult<()> {
        py.allow_threads(|| {
            context_menu_impl!(&menu, |menu| delegate_inner!(
                self,
                popup_menu_at,
                menu,
                position
            ))
        })
    }

    fn is_fullscreen(&self, py: Python<'_>) -> PyResult<bool> {
        py.allow_threads(|| delegate_inner!(self, is_fullscreen,))
    }

    fn is_minimized(&self, py: Python<'_>) -> PyResult<bool> {
        py.allow_threads(|| delegate_inner!(self, is_minimized,))
    }

    fn is_maximized(&self, py: Python<'_>) -> PyResult<bool> {
        py.allow_threads(|| delegate_inner!(self, is_maximized,))
    }

    fn is_focused(&self, py: Python<'_>) -> PyResult<bool> {
        py.allow_threads(|| delegate_inner!(self, is_focused,))
    }

    fn is_decorated(&self, py: Python<'_>) -> PyResult<bool> {
        py.allow_threads(|| delegate_inner!(self, is_decorated,))
    }

    fn is_resizable(&self, py: Python<'_>) -> PyResult<bool> {
        py.allow_threads(|| delegate_inner!(self, is_resizable,))
    }

    fn is_enabled(&self, py: Python<'_>) -> PyResult<bool> {
        py.allow_threads(|| delegate_inner!(self, is_enabled,))
    }

    fn is_maximizable(&self, py: Python<'_>) -> PyResult<bool> {
        py.allow_threads(|| delegate_inner!(self, is_maximizable,))
    }

    fn is_minimizable(&self, py: Python<'_>) -> PyResult<bool> {
        py.allow_threads(|| delegate_inner!(self, is_minimizable,))
    }

    fn is_closable(&self, py: Python<'_>) -> PyResult<bool> {
        py.allow_threads(|| delegate_inner!(self, is_closable,))
    }

    fn is_visible(&self, py: Python<'_>) -> PyResult<bool> {
        py.allow_threads(|| delegate_inner!(self, is_visible,))
    }

    fn title(&self, py: Python<'_>) -> PyResult<String> {
        py.allow_threads(|| delegate_inner!(self, title,))
    }

    fn center(&self, py: Python<'_>) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, center,))
    }

    fn set_resizable(&self, py: Python<'_>, resizable: bool) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, set_resizable, resizable))
    }

    fn set_enabled(&self, py: Python<'_>, enabled: bool) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, set_enabled, enabled))
    }

    fn set_maximizable(&self, py: Python<'_>, maximizable: bool) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, set_maximizable, maximizable))
    }

    fn set_minimizable(&self, py: Python<'_>, minimizable: bool) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, set_minimizable, minimizable))
    }

    fn set_closable(&self, py: Python<'_>, closable: bool) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, set_closable, closable))
    }

    fn set_title(&self, py: Python<'_>, title: &str) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, set_title, title))
    }

    fn maximize(&self, py: Python<'_>) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, maximize,))
    }

    fn unmaximize(&self, py: Python<'_>) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, unmaximize,))
    }

    fn minimize(&self, py: Python<'_>) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, minimize,))
    }

    fn unminimize(&self, py: Python<'_>) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, unminimize,))
    }

    fn show(&self, py: Python<'_>) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, show,))
    }

    fn hide(&self, py: Python<'_>) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, hide,))
    }

    fn close(&self, py: Python<'_>) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, close,))
    }

    fn destroy(&self, py: Python<'_>) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, destroy,))
    }

    fn set_decorations(&self, py: Python<'_>, decorations: bool) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, set_decorations, decorations))
    }

    fn set_shadow(&self, py: Python<'_>, shadow: bool) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, set_shadow, shadow))
    }

    fn set_always_on_bottom(&self, py: Python<'_>, always_on_bottom: bool) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, set_always_on_bottom, always_on_bottom))
    }

    fn set_always_on_top(&self, py: Python<'_>, always_on_top: bool) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, set_always_on_top, always_on_top))
    }

    fn set_visible_on_all_workspaces(
        &self,
        py: Python<'_>,
        visible_on_all_workspaces: bool,
    ) -> PyResult<()> {
        py.allow_threads(|| {
            delegate_inner!(
                self,
                set_visible_on_all_workspaces,
                visible_on_all_workspaces
            )
        })
    }

    fn set_content_protected(&self, py: Python<'_>, protected: bool) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, set_content_protected, protected))
    }

    fn set_fullscreen(&self, py: Python<'_>, fullscreen: bool) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, set_fullscreen, fullscreen))
    }

    fn set_focus(&self, py: Python<'_>) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, set_focus,))
    }

    fn set_icon(&self, py: Python<'_>, icon: Py<Image>) -> PyResult<()> {
        let icon = icon.get().to_tauri(py);
        py.allow_threads(|| delegate_inner!(self, set_icon, icon))
    }

    fn set_skip_taskbar(&self, py: Python<'_>, skip: bool) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, set_skip_taskbar, skip))
    }

    fn set_cursor_grab(&self, py: Python<'_>, grab: bool) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, set_cursor_grab, grab))
    }

    fn set_cursor_visible(&self, py: Python<'_>, visible: bool) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, set_cursor_visible, visible))
    }

    fn set_ignore_cursor_events(&self, py: Python<'_>, ignore: bool) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, set_ignore_cursor_events, ignore))
    }

    fn start_dragging(&self, py: Python<'_>) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, start_dragging,))
    }

    #[pyo3(signature = (count))]
    fn set_badge_count(&self, py: Python<'_>, count: Option<i64>) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, set_badge_count, count))
    }

    fn print(&self, py: Python<'_>) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, print,))
    }

    fn url<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyString>> {
        let url = py.allow_threads(|| delegate_inner!(self, url,))?;
        Ok(PyString::new(py, url.as_ref()))
    }

    // // TODO, FIXME: Why `navigate` need `mut self`? We should ask tauri developers.
    // // see: <https://github.com/tauri-apps/tauri/issues/12430>
    // fn navigate(&self, url: &str) -> PyResult<()> {
    //     let url = Url::parse(url).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    //     delegate_inner!(mut self, navigate, url)
    // }

    fn eval(&self, py: Python<'_>, js: &str) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, eval, js))
    }

    fn set_zoom(&self, py: Python<'_>, scale_factor: f64) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, set_zoom, scale_factor))
    }

    fn clear_all_browsing_data(&self, py: Python<'_>) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, clear_all_browsing_data,))
    }

    /// see also: [tauri::webview::WebviewWindow::as_ref]
    fn as_ref_webview(&self) -> Webview {
        let webview = self.0.inner_ref().as_ref().clone();
        Webview::new(webview)
    }
}

/// see also: [tauri::webview::Webview]
#[pyclass(frozen)]
#[non_exhaustive]
pub struct Webview(pub PyWrapper<PyWrapperT0<TauriWebview>>);

impl Webview {
    pub(crate) fn new(webview: TauriWebview) -> Self {
        Self(PyWrapper::new0(webview))
    }
}

#[pymethods]
impl Webview {
    fn window(&self) -> Window {
        let window = self.0.inner_ref().window();
        Window::new(window)
    }
}
