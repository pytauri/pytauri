use std::{borrow::Cow, path::PathBuf};

use pyo3::{
    prelude::*,
    pybacked::PyBackedStr,
    types::{PyDict, PyString},
};
use pyo3_utils::{
    from_py_dict::{derive_from_py_dict, FromPyDict as _, NotRequired},
    py_wrapper::{PyWrapper, PyWrapperT0},
    serde::PySerde,
    ungil::UnsafeUngilExt,
};
use tauri::webview::{
    self,
    cookie::{self, time::OffsetDateTime},
};

use crate::{
    ext_mod::{
        image::Image,
        manager_method_impl,
        menu::{context_menu_impl, ImplContextMenu, Menu, MenuEvent},
        window::{Effects, Monitor, ProgressBarState, TitleBarStyle, Window},
        CursorIcon, ImplManager, PhysicalPositionF64, PhysicalPositionI32, PhysicalSizeU32,
        Position, Size, Theme, Url, UserAttentionType, WebviewEvent, WebviewUrl, WindowEvent,
    },
    tauri_runtime::Runtime,
    utils::{cfg_impl, delegate_inner, PyResultExt as _, TauriError},
};

pub(crate) type TauriWebviewWindow = webview::WebviewWindow<Runtime>;
type TauriWebview = webview::Webview<Runtime>;

/// See also: [tauri::utils::config::WindowConfig]
// TODO, PERF: use `&Config` to avoid clone,
// see [<PySerde as FromPyObject>::extract_bound] comment.
pub(crate) type WindowConfigFrom = PySerde<tauri::utils::config::WindowConfig>;
/// See also: [tauri::utils::config::WindowConfig]
#[expect(dead_code)] // TODO
pub(crate) type WindowConfigInto<'a> = PySerde<Cow<'a, tauri::utils::config::WindowConfig>>;

/// See also: [tauri::webview::WebviewWindow]
#[pyclass(frozen)]
#[non_exhaustive]
pub struct WebviewWindow(pub PyWrapper<PyWrapperT0<TauriWebviewWindow>>);

impl WebviewWindow {
    pub(crate) fn new(webview_window: TauriWebviewWindow) -> Self {
        Self(PyWrapper::new0(webview_window))
    }
}

#[pymethods]
impl WebviewWindow {
    // equivalent to `tauri::WebviewWindow::builder`
    #[new]
    #[pyo3(signature = (manager, label, url, /, **kwargs))]
    fn __new__(
        py: Python<'_>,
        manager: ImplManager,
        label: String,
        url: &Bound<'_, WebviewUrl>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Self> {
        WebviewWindowBuilder::build(py, manager, label, url, kwargs)
    }

    fn run_on_main_thread(&self, py: Python<'_>, handler: PyObject) -> PyResult<()> {
        py.allow_threads(|| {
            delegate_inner!(self, run_on_main_thread, move || {
                Python::with_gil(|py| {
                    let handler = handler.bind(py);
                    let result = handler.call0();
                    result.unwrap_unraisable_py_result(py, Some(handler), || {
                        "Python exception occurred in `WebviewWindow::run_on_main_thread`"
                    });
                })
            })
        })
    }

    fn label<'py>(&self, py: Python<'py>) -> Bound<'py, PyString> {
        let webview_window = self.0.inner_ref();
        // if `label` is immutable, we can intern it to save memory.
        PyString::intern(py, webview_window.label())
    }

    fn on_window_event(&self, py: Python<'_>, handler: PyObject) {
        py.allow_threads(|| {
            self.0.inner_ref().on_window_event(move |window_event| {
                Python::with_gil(|py| {
                    let window_event: WindowEvent = WindowEvent::from_tauri(py, window_event)
                        // TODO: maybe we should only `write_unraisable` and log it instead of `panic` here?
                        .expect("Failed to convert `WindowEvent` to pyobject");

                    let handler = handler.bind(py);
                    let result = handler.call1((window_event,));
                    result.unwrap_unraisable_py_result(py, Some(handler), || {
                        "Python exception occurred in `WebviewWindow::on_window_event` handler"
                    });
                })
            })
        })
    }

    fn on_webview_event(&self, py: Python<'_>, handler: PyObject) {
        py.allow_threads(|| {
            self.0.inner_ref().on_webview_event(move |webview_event| {
                Python::with_gil(|py| {
                    let webview_event: WebviewEvent = WebviewEvent::from_tauri(py, webview_event)
                        // TODO: maybe we should only `write_unraisable` and log it instead of `panic` here?
                        .expect("Failed to convert `WebviewEvent` to pyobject");

                    let handler = handler.bind(py);
                    let result = handler.call1((webview_event,));
                    result.unwrap_unraisable_py_result(py, Some(handler), || {
                        "Python exception occurred in `WebviewWindow::on_webview_event` handler"
                    });
                })
            })
        })
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
                        let window: &Py<Self> = &moved_slf; // TODO, XXX, FIXME: return `Window` instead of `WebviewWindow`?
                        debug_assert_eq!(
                            &*window.get().0.inner_ref().as_ref().window_ref(),
                            _window
                        );
                        let menu_event: Bound<'_, MenuEvent> =
                            MenuEvent::intern(py, &menu_event.id.0);

                        let handler = handler.bind(py);
                        let result = handler.call1((window, menu_event));
                        result.unwrap_unraisable_py_result(py, Some(handler), || {
                            "Python exception occurred in `WebviewWindow::on_menu_event` handler"
                        });
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
        position: Py<Position>,
    ) -> PyResult<()> {
        let position = position.get().to_tauri(py)?;
        py.allow_threads(|| {
            context_menu_impl!(&menu, |menu| delegate_inner!(
                self,
                popup_menu_at,
                menu,
                position
            ))
        })
    }

    fn scale_factor(&self, py: Python<'_>) -> PyResult<f64> {
        py.allow_threads(|| delegate_inner!(self, scale_factor,))
    }

    fn inner_position(&self, py: Python<'_>) -> PyResult<PhysicalPositionI32> {
        let position = py.allow_threads(|| delegate_inner!(self, inner_position,))?;
        PhysicalPositionI32::from_tauri(py, position)
    }

    fn outer_position(&self, py: Python<'_>) -> PyResult<PhysicalPositionI32> {
        let position = py.allow_threads(|| delegate_inner!(self, outer_position,))?;
        PhysicalPositionI32::from_tauri(py, position)
    }

    fn inner_size(&self, py: Python<'_>) -> PyResult<PhysicalSizeU32> {
        let size = py.allow_threads(|| delegate_inner!(self, inner_size,))?;
        PhysicalSizeU32::from_tauri(py, size)
    }

    fn outer_size(&self, py: Python<'_>) -> PyResult<PhysicalSizeU32> {
        let size = py.allow_threads(|| delegate_inner!(self, outer_size,))?;
        PhysicalSizeU32::from_tauri(py, size)
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

    fn is_always_on_top(&self, py: Python<'_>) -> PyResult<bool> {
        py.allow_threads(|| delegate_inner!(self, is_always_on_top,))
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

    fn current_monitor(&self, py: Python<'_>) -> PyResult<Option<Monitor>> {
        let monitor = py.allow_threads(|| delegate_inner!(self, current_monitor,))?;
        let monitor = monitor.map(|m| Monitor::from_tauri(py, m)).transpose()?;
        Ok(monitor)
    }

    fn primary_monitor(&self, py: Python<'_>) -> PyResult<Option<Monitor>> {
        let monitor = py.allow_threads(|| delegate_inner!(self, primary_monitor,))?;
        let monitor = monitor.map(|m| Monitor::from_tauri(py, m)).transpose()?;
        Ok(monitor)
    }

    fn monitor_from_point(&self, py: Python<'_>, x: f64, y: f64) -> PyResult<Option<Monitor>> {
        let monitor = py.allow_threads(|| delegate_inner!(self, monitor_from_point, x, y))?;
        let monitor = monitor.map(|m| Monitor::from_tauri(py, m)).transpose()?;
        Ok(monitor)
    }

    fn available_monitors(&self, py: Python<'_>) -> PyResult<Vec<Monitor>> {
        let monitors = py.allow_threads(|| delegate_inner!(self, available_monitors,))?;
        let monitors = monitors
            .into_iter()
            .map(|m| Monitor::from_tauri(py, m))
            .collect::<PyResult<Vec<_>>>()?;
        Ok(monitors)
    }

    fn theme(&self, py: Python<'_>) -> PyResult<Theme> {
        py.allow_threads(|| delegate_inner!(self, theme,).map(Into::into))
    }

    fn cursor_position(&self, py: Python<'_>) -> PyResult<PhysicalPositionF64> {
        let position = py.allow_threads(|| delegate_inner!(self, cursor_position,))?;
        PhysicalPositionF64::from_tauri(py, position)
    }

    fn center(&self, py: Python<'_>) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, center,))
    }

    fn request_user_attention(
        &self,
        py: Python<'_>,
        attention_type: Option<UserAttentionType>,
    ) -> PyResult<()> {
        py.allow_threads(|| {
            delegate_inner!(self, request_user_attention, attention_type.map(Into::into))
        })
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

    fn set_effects(&self, py: Python<'_>, effects: Option<Effects>) -> PyResult<()> {
        py.allow_threads(|| {
            let effects = effects.map(|e| e.into_tauri().build());
            delegate_inner!(self, set_effects, effects)
        })
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

    fn set_size(&self, py: Python<'_>, size: Py<Size>) -> PyResult<()> {
        let size = size.get().to_tauri(py)?;
        py.allow_threads(|| delegate_inner!(self, set_size, size))
    }

    fn set_min_size(&self, py: Python<'_>, size: Option<Py<Size>>) -> PyResult<()> {
        let size = size.map(|s| s.get().to_tauri(py)).transpose()?;
        py.allow_threads(|| delegate_inner!(self, set_min_size, size))
    }

    fn set_max_size(&self, py: Python<'_>, size: Option<Py<Size>>) -> PyResult<()> {
        let size = size.map(|s| s.get().to_tauri(py)).transpose()?;
        py.allow_threads(|| delegate_inner!(self, set_max_size, size))
    }

    // TODO: `set_size_constraints`, we need wait for tauri to expose `dpi::PixelUnit` first.
    // PR: <https://github.com/tauri-apps/tauri/pull/14009>

    fn set_position(&self, py: Python<'_>, position: Py<Position>) -> PyResult<()> {
        let position = position.get().to_tauri(py)?;
        py.allow_threads(|| delegate_inner!(self, set_position, position))
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

    fn set_background_color(&self, py: Python<'_>, color: Option<Color>) -> PyResult<()> {
        let color = color.map(|c| c.0);
        py.allow_threads(|| delegate_inner!(self, set_background_color, color))
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

    fn set_cursor_icon(&self, py: Python<'_>, icon: CursorIcon) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, set_cursor_icon, icon.into()))
    }

    fn set_cursor_position(&self, py: Python<'_>, position: Py<Position>) -> PyResult<()> {
        let position = position.get().to_tauri(py)?;
        py.allow_threads(|| delegate_inner!(self, set_cursor_position, position))
    }

    fn set_ignore_cursor_events(&self, py: Python<'_>, ignore: bool) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, set_ignore_cursor_events, ignore))
    }

    fn start_dragging(&self, py: Python<'_>) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, start_dragging,))
    }

    #[cfg(windows)]
    fn set_overlay_icon(&self, py: Python<'_>, icon: Option<Py<Image>>) -> PyResult<()> {
        let icon = icon.as_ref().map(|i| i.get().to_tauri(py));
        py.allow_threads(|| delegate_inner!(self, set_overlay_icon, icon))
    }

    fn set_badge_count(&self, py: Python<'_>, count: Option<i64>) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, set_badge_count, count))
    }

    #[cfg(target_os = "macos")]
    fn set_badge_label(&self, py: Python<'_>, label: Option<String>) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, set_badge_label, label))
    }

    fn set_progress_bar(&self, py: Python<'_>, progress_state: ProgressBarState) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, set_progress_bar, progress_state.into_tauri()))
    }

    fn set_title_bar_style(&self, py: Python<'_>, style: TitleBarStyle) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, set_title_bar_style, style.into()))
    }

    fn set_theme(&self, py: Python<'_>, theme: Option<Theme>) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, set_theme, theme.map(Into::into)))
    }

    fn print(&self, py: Python<'_>) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, print,))
    }

    fn url(&self, py: Python<'_>) -> PyResult<Url<'_>> {
        let url = py.allow_threads(|| delegate_inner!(self, url,))?;
        Ok(url.into())
    }

    fn navigate(&self, py: Python<'_>, url: Url<'_>) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, navigate, url.into()))
    }

    fn reload(&self, py: Python<'_>) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, reload,))
    }

    fn eval(&self, py: Python<'_>, js: &str) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, eval, js))
    }

    fn open_devtools(&self, py: Python<'_>) -> PyResult<()> {
        cfg_impl!(|any(debug_assertions, feature = "tauri-devtools")| -> () {
            py.allow_threads(|| {
                self.0.inner_ref().open_devtools();
                Ok(())
            })
        })
    }

    fn close_devtools(&self, py: Python<'_>) -> PyResult<()> {
        cfg_impl!(|any(debug_assertions, feature = "tauri-devtools")| -> () {
            py.allow_threads(|| {
                self.0.inner_ref().close_devtools();
                Ok(())
            })
        })
    }

    fn is_devtools_open(&self, py: Python<'_>) -> PyResult<bool> {
        cfg_impl!(|any(debug_assertions, feature = "tauri-devtools")| -> bool {
            py.allow_threads(|| {
                Ok(self.0.inner_ref().is_devtools_open())
            })
        })
    }

    fn set_zoom(&self, py: Python<'_>, scale_factor: f64) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, set_zoom, scale_factor))
    }

    fn clear_all_browsing_data(&self, py: Python<'_>) -> PyResult<()> {
        py.allow_threads(|| delegate_inner!(self, clear_all_browsing_data,))
    }

    fn cookies_for_url(&self, py: Python<'_>, url: Url<'_>) -> PyResult<Vec<Cookie>> {
        let cookies = py.allow_threads(|| delegate_inner!(self, cookies_for_url, url.into()))?;
        let cookies = cookies
            .into_iter()
            .map(|c| Cookie::from_tauri(py, &c))
            .collect::<Vec<_>>();
        Ok(cookies)
    }

    fn cookies(&self, py: Python<'_>) -> PyResult<Vec<Cookie>> {
        let cookies = py.allow_threads(|| delegate_inner!(self, cookies,))?;
        let cookies = cookies
            .into_iter()
            .map(|c| Cookie::from_tauri(py, &c))
            .collect::<Vec<_>>();
        Ok(cookies)
    }

    fn set_cookie(&self, py: Python<'_>, cookie: Cookie) -> PyResult<()> {
        let cookie = cookie.to_tauri(py)?;
        py.allow_threads(|| delegate_inner!(self, set_cookie, cookie))
    }

    fn delete_cookie(&self, py: Python<'_>, cookie: Cookie) -> PyResult<()> {
        let cookie = cookie.to_tauri(py)?;
        py.allow_threads(|| delegate_inner!(self, delete_cookie, cookie))
    }

    /// See also: [tauri::webview::WebviewWindow::as_ref]
    fn as_ref_webview(&self) -> Webview {
        let webview = self.0.inner_ref().as_ref().clone();
        Webview::new(webview)
    }

    // TODO: `as_ref_windows`, see <https://github.com/tauri-apps/tauri/pull/14012>
}

/// See also: [tauri::webview::Webview]
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

/// See also: [tauri::webview::Color]
///
/// `(r, g, b, a)`
pub struct Color(pub(crate) webview::Color);

impl<'py> FromPyObject<'py> for Color {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let (r, g, b, a): (u8, u8, u8, u8) = ob.extract()?;
        Ok(Self(webview::Color(r, g, b, a)))
    }
}

macro_rules! same_site_impl {
    ($ident:ident => : $( $(#[$meta:meta])* $variant:ident ),*) => {
        /// See also: [cookie::SameSite]
        #[pyclass(frozen, eq, eq_int)]
        #[derive(PartialEq, Clone, Copy)]
        #[non_exhaustive]
        pub enum $ident {
            $(
                $(#[$meta])*
                $variant,
            )*
        }

        impl From<cookie::SameSite> for $ident {
            fn from(val: cookie::SameSite) -> Self {
                match val {
                    $(cookie::SameSite::$variant => $ident::$variant,)*
                }
            }
        }

        impl From<$ident> for cookie::SameSite {
            fn from(val: $ident) -> Self {
                match val {
                    $($ident::$variant => cookie::SameSite::$variant,)*
                }
            }
        }
    };
}

same_site_impl!(SameSite => : Strict, Lax, #[pyo3(name = "None_")] None);

// NOTE: we need to implement this manually
// because [NotRequired::into_py_with_none] requires `'&T: IntoPyObject`
impl<'py> IntoPyObject<'py> for &SameSite {
    type Error = <SameSite as IntoPyObject<'py>>::Error;
    type Output = <SameSite as IntoPyObject<'py>>::Output;
    type Target = <SameSite as IntoPyObject<'py>>::Target;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        <SameSite as IntoPyObject<'py>>::into_pyobject(*self, py)
    }
}

/// See also: [tauri::webview::Cookie]
// ref:
// - <https://github.com/encode/starlette/blob/6ee94f2cac955eeae68d2898a8dec8cf17b48736/starlette/responses.py#L91-L103>
// - <https://docs.python.org/3.14/library/http.cookies.html#http.cookies.Morsel>
//
// TODO: [IntoPyObject] does not use `pyo3::intern`, we should file an issue to pyo3
// TODO: Submit a feature request to pyo3 to add `#[pyo3(skip_if)]` for skipping certain fields
#[derive(IntoPyObject, IntoPyObjectRef)]
pub struct Cookie {
    key: Py<PyString>,
    value: Py<PyString>,
    max_age: Option<i64>,
    expires: Option<OffsetDateTime>,
    path: Option<Py<PyString>>,
    domain: Option<Py<PyString>>,
    secure: Option<bool>,
    httponly: Option<bool>,
    samesite: Option<SameSite>,
    partitioned: Option<bool>,
}

derive_from_py_dict!(Cookie {
    key,
    value,
    #[pyo3(default)]
    max_age,
    #[pyo3(default)]
    expires,
    #[pyo3(default)]
    path,
    #[pyo3(default)]
    domain,
    #[pyo3(default)]
    secure,
    #[pyo3(default)]
    httponly,
    #[pyo3(default)]
    samesite,
    #[pyo3(default)]
    partitioned,
});

impl<'py> FromPyObject<'py> for Cookie {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let dict = ob.downcast::<PyDict>()?;
        Self::from_py_dict(dict)
    }
}

impl Cookie {
    pub(crate) fn from_tauri(py: Python<'_>, cookie: &webview::Cookie<'_>) -> Self {
        let key = PyString::new(py, cookie.name()).unbind();
        let value = PyString::new(py, cookie.value()).unbind();
        let max_age = cookie.max_age().map(|d| d.whole_seconds());
        let expires = cookie.expires_datetime();
        let path = cookie.path().map(|p| PyString::new(py, p).unbind());
        let domain = cookie.domain().map(|d| PyString::new(py, d).unbind());
        let secure = cookie.secure();
        let httponly = cookie.http_only();
        let samesite = cookie.same_site().map(|s| s.into());
        let partitioned = cookie.partitioned();
        Self {
            key,
            value,
            max_age,
            expires,
            path,
            domain,
            secure,
            httponly,
            samesite,
            partitioned,
        }
    }

    pub(crate) fn to_tauri(&self, py: Python<'_>) -> PyResult<webview::Cookie<'_>> {
        let Self {
            key,
            value,
            max_age,
            expires,
            path,
            domain,
            secure,
            httponly,
            samesite,
            partitioned,
        } = self;

        // TODO, PERF: once we drop py39 support, we can use [PyStringMethods::to_str] directly.
        let key = key.to_cow(py)?;
        let value = value.to_cow(py)?;
        let mut cookie_builder = cookie::CookieBuilder::new(key, value);

        if let Some(max_age) = max_age {
            cookie_builder = cookie_builder.max_age(cookie::time::Duration::seconds(*max_age));
        }
        if let Some(expires) = expires {
            cookie_builder = cookie_builder.expires(*expires);
        }
        if let Some(path) = path {
            cookie_builder = cookie_builder.path(path.to_cow(py)?);
        }
        if let Some(domain) = domain {
            cookie_builder = cookie_builder.domain(domain.to_cow(py)?);
        }
        if let Some(secure) = secure {
            cookie_builder = cookie_builder.secure(*secure);
        }
        if let Some(httponly) = httponly {
            cookie_builder = cookie_builder.http_only(*httponly);
        }
        if let Some(samesite) = samesite {
            cookie_builder = cookie_builder.same_site((*samesite).into());
        }
        if let Some(partitioned) = partitioned {
            cookie_builder = cookie_builder.partitioned(*partitioned);
        }
        Ok(cookie_builder.build())
    }
}

/// See also: [tauri::webview::WebviewWindowBuilder]
#[non_exhaustive]
pub struct WebviewWindowBuilderArgs {
    // TODO, FIXME: on_menu_event: NotRequired<PyObject>,
    // `on_menu_event` passes `Window` instead of `WebviewWindow`,
    // we need to submit a PR to tauri to modify the signature,
    // or add a method to get `WebviewWindow` from `Window`

    // TODO: on_web_resource_request
    on_navigation: NotRequired<PyObject>,
    // TODO: on_new_window
    on_document_title_changed: NotRequired<PyObject>,
    // TODO: on_download, on_page_load
    menu: NotRequired<Py<Menu>>,
    center: NotRequired<bool>,
    position: NotRequired<(f64, f64)>,
    inner_size: NotRequired<(f64, f64)>,
    min_inner_size: NotRequired<(f64, f64)>,
    max_inner_size: NotRequired<(f64, f64)>,
    // TODO: inner_size_constraints
    prevent_overflow: NotRequired<bool>,
    prevent_overflow_with_margin: NotRequired<Py<Size>>,
    resizable: NotRequired<bool>,
    maximizable: NotRequired<bool>,
    minimizable: NotRequired<bool>,
    closable: NotRequired<bool>,
    title: NotRequired<String>,
    fullscreen: NotRequired<bool>,
    focusable: NotRequired<bool>,
    focused: NotRequired<bool>,
    maximized: NotRequired<bool>,
    visible: NotRequired<bool>,
    theme: NotRequired<Option<Theme>>,
    decorations: NotRequired<bool>,
    always_on_bottom: NotRequired<bool>,
    always_on_top: NotRequired<bool>,
    visible_on_all_workspaces: NotRequired<bool>,
    content_protected: NotRequired<bool>,
    icon: NotRequired<Py<Image>>,
    skip_taskbar: NotRequired<bool>,
    window_classname: NotRequired<String>,
    shadow: NotRequired<bool>,
    parent: NotRequired<Py<WebviewWindow>>,
    #[cfg(windows)]
    owner: NotRequired<Py<WebviewWindow>>,
    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    transient_for: NotRequired<Py<WebviewWindow>>,
    #[cfg(windows)]
    drag_and_drop: NotRequired<bool>,
    #[cfg(target_os = "macos")]
    title_bar_style: NotRequired<TitleBarStyle>,
    #[cfg(target_os = "macos")]
    traffic_light_position: NotRequired<Py<Position>>,
    #[cfg(target_os = "macos")]
    allow_link_preview: NotRequired<bool>,
    #[cfg(target_os = "macos")]
    hidden_title: NotRequired<bool>,
    #[cfg(target_os = "macos")]
    tabbing_identifier: NotRequired<PyBackedStr>,
    effects: NotRequired<Effects>,
    accept_first_mouse: NotRequired<bool>,
    initialization_script: NotRequired<String>,
    initialization_script_for_all_frames: NotRequired<String>,
    user_agent: NotRequired<PyBackedStr>,
    additional_browser_args: NotRequired<PyBackedStr>,
    data_directory: NotRequired<PathBuf>,
    disable_drag_drop_handler: NotRequired<bool>,
    enable_clipboard_access: NotRequired<bool>,
    incognito: NotRequired<bool>,
    auto_resize: NotRequired<bool>,
    // TODO, PERF: remove `'static` bound
    proxy_url: NotRequired<Url<'static>>,
    transparent: NotRequired<bool>,
    zoom_hotkeys_enabled: NotRequired<bool>,
    browser_extensions_enabled: NotRequired<bool>,
    extensions_path: NotRequired<PathBuf>,
    // TODO: data_store_identifier
    use_https_scheme: NotRequired<bool>,
    devtools: NotRequired<bool>,
    background_color: NotRequired<Color>,
    // TODO: BackgroundThrottlingPolicy
    disable_javascript: NotRequired<bool>,
    // TODO: window_features
}

derive_from_py_dict!(WebviewWindowBuilderArgs {
    #[pyo3(default)]
    on_navigation,
    #[pyo3(default)]
    on_document_title_changed,
    #[pyo3(default)]
    menu,
    #[pyo3(default)]
    center,
    #[pyo3(default)]
    position,
    #[pyo3(default)]
    inner_size,
    #[pyo3(default)]
    min_inner_size,
    #[pyo3(default)]
    max_inner_size,
    #[pyo3(default)]
    prevent_overflow,
    #[pyo3(default)]
    prevent_overflow_with_margin,
    #[pyo3(default)]
    resizable,
    #[pyo3(default)]
    maximizable,
    #[pyo3(default)]
    minimizable,
    #[pyo3(default)]
    closable,
    #[pyo3(default)]
    title,
    #[pyo3(default)]
    fullscreen,
    #[pyo3(default)]
    focusable,
    #[pyo3(default)]
    focused,
    #[pyo3(default)]
    maximized,
    #[pyo3(default)]
    visible,
    #[pyo3(default)]
    theme,
    #[pyo3(default)]
    decorations,
    #[pyo3(default)]
    always_on_bottom,
    #[pyo3(default)]
    always_on_top,
    #[pyo3(default)]
    visible_on_all_workspaces,
    #[pyo3(default)]
    content_protected,
    #[pyo3(default)]
    icon,
    #[pyo3(default)]
    skip_taskbar,
    #[pyo3(default)]
    window_classname,
    #[pyo3(default)]
    shadow,
    #[pyo3(default)]
    parent,
    #[cfg(windows)]
    #[pyo3(default)]
    owner,
    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    #[pyo3(default)]
    transient_for,
    #[cfg(windows)]
    #[pyo3(default)]
    drag_and_drop,
    #[cfg(target_os = "macos")]
    #[pyo3(default)]
    title_bar_style,
    #[cfg(target_os = "macos")]
    #[pyo3(default)]
    traffic_light_position,
    #[cfg(target_os = "macos")]
    #[pyo3(default)]
    allow_link_preview,
    #[cfg(target_os = "macos")]
    #[pyo3(default)]
    hidden_title,
    #[cfg(target_os = "macos")]
    #[pyo3(default)]
    tabbing_identifier,
    #[pyo3(default)]
    effects,
    #[pyo3(default)]
    accept_first_mouse,
    #[pyo3(default)]
    initialization_script,
    #[pyo3(default)]
    initialization_script_for_all_frames,
    #[pyo3(default)]
    user_agent,
    #[pyo3(default)]
    additional_browser_args,
    #[pyo3(default)]
    data_directory,
    #[pyo3(default)]
    disable_drag_drop_handler,
    #[pyo3(default)]
    enable_clipboard_access,
    #[pyo3(default)]
    incognito,
    #[pyo3(default)]
    auto_resize,
    #[pyo3(default)]
    proxy_url,
    #[pyo3(default)]
    transparent,
    #[pyo3(default)]
    zoom_hotkeys_enabled,
    #[pyo3(default)]
    browser_extensions_enabled,
    #[pyo3(default)]
    extensions_path,
    #[pyo3(default)]
    use_https_scheme,
    #[pyo3(default)]
    devtools,
    #[pyo3(default)]
    background_color,
    #[pyo3(default)]
    disable_javascript,
});

impl WebviewWindowBuilderArgs {
    fn from_kwargs(kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<Option<Self>> {
        kwargs.map(Self::from_py_dict).transpose()
    }

    fn apply_to_builder<'a, M>(
        self,
        py: Python<'_>,
        mut builder: webview::WebviewWindowBuilder<'a, Runtime, M>,
    ) -> PyResult<webview::WebviewWindowBuilder<'a, Runtime, M>>
    where
        M: tauri::Manager<Runtime>,
    {
        let Self {
            on_navigation,
            on_document_title_changed,
            menu,
            center,
            position,
            inner_size,
            min_inner_size,
            max_inner_size,
            prevent_overflow,
            prevent_overflow_with_margin,
            resizable,
            maximizable,
            minimizable,
            closable,
            title,
            fullscreen,
            focusable,
            focused,
            maximized,
            visible,
            theme,
            decorations,
            always_on_bottom,
            always_on_top,
            visible_on_all_workspaces,
            content_protected,
            icon,
            skip_taskbar,
            window_classname,
            shadow,
            parent,
            #[cfg(windows)]
            owner,
            #[cfg(any(
                target_os = "linux",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "netbsd",
                target_os = "openbsd"
            ))]
            transient_for,
            #[cfg(windows)]
            drag_and_drop,
            #[cfg(target_os = "macos")]
            title_bar_style,
            #[cfg(target_os = "macos")]
            traffic_light_position,
            #[cfg(target_os = "macos")]
            allow_link_preview,
            #[cfg(target_os = "macos")]
            hidden_title,
            #[cfg(target_os = "macos")]
            tabbing_identifier,
            effects,
            accept_first_mouse,
            initialization_script,
            initialization_script_for_all_frames,
            user_agent,
            additional_browser_args,
            data_directory,
            disable_drag_drop_handler,
            enable_clipboard_access,
            incognito,
            auto_resize,
            proxy_url,
            transparent,
            zoom_hotkeys_enabled,
            browser_extensions_enabled,
            extensions_path,
            use_https_scheme,
            devtools,
            background_color,
            disable_javascript,
        } = self;

        if let Some(on_navigation) = on_navigation.0 {
            builder = builder.on_navigation(move |url| {
                Python::with_gil(|py| {
                    let url = Url::from(url);

                    let handler = on_navigation.bind(py);
                    let ret = handler
                        .call1((url,))
                        .unwrap_unraisable_py_result(py, Some(handler), || {
                            "Python exception occurred in `WebviewWindowBuilder::on_navigation` handler"
                        });
                    ret.extract::<bool>()
                        .unwrap_unraisable_py_result(py, Some(&ret), || {
                            "`WebviewWindowBuilder::on_navigation` return non-bool value"
                        })
                })
            });
        };
        if let Some(on_document_title_changed) = on_document_title_changed.0 {
            builder = builder.on_document_title_changed(move |webview_window, title| {
                Python::with_gil(|py| {
                    let webview_window = WebviewWindow::new(webview_window);

                    let handler = on_document_title_changed.bind(py);
                    handler
                        .call1((webview_window, title))
                        .unwrap_unraisable_py_result(py, Some(handler), || {
                            "Python exception occurred in `WebviewWindowBuilder::on_document_title_changed` handler"
                        });
                })
            });
        };
        if let Some(menu) = menu.0 {
            let menu = menu.get().0.inner_ref().clone();
            builder = builder.menu(menu);
        }
        if let Some(true) = center.0 {
            builder = builder.center();
        }
        if let Some((x, y)) = position.0 {
            builder = builder.position(x, y);
        }
        if let Some((width, height)) = inner_size.0 {
            builder = builder.inner_size(width, height);
        }
        if let Some((min_width, min_height)) = min_inner_size.0 {
            builder = builder.min_inner_size(min_width, min_height);
        }
        if let Some((max_width, max_height)) = max_inner_size.0 {
            builder = builder.max_inner_size(max_width, max_height);
        }
        if let Some(true) = prevent_overflow.0 {
            builder = builder.prevent_overflow();
        }
        if let Some(margin) = prevent_overflow_with_margin.0 {
            let margin = margin.get().to_tauri(py)?;
            builder = builder.prevent_overflow_with_margin(margin);
        }
        if let Some(resizable) = resizable.0 {
            builder = builder.resizable(resizable);
        }
        if let Some(maximizable) = maximizable.0 {
            builder = builder.maximizable(maximizable);
        }
        if let Some(minimizable) = minimizable.0 {
            builder = builder.minimizable(minimizable);
        }
        if let Some(closable) = closable.0 {
            builder = builder.closable(closable);
        }
        if let Some(title) = title.0 {
            builder = builder.title(title);
        }
        if let Some(fullscreen) = fullscreen.0 {
            builder = builder.fullscreen(fullscreen);
        }
        if let Some(focusable) = focusable.0 {
            builder = builder.focusable(focusable);
        }
        if let Some(focused) = focused.0 {
            builder = builder.focused(focused);
        }
        if let Some(maximized) = maximized.0 {
            builder = builder.maximized(maximized);
        }
        if let Some(visible) = visible.0 {
            builder = builder.visible(visible);
        }
        if let Some(theme) = theme.0 {
            builder = builder.theme(theme.map(Into::into));
        }
        if let Some(decorations) = decorations.0 {
            builder = builder.decorations(decorations);
        }
        if let Some(always_on_bottom) = always_on_bottom.0 {
            builder = builder.always_on_bottom(always_on_bottom);
        }
        if let Some(always_on_top) = always_on_top.0 {
            builder = builder.always_on_top(always_on_top);
        }
        if let Some(visible_on_all_workspaces) = visible_on_all_workspaces.0 {
            builder = builder.visible_on_all_workspaces(visible_on_all_workspaces);
        }
        if let Some(content_protected) = content_protected.0 {
            builder = builder.content_protected(content_protected);
        }
        if let Some(icon) = icon.0 {
            // TODO, PERF, FIXME: avoid clone.
            // This is a signature error of tauri.
            // The signature should have been `icon: Image<'_>`,
            // but instead it is `icon: Image<'a>`. We need to submit a PR.
            let icon = icon.get().to_tauri(py).to_owned();
            builder = builder.icon(icon).map_err(TauriError::from)?;
        }
        if let Some(skip_taskbar) = skip_taskbar.0 {
            builder = builder.skip_taskbar(skip_taskbar);
        }
        if let Some(window_classname) = window_classname.0 {
            builder = builder.window_classname(window_classname);
        }
        if let Some(shadow) = shadow.0 {
            builder = builder.shadow(shadow);
        }
        if let Some(parent) = parent.0 {
            let parent = parent.get().0.inner_ref();
            builder = builder.parent(&parent).map_err(TauriError::from)?;
        }
        #[cfg(windows)]
        if let Some(owner) = owner.0 {
            let owner = owner.get().0.inner_ref();
            builder = builder.owner(&owner).map_err(TauriError::from)?;
        }
        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        if let Some(transient_for) = transient_for.0 {
            let transient_for = transient_for.get().0.inner_ref();
            builder = builder
                .transient_for(&transient_for)
                .map_err(TauriError::from)?;
        }
        #[cfg(windows)]
        if let Some(drag_and_drop) = drag_and_drop.0 {
            builder = builder.drag_and_drop(drag_and_drop);
        }
        #[cfg(target_os = "macos")]
        if let Some(title_bar_style) = title_bar_style.0 {
            builder = builder.title_bar_style(title_bar_style.into());
        }
        #[cfg(target_os = "macos")]
        if let Some(traffic_light_position) = traffic_light_position.0 {
            let traffic_light_position = traffic_light_position.get().to_tauri(py)?;
            builder = builder.traffic_light_position(traffic_light_position);
        }
        #[cfg(target_os = "macos")]
        if let Some(allow_link_preview) = allow_link_preview.0 {
            builder = builder.allow_link_preview(allow_link_preview);
        }
        #[cfg(target_os = "macos")]
        if let Some(hidden_title) = hidden_title.0 {
            builder = builder.hidden_title(hidden_title);
        }
        #[cfg(target_os = "macos")]
        if let Some(tabbing_identifier) = tabbing_identifier.0 {
            builder = builder.tabbing_identifier(&tabbing_identifier);
        }
        if let Some(effects) = effects.0 {
            let effects = effects.into_tauri().build();
            builder = builder.effects(effects);
        }
        if let Some(accept_first_mouse) = accept_first_mouse.0 {
            builder = builder.accept_first_mouse(accept_first_mouse);
        }
        if let Some(initialization_script) = initialization_script.0 {
            builder = builder.initialization_script(initialization_script);
        }
        if let Some(initialization_script_for_all_frames) = initialization_script_for_all_frames.0 {
            builder =
                builder.initialization_script_for_all_frames(initialization_script_for_all_frames);
        }
        if let Some(user_agent) = user_agent.0 {
            builder = builder.user_agent(&user_agent);
        }
        if let Some(additional_browser_args) = additional_browser_args.0 {
            builder = builder.additional_browser_args(&additional_browser_args);
        }
        if let Some(data_directory) = data_directory.0 {
            builder = builder.data_directory(data_directory);
        }
        if let Some(true) = disable_drag_drop_handler.0 {
            builder = builder.disable_drag_drop_handler();
        }
        if let Some(true) = enable_clipboard_access.0 {
            builder = builder.enable_clipboard_access();
        }
        if let Some(incognito) = incognito.0 {
            builder = builder.incognito(incognito);
        }
        if let Some(true) = auto_resize.0 {
            builder = builder.auto_resize();
        }
        if let Some(proxy_url) = proxy_url.0 {
            builder = builder.proxy_url(proxy_url.into());
        }
        builder = cfg_impl!(
            |any(not(target_os = "macos"), feature = "tauri-macos-private-api")| -> _ {
                if let Some(transparent) = transparent.0 {
                    builder = builder.transparent(transparent);
                }
                Ok(builder)
            }
        )?;
        if let Some(zoom_hotkeys_enabled) = zoom_hotkeys_enabled.0 {
            builder = builder.zoom_hotkeys_enabled(zoom_hotkeys_enabled);
        }
        if let Some(browser_extensions_enabled) = browser_extensions_enabled.0 {
            builder = builder.browser_extensions_enabled(browser_extensions_enabled);
        }
        if let Some(extensions_path) = extensions_path.0 {
            builder = builder.extensions_path(extensions_path);
        }
        if let Some(use_https_scheme) = use_https_scheme.0 {
            builder = builder.use_https_scheme(use_https_scheme);
        }
        if let Some(devtools) = devtools.0 {
            builder = builder.devtools(devtools);
        }
        if let Some(background_color) = background_color.0 {
            builder = builder.background_color(background_color.0);
        }
        if let Some(true) = disable_javascript.0 {
            builder = builder.disable_javascript();
        }

        Ok(builder)
    }
}

/// See also: [tauri::webview::WebviewWindowBuilder]
#[pyclass(frozen)]
#[non_exhaustive]
pub struct WebviewWindowBuilder;

#[pymethods]
impl WebviewWindowBuilder {
    #[staticmethod]
    #[pyo3(signature = (manager, label, url, /, **kwargs))]
    fn build(
        py: Python<'_>,
        manager: ImplManager,
        label: String,
        url: &Bound<'_, WebviewUrl>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<WebviewWindow> {
        let url = url.get().to_tauri()?;
        let args = WebviewWindowBuilderArgs::from_kwargs(kwargs)?;
        manager_method_impl!(py, &manager, move |py, manager| {
            let mut builder = webview::WebviewWindowBuilder::new(manager, label, url);
            if let Some(args) = args {
                builder = args.apply_to_builder(py, builder)?;
            }

            let webview_window = unsafe {
                // Safety: `WebviewWindowBuilder` does not hold the GIL, so this is safe
                py.allow_threads_unsend(builder, |builder| builder.build())
            }
            .map_err(TauriError::from)?;

            PyResult::Ok(WebviewWindow::new(webview_window))
        })?
    }

    #[staticmethod]
    #[pyo3(signature = (manager, config, /, **kwargs))]
    fn from_config(
        py: Python<'_>,
        manager: ImplManager,
        config: WindowConfigFrom,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<WebviewWindow> {
        let config = config.into_inner();
        let args = WebviewWindowBuilderArgs::from_kwargs(kwargs)?;
        manager_method_impl!(py, &manager, move |py, manager| {
            let mut builder = webview::WebviewWindowBuilder::from_config(manager, &config)
                .map_err(TauriError::from)?;
            if let Some(args) = args {
                builder = args.apply_to_builder(py, builder)?;
            }

            let webview_window = unsafe {
                // Safety: `WebviewWindowBuilder` does not hold the GIL, so this is safe
                py.allow_threads_unsend(builder, |builder| builder.build())
            }
            .map_err(TauriError::from)?;

            PyResult::Ok(WebviewWindow::new(webview_window))
        })?
    }
}
