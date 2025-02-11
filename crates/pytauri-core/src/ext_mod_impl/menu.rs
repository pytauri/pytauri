use std::ops::Deref;

use pyo3::{marker::Ungil, prelude::*, types::PyString};
use pyo3_utils::{
    py_wrapper::{PyWrapper, PyWrapperSemverExt as _, PyWrapperT0},
    ungil::UnsafeUngilExt,
};
use tauri::menu::{self, ContextMenu as _, IsMenuItem, MenuId};

use crate::{
    ext_mod_impl::{self, ImplManager, PyAppHandleExt as _},
    manager_method_impl,
    tauri_runtime::Runtime,
    utils::TauriError,
};

type TauriMenu = menu::Menu<Runtime>;
type TauriMenuItem = menu::MenuItem<Runtime>;
type TauriSubmenu = menu::Submenu<Runtime>;
type TauriPredefinedMenuItem = menu::PredefinedMenuItem<Runtime>;
type TauriCheckMenuItem = menu::CheckMenuItem<Runtime>;
type TauriIconMenuItem = menu::IconMenuItem<Runtime>;
type TauriMenuItemKind = menu::MenuItemKind<Runtime>;

/// see also: [tauri::menu::MenuId]
pub type MenuID = PyString;
/// see also: [tauri::menu::MenuEvent]
pub type MenuEvent = MenuID;
pub use menu::{HELP_SUBMENU_ID, WINDOW_SUBMENU_ID};

/// See also: [tauri::menu::MenuItemKind].
#[derive(FromPyObject, IntoPyObject, IntoPyObjectRef)]
#[non_exhaustive]
pub enum MenuItemKind {
    MenuItem(Py<MenuItem>),
    Submenu(Py<Submenu>),
    Predefined(Py<PredefinedMenuItem>),
    Check(Py<CheckMenuItem>),
    Icon(Py<IconMenuItem>),
}

macro_rules! menu_item_kind_method_impl {
    ($menu_item_kind:expr, $macro:ident) => {
        match $menu_item_kind {
            MenuItemKind::MenuItem(v) => $macro!(v),
            MenuItemKind::Submenu(v) => $macro!(v),
            MenuItemKind::Predefined(v) => $macro!(v),
            MenuItemKind::Check(v) => $macro!(v),
            MenuItemKind::Icon(v) => $macro!(v),
        }
    };
}

trait TauriMenuProto {
    fn append(&self, item: &dyn IsMenuItem<Runtime>) -> tauri::Result<()>;
    fn prepend(&self, item: &dyn IsMenuItem<Runtime>) -> tauri::Result<()>;
    fn insert(&self, item: &dyn IsMenuItem<Runtime>, position: usize) -> tauri::Result<()>;
    fn remove(&self, item: &dyn IsMenuItem<Runtime>) -> tauri::Result<()>;
}

macro_rules! impl_tauri_menu_proto {
    ($trait:ty => $($implementor:ty),* => $append:ident, $prepend:ident, $insert:ident, $remove:ident,) => {

        $(
            impl $trait for $implementor {
                #[inline]
                fn $append(&self, item: &dyn IsMenuItem<Runtime>) -> tauri::Result<()> {
                    self.$append(item)
                }

                #[inline]
                fn $prepend(&self, item: &dyn IsMenuItem<Runtime>) -> tauri::Result<()> {
                    self.$prepend(item)
                }

                #[inline]
                fn $insert(
                    &self,
                    item: &dyn IsMenuItem<Runtime>,
                    position: usize,
                ) -> tauri::Result<()> {
                    self.$insert(item, position)
                }

                #[inline]
                fn $remove(&self, item: &dyn IsMenuItem<Runtime>) -> tauri::Result<()> {
                    self.$remove(item)
                }
            }
        )*

    };
}

impl_tauri_menu_proto!(TauriMenuProto => TauriMenu, TauriSubmenu => append, prepend, insert, remove,);

impl MenuItemKind {
    #[inline]
    fn append_to_menu(&self, menu: &impl TauriMenuProto) -> tauri::Result<()> {
        macro_rules! append_to_menu_impl {
            ($wrapper:expr) => {{
                let menu_item = $wrapper.get().0.inner_ref();
                menu.append(menu_item.deref())
            }};
        }
        menu_item_kind_method_impl!(self, append_to_menu_impl)
    }

    #[inline]
    fn append_items_to_menu<'a>(
        items: impl Iterator<Item = &'a Self>,
        menu: &impl TauriMenuProto,
    ) -> tauri::Result<()> {
        for item_kind in items {
            item_kind.append_to_menu(menu)?;
        }
        Ok(())
    }

    #[inline]
    fn prepend_to_menu(&self, menu: &impl TauriMenuProto) -> tauri::Result<()> {
        macro_rules! prepend_to_menu_impl {
            ($wrapper:expr) => {{
                let menu_item = $wrapper.get().0.inner_ref();
                menu.prepend(menu_item.deref())
            }};
        }
        menu_item_kind_method_impl!(self, prepend_to_menu_impl)
    }

    #[inline]
    fn prepend_items_to_menu<'a>(
        items: impl Iterator<Item = &'a Self>,
        menu: &impl TauriMenuProto,
    ) -> tauri::Result<()> {
        for item_kind in items {
            item_kind.prepend_to_menu(menu)?;
        }
        Ok(())
    }

    #[inline]
    fn insert_to_menu(&self, menu: &impl TauriMenuProto, position: usize) -> tauri::Result<()> {
        macro_rules! insert_to_menu_impl {
            ($wrapper:expr) => {{
                let menu_item = $wrapper.get().0.inner_ref();
                menu.insert(menu_item.deref(), position)
            }};
        }
        menu_item_kind_method_impl!(self, insert_to_menu_impl)
    }

    #[inline]
    fn insert_items_to_menu<'a>(
        items: impl Iterator<Item = &'a Self>,
        menu: &impl TauriMenuProto,
        position: usize,
    ) -> tauri::Result<()> {
        for (idx, item_kind) in items.enumerate() {
            item_kind.insert_to_menu(menu, position + idx)?;
        }
        Ok(())
    }

    #[inline]
    fn remove_from_menu(&self, menu: &impl TauriMenuProto) -> tauri::Result<()> {
        macro_rules! remove_from_menu_impl {
            ($wrapper:expr) => {{
                let menu_item = $wrapper.get().0.inner_ref();
                menu.remove(menu_item.deref())
            }};
        }
        menu_item_kind_method_impl!(self, remove_from_menu_impl)
    }
}

impl MenuItemKind {
    fn from_tauri(py: Python<'_>, menu_kind: TauriMenuItemKind) -> PyResult<Self> {
        let menu_kind = match menu_kind {
            TauriMenuItemKind::Submenu(submenu) => {
                MenuItemKind::Submenu(Submenu::new(submenu).into_pyobject(py)?.unbind())
            }
            TauriMenuItemKind::MenuItem(menu_item) => {
                MenuItemKind::MenuItem(MenuItem::new(menu_item).into_pyobject(py)?.unbind())
            }
            TauriMenuItemKind::Check(check_menu_item) => MenuItemKind::Check(
                CheckMenuItem::new(check_menu_item)
                    .into_pyobject(py)?
                    .unbind(),
            ),
            TauriMenuItemKind::Predefined(predefined_menu_item) => MenuItemKind::Predefined(
                PredefinedMenuItem::new(predefined_menu_item)
                    .into_pyobject(py)?
                    .unbind(),
            ),
            TauriMenuItemKind::Icon(icon_menu_item) => MenuItemKind::Icon(
                IconMenuItem::new(icon_menu_item)
                    .into_pyobject(py)?
                    .unbind(),
            ),
        };
        Ok(menu_kind)
    }
}

/// see also: [tauri::menu::Menu]
#[pyclass(frozen)]
#[non_exhaustive]
pub struct Menu(pub PyWrapper<PyWrapperT0<TauriMenu>>);

impl Menu {
    pub(crate) fn new(menu: TauriMenu) -> Self {
        Self(PyWrapper::new0(menu))
    }

    #[inline]
    fn new_impl(
        py: Python<'_>,
        manager: &impl tauri::Manager<Runtime>,
        id: Option<impl Into<menu::MenuId> + Send>,
        items: Option<Vec<MenuItemKind>>,
    ) -> PyResult<Self> {
        unsafe {
            py.allow_threads_unsend(manager, |manager| {
                let menu = if let Some(id) = id {
                    TauriMenu::with_id(manager, id)
                } else {
                    TauriMenu::new(manager)
                }?;

                if let Some(items) = items {
                    MenuItemKind::append_items_to_menu(items.iter(), &menu)?;
                }
                tauri::Result::Ok(Self::new(menu))
            })
            .map_err(TauriError::from)
            .map_err(PyErr::from)
        }
    }
}

// All methods must release the GIL, because `menu` call `run_on_main_thread` internally, which may block.
#[pymethods]
impl Menu {
    #[new]
    fn __new__(py: Python<'_>, manager: ImplManager) -> PyResult<Self> {
        macro_rules! new_impl {
            ($wrapper:expr) => {{
                let py_ref = $wrapper.borrow(py);
                let guard = py_ref.0.inner_ref_semver()??;
                Self::new_impl(py, guard.deref(), None::<&str>, None)
            }};
        }
        manager_method_impl!(manager, new_impl)
    }

    #[staticmethod]
    fn with_id(py: Python<'_>, manager: ImplManager, id: String) -> PyResult<Self> {
        macro_rules! with_id_impl {
            ($wrapper:expr) => {{
                let py_ref = $wrapper.borrow(py);
                let guard = py_ref.0.inner_ref_semver()??;
                Self::new_impl(py, guard.deref(), Some(MenuId(id)), None)
            }};
        }
        manager_method_impl!(manager, with_id_impl)
    }

    #[staticmethod]
    fn with_items(
        py: Python<'_>,
        manager: ImplManager,
        items: Vec<MenuItemKind>,
    ) -> PyResult<Self> {
        macro_rules! with_items_impl {
            ($wrapper:expr) => {{
                let py_ref = $wrapper.borrow(py);
                let guard = py_ref.0.inner_ref_semver()??;
                Self::new_impl(py, guard.deref(), None::<&str>, Some(items))
            }};
        }
        manager_method_impl!(manager, with_items_impl)
    }

    #[staticmethod]
    fn with_id_and_items(
        py: Python<'_>,
        manager: ImplManager,
        id: String,
        items: Vec<MenuItemKind>,
    ) -> PyResult<Self> {
        macro_rules! with_id_and_items_impl {
            ($wrapper:expr) => {{
                let py_ref = $wrapper.borrow(py);
                let guard = py_ref.0.inner_ref_semver()??;
                Self::new_impl(py, guard.deref(), Some(MenuId(id)), Some(items))
            }};
        }
        manager_method_impl!(manager, with_id_and_items_impl)
    }

    #[staticmethod]
    fn default(py: Python<'_>, app_handle: Py<ext_mod_impl::AppHandle>) -> PyResult<Self> {
        py.allow_threads(|| {
            let app_handle = app_handle.get().0.inner_ref();
            let menu = TauriMenu::default(app_handle.deref()).map_err(TauriError::from)?;
            Ok(Menu::new(menu))
        })
    }

    fn app_handle(&self, py: Python<'_>) -> Py<ext_mod_impl::AppHandle> {
        let menu = self.0.inner_ref();
        let app_handle = menu.app_handle().py_app_handle().clone_ref(py);
        app_handle
    }

    fn id<'py>(&self, py: Python<'py>) -> Bound<'py, PyString> {
        let menu = self.0.inner_ref();
        // TODO, PERF: do we really need `PyString::intern` here?
        PyString::intern(py, &menu.id().0)
    }

    fn append(&self, py: Python<'_>, item: MenuItemKind) -> PyResult<()> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            item.append_to_menu(menu.deref())
                .map_err(TauriError::from)?;
            Ok(())
        })
    }

    fn append_items(&self, py: Python<'_>, items: Vec<MenuItemKind>) -> PyResult<()> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            MenuItemKind::append_items_to_menu(items.iter(), menu.deref())
                .map_err(TauriError::from)?;
            Ok(())
        })
    }

    fn prepend(&self, py: Python<'_>, item: MenuItemKind) -> PyResult<()> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            item.prepend_to_menu(menu.deref())
                .map_err(TauriError::from)?;
            Ok(())
        })
    }

    fn prepend_items(&self, py: Python<'_>, items: Vec<MenuItemKind>) -> PyResult<()> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            MenuItemKind::prepend_items_to_menu(items.iter(), menu.deref())
                .map_err(TauriError::from)?;
            Ok(())
        })
    }

    fn insert(&self, py: Python<'_>, item: MenuItemKind, position: usize) -> PyResult<()> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            item.insert_to_menu(menu.deref(), position)
                .map_err(TauriError::from)?;
            Ok(())
        })
    }

    fn insert_items(
        &self,
        py: Python<'_>,
        items: Vec<MenuItemKind>,
        position: usize,
    ) -> PyResult<()> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            MenuItemKind::insert_items_to_menu(items.iter(), menu.deref(), position)
                .map_err(TauriError::from)?;
            Ok(())
        })
    }

    fn remove(&self, py: Python<'_>, item: MenuItemKind) -> PyResult<()> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            item.remove_from_menu(menu.deref())
                .map_err(TauriError::from)?;
            Ok(())
        })
    }

    fn remove_at(&self, py: Python<'_>, position: usize) -> PyResult<Option<MenuItemKind>> {
        let item_kind = py.allow_threads(|| {
            let menu = self.0.inner_ref();
            menu.remove_at(position)
                .map_err(TauriError::from)
                .map_err(PyErr::from)
        })?;

        let item_kind = match item_kind {
            Some(item_kind) => Some(MenuItemKind::from_tauri(py, item_kind)?),
            None => None,
        };

        Ok(item_kind)
    }

    fn get(&self, py: Python<'_>, id: &str) -> PyResult<Option<MenuItemKind>> {
        let item_kind = py.allow_threads(|| {
            let menu = self.0.inner_ref();
            menu.get(id)
        });

        let item_kind = match item_kind {
            Some(item_kind) => Some(MenuItemKind::from_tauri(py, item_kind)?),
            None => None,
        };

        Ok(item_kind)
    }

    fn items(&self, py: Python<'_>) -> PyResult<Vec<MenuItemKind>> {
        let items = py.allow_threads(|| {
            let menu = self.0.inner_ref();
            menu.items().map_err(TauriError::from)
        })?;

        let mut vec = Vec::with_capacity(items.len());
        for items in items {
            vec.push(MenuItemKind::from_tauri(py, items)?);
        }
        Ok(vec)
    }

    fn set_as_app_menu(&self, py: Python<'_>) -> PyResult<()> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            menu.set_as_app_menu().map_err(TauriError::from)?;
            Ok(())
        })
    }

    fn set_as_window_menu(
        &self,
        py: Python<'_>,
        window: Py<ext_mod_impl::window::Window>,
    ) -> PyResult<()> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            let window = window.get().0.inner_ref();
            menu.set_as_window_menu(window.deref())
                .map_err(TauriError::from)?;
            Ok(())
        })
    }
}

/// see also: [tauri::menu::Submenu]
#[pyclass(frozen)]
#[non_exhaustive]
pub struct Submenu(pub PyWrapper<PyWrapperT0<TauriSubmenu>>);

impl Submenu {
    fn new(menu: TauriSubmenu) -> Self {
        Self(PyWrapper::new0(menu))
    }

    #[inline]
    fn new_impl(
        py: Python<'_>,
        manager: &impl tauri::Manager<Runtime>,
        text: &str,
        enabled: bool,
        id: Option<impl Into<menu::MenuId> + Send>,
        items: Option<Vec<MenuItemKind>>,
    ) -> PyResult<Self> {
        unsafe {
            py.allow_threads_unsend(manager, |manager| {
                let menu = if let Some(id) = id {
                    TauriSubmenu::with_id(manager, id, text, enabled)
                } else {
                    TauriSubmenu::new(manager, text, enabled)
                }?;

                if let Some(items) = items {
                    MenuItemKind::append_items_to_menu(items.iter(), &menu)?;
                }
                tauri::Result::Ok(Self::new(menu))
            })
            .map_err(TauriError::from)
            .map_err(PyErr::from)
        }
    }
}

// All methods must release the GIL, because `menu` call `run_on_main_thread` internally, which may block.
#[pymethods]
impl Submenu {
    #[new]
    fn __new__(py: Python<'_>, manager: ImplManager, text: &str, enabled: bool) -> PyResult<Self> {
        macro_rules! new_impl {
            ($wrapper:expr) => {{
                let py_ref = $wrapper.borrow(py);
                let guard = py_ref.0.inner_ref_semver()??;
                Self::new_impl(py, guard.deref(), text, enabled, None::<&str>, None)
            }};
        }
        manager_method_impl!(manager, new_impl)
    }

    #[staticmethod]
    fn with_id(
        py: Python<'_>,
        manager: ImplManager,
        id: String,
        text: &str,
        enabled: bool,
    ) -> PyResult<Self> {
        macro_rules! with_id_impl {
            ($wrapper:expr) => {{
                let py_ref = $wrapper.borrow(py);
                let guard = py_ref.0.inner_ref_semver()??;
                Self::new_impl(py, guard.deref(), text, enabled, Some(MenuId(id)), None)
            }};
        }
        manager_method_impl!(manager, with_id_impl)
    }

    #[staticmethod]
    fn with_items(
        py: Python<'_>,
        manager: ImplManager,
        text: &str,
        enabled: bool,
        items: Vec<MenuItemKind>,
    ) -> PyResult<Self> {
        macro_rules! with_items_impl {
            ($wrapper:expr) => {{
                let py_ref = $wrapper.borrow(py);
                let guard = py_ref.0.inner_ref_semver()??;
                Self::new_impl(py, guard.deref(), text, enabled, None::<&str>, Some(items))
            }};
        }
        manager_method_impl!(manager, with_items_impl)
    }

    #[staticmethod]
    fn with_id_and_items(
        py: Python<'_>,
        manager: ImplManager,
        id: String,
        text: &str,
        enabled: bool,
        items: Vec<MenuItemKind>,
    ) -> PyResult<Self> {
        macro_rules! with_id_and_items_impl {
            ($wrapper:expr) => {{
                let py_ref = $wrapper.borrow(py);
                let guard = py_ref.0.inner_ref_semver()??;
                Self::new_impl(
                    py,
                    guard.deref(),
                    text,
                    enabled,
                    Some(MenuId(id)),
                    Some(items),
                )
            }};
        }
        manager_method_impl!(manager, with_id_and_items_impl)
    }

    fn app_handle(&self, py: Python<'_>) -> Py<ext_mod_impl::AppHandle> {
        let menu = self.0.inner_ref();
        let app_handle = menu.app_handle().py_app_handle().clone_ref(py);
        app_handle
    }

    fn id<'py>(&self, py: Python<'py>) -> Bound<'py, PyString> {
        let menu = self.0.inner_ref();
        // TODO, PERF: do we really need `PyString::intern` here?
        PyString::intern(py, &menu.id().0)
    }

    fn append(&self, py: Python<'_>, item: MenuItemKind) -> PyResult<()> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            item.append_to_menu(menu.deref())
                .map_err(TauriError::from)?;
            Ok(())
        })
    }

    fn append_items(&self, py: Python<'_>, items: Vec<MenuItemKind>) -> PyResult<()> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            MenuItemKind::append_items_to_menu(items.iter(), menu.deref())
                .map_err(TauriError::from)?;
            Ok(())
        })
    }

    fn prepend(&self, py: Python<'_>, item: MenuItemKind) -> PyResult<()> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            item.prepend_to_menu(menu.deref())
                .map_err(TauriError::from)?;
            Ok(())
        })
    }

    fn prepend_items(&self, py: Python<'_>, items: Vec<MenuItemKind>) -> PyResult<()> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            MenuItemKind::prepend_items_to_menu(items.iter(), menu.deref())
                .map_err(TauriError::from)?;
            Ok(())
        })
    }

    fn insert(&self, py: Python<'_>, item: MenuItemKind, position: usize) -> PyResult<()> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            item.insert_to_menu(menu.deref(), position)
                .map_err(TauriError::from)?;
            Ok(())
        })
    }

    fn insert_items(
        &self,
        py: Python<'_>,
        items: Vec<MenuItemKind>,
        position: usize,
    ) -> PyResult<()> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            MenuItemKind::insert_items_to_menu(items.iter(), menu.deref(), position)
                .map_err(TauriError::from)?;
            Ok(())
        })
    }

    fn remove(&self, py: Python<'_>, item: MenuItemKind) -> PyResult<()> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            item.remove_from_menu(menu.deref())
                .map_err(TauriError::from)?;
            Ok(())
        })
    }

    fn remove_at(&self, py: Python<'_>, position: usize) -> PyResult<Option<MenuItemKind>> {
        let item_kind = py.allow_threads(|| {
            let menu = self.0.inner_ref();
            menu.remove_at(position)
                .map_err(TauriError::from)
                .map_err(PyErr::from)
        })?;

        let item_kind = match item_kind {
            Some(item_kind) => Some(MenuItemKind::from_tauri(py, item_kind)?),
            None => None,
        };

        Ok(item_kind)
    }

    fn get(&self, py: Python<'_>, id: &str) -> PyResult<Option<MenuItemKind>> {
        let item_kind = py.allow_threads(|| {
            let menu = self.0.inner_ref();
            menu.get(id)
        });

        let item_kind = match item_kind {
            Some(item_kind) => Some(MenuItemKind::from_tauri(py, item_kind)?),
            None => None,
        };

        Ok(item_kind)
    }

    fn items(&self, py: Python<'_>) -> PyResult<Vec<MenuItemKind>> {
        let items = py.allow_threads(|| {
            let menu = self.0.inner_ref();
            menu.items().map_err(TauriError::from)
        })?;

        let mut vec = Vec::with_capacity(items.len());
        for items in items {
            vec.push(MenuItemKind::from_tauri(py, items)?);
        }
        Ok(vec)
    }

    fn text(&self, py: Python<'_>) -> PyResult<String> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            let text = menu.text().map_err(TauriError::from)?;
            Ok(text)
        })
    }

    fn set_text(&self, py: Python<'_>, text: &str) -> PyResult<()> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            menu.set_text(text).map_err(TauriError::from)?;
            Ok(())
        })
    }

    fn is_enabled(&self, py: Python<'_>) -> PyResult<bool> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            let enabled = menu.is_enabled().map_err(TauriError::from)?;
            Ok(enabled)
        })
    }

    fn set_enabled(&self, py: Python<'_>, enabled: bool) -> PyResult<()> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            menu.set_enabled(enabled).map_err(TauriError::from)?;
            Ok(())
        })
    }
}

/// see also: [tauri::menu::MenuItem]
#[pyclass(frozen)]
#[non_exhaustive]
pub struct MenuItem(pub PyWrapper<PyWrapperT0<TauriMenuItem>>);

impl MenuItem {
    fn new(menu: TauriMenuItem) -> Self {
        Self(PyWrapper::new0(menu))
    }

    #[inline]
    fn new_impl(
        py: Python<'_>,
        manager: &impl tauri::Manager<Runtime>,
        text: &str,
        enabled: bool,
        accelerator: Option<&str>,
        id: Option<impl Into<menu::MenuId> + Send>,
    ) -> PyResult<Self> {
        unsafe {
            py.allow_threads_unsend(manager, |manager| {
                let menu = if let Some(id) = id {
                    TauriMenuItem::with_id(manager, id, text, enabled, accelerator)
                } else {
                    TauriMenuItem::new(manager, text, enabled, accelerator)
                }?;

                tauri::Result::Ok(Self::new(menu))
            })
            .map_err(TauriError::from)
            .map_err(PyErr::from)
        }
    }
}

#[pymethods]
impl MenuItem {
    #[new]
    #[pyo3(signature = (manager, text, enabled, accelerator=None))]
    fn __new__(
        py: Python<'_>,
        manager: ImplManager,
        text: &str,
        enabled: bool,
        accelerator: Option<&str>,
    ) -> PyResult<Self> {
        macro_rules! new_impl {
            ($wrapper:expr) => {{
                let py_ref = $wrapper.borrow(py);
                let guard = py_ref.0.inner_ref_semver()??;
                Self::new_impl(py, guard.deref(), text, enabled, accelerator, None::<&str>)
            }};
        }
        manager_method_impl!(manager, new_impl)
    }

    #[staticmethod]
    #[pyo3(signature = (manager, id, text, enabled, accelerator=None))]
    fn with_id(
        py: Python<'_>,
        manager: ImplManager,
        id: String,
        text: &str,
        enabled: bool,
        accelerator: Option<&str>,
    ) -> PyResult<Self> {
        macro_rules! with_id_impl {
            ($wrapper:expr) => {{
                let py_ref = $wrapper.borrow(py);
                let guard = py_ref.0.inner_ref_semver()??;
                Self::new_impl(
                    py,
                    guard.deref(),
                    text,
                    enabled,
                    accelerator,
                    Some(MenuId(id)),
                )
            }};
        }
        manager_method_impl!(manager, with_id_impl)
    }

    fn app_handle(&self, py: Python<'_>) -> Py<ext_mod_impl::AppHandle> {
        let menu = self.0.inner_ref();
        let app_handle = menu.app_handle().py_app_handle().clone_ref(py);
        app_handle
    }

    fn id<'py>(&self, py: Python<'py>) -> Bound<'py, PyString> {
        let menu = self.0.inner_ref();
        // TODO, PERF: do we really need `PyString::intern` here?
        PyString::intern(py, &menu.id().0)
    }

    fn text(&self, py: Python<'_>) -> PyResult<String> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            let text = menu.text().map_err(TauriError::from)?;
            Ok(text)
        })
    }

    fn set_text(&self, py: Python<'_>, text: &str) -> PyResult<()> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            menu.set_text(text).map_err(TauriError::from)?;
            Ok(())
        })
    }

    fn is_enabled(&self, py: Python<'_>) -> PyResult<bool> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            let enabled = menu.is_enabled().map_err(TauriError::from)?;
            Ok(enabled)
        })
    }

    fn set_enabled(&self, py: Python<'_>, enabled: bool) -> PyResult<()> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            menu.set_enabled(enabled).map_err(TauriError::from)?;
            Ok(())
        })
    }

    #[pyo3(signature = (accelerator))]
    fn set_accelerator(&self, py: Python<'_>, accelerator: Option<&str>) -> PyResult<()> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            menu.set_accelerator(accelerator)
                .map_err(TauriError::from)?;
            Ok(())
        })
    }
}

/// see also: [tauri::menu::PredefinedMenuItem]
#[pyclass(frozen)]
#[non_exhaustive]
pub struct PredefinedMenuItem(pub PyWrapper<PyWrapperT0<TauriPredefinedMenuItem>>);

impl PredefinedMenuItem {
    fn new(menu: TauriPredefinedMenuItem) -> Self {
        Self(PyWrapper::new0(menu))
    }

    #[inline]
    fn delegate_inner<M, F>(py: Python<'_>, manager: &M, func: F) -> PyResult<Self>
    where
        M: tauri::Manager<Runtime>,
        F: FnOnce(&M) -> tauri::Result<TauriPredefinedMenuItem> + Ungil + Send,
    {
        unsafe {
            py.allow_threads_unsend(manager, |manager| {
                let menu = func(manager)?;
                tauri::Result::Ok(Self::new(menu))
            })
        }
        .map_err(TauriError::from)
        .map_err(PyErr::from)
    }
}

#[pymethods]
impl PredefinedMenuItem {
    #[staticmethod]
    fn separator(py: Python<'_>, manager: ImplManager) -> PyResult<Self> {
        macro_rules! separator_impl {
            ($wrapper:expr) => {{
                let py_ref = $wrapper.borrow(py);
                let guard = py_ref.0.inner_ref_semver()??;
                Self::delegate_inner(py, guard.deref(), move |manager| {
                    TauriPredefinedMenuItem::separator(manager)
                })
            }};
        }
        manager_method_impl!(manager, separator_impl)
    }

    #[staticmethod]
    #[pyo3(signature = (manager, text=None))]
    fn copy(py: Python<'_>, manager: ImplManager, text: Option<&str>) -> PyResult<Self> {
        macro_rules! copy_impl {
            ($wrapper:expr) => {{
                let py_ref = $wrapper.borrow(py);
                let guard = py_ref.0.inner_ref_semver()??;
                Self::delegate_inner(py, guard.deref(), move |manager| {
                    TauriPredefinedMenuItem::copy(manager, text)
                })
            }};
        }
        manager_method_impl!(manager, copy_impl)
    }

    #[staticmethod]
    #[pyo3(signature = (manager, text=None))]
    fn cut(py: Python<'_>, manager: ImplManager, text: Option<&str>) -> PyResult<Self> {
        macro_rules! cut_impl {
            ($wrapper:expr) => {{
                let py_ref = $wrapper.borrow(py);
                let guard = py_ref.0.inner_ref_semver()??;
                Self::delegate_inner(py, guard.deref(), move |manager| {
                    TauriPredefinedMenuItem::cut(manager, text)
                })
            }};
        }
        manager_method_impl!(manager, cut_impl)
    }

    #[staticmethod]
    #[pyo3(signature = (manager, text=None))]
    fn paste(py: Python<'_>, manager: ImplManager, text: Option<&str>) -> PyResult<Self> {
        macro_rules! paste_impl {
            ($wrapper:expr) => {{
                let py_ref = $wrapper.borrow(py);
                let guard = py_ref.0.inner_ref_semver()??;
                Self::delegate_inner(py, guard.deref(), move |manager| {
                    TauriPredefinedMenuItem::paste(manager, text)
                })
            }};
        }
        manager_method_impl!(manager, paste_impl)
    }

    #[staticmethod]
    #[pyo3(signature = (manager, text=None))]
    fn select_all(py: Python<'_>, manager: ImplManager, text: Option<&str>) -> PyResult<Self> {
        macro_rules! select_all_impl {
            ($wrapper:expr) => {{
                let py_ref = $wrapper.borrow(py);
                let guard = py_ref.0.inner_ref_semver()??;
                Self::delegate_inner(py, guard.deref(), move |manager| {
                    TauriPredefinedMenuItem::select_all(manager, text)
                })
            }};
        }
        manager_method_impl!(manager, select_all_impl)
    }

    #[staticmethod]
    #[pyo3(signature = (manager, text=None))]
    fn undo(py: Python<'_>, manager: ImplManager, text: Option<&str>) -> PyResult<Self> {
        macro_rules! undo_impl {
            ($wrapper:expr) => {{
                let py_ref = $wrapper.borrow(py);
                let guard = py_ref.0.inner_ref_semver()??;
                Self::delegate_inner(py, guard.deref(), move |manager| {
                    TauriPredefinedMenuItem::undo(manager, text)
                })
            }};
        }
        manager_method_impl!(manager, undo_impl)
    }

    #[staticmethod]
    #[pyo3(signature = (manager, text=None))]
    fn redo(py: Python<'_>, manager: ImplManager, text: Option<&str>) -> PyResult<Self> {
        macro_rules! redo_impl {
            ($wrapper:expr) => {{
                let py_ref = $wrapper.borrow(py);
                let guard = py_ref.0.inner_ref_semver()??;
                Self::delegate_inner(py, guard.deref(), move |manager| {
                    TauriPredefinedMenuItem::redo(manager, text)
                })
            }};
        }
        manager_method_impl!(manager, redo_impl)
    }

    #[staticmethod]
    #[pyo3(signature = (manager, text=None))]
    fn minimize(py: Python<'_>, manager: ImplManager, text: Option<&str>) -> PyResult<Self> {
        macro_rules! minimize_impl {
            ($wrapper:expr) => {{
                let py_ref = $wrapper.borrow(py);
                let guard = py_ref.0.inner_ref_semver()??;
                Self::delegate_inner(py, guard.deref(), move |manager| {
                    TauriPredefinedMenuItem::minimize(manager, text)
                })
            }};
        }
        manager_method_impl!(manager, minimize_impl)
    }

    #[staticmethod]
    #[pyo3(signature = (manager, text=None))]
    fn maximize(py: Python<'_>, manager: ImplManager, text: Option<&str>) -> PyResult<Self> {
        macro_rules! maximize_impl {
            ($wrapper:expr) => {{
                let py_ref = $wrapper.borrow(py);
                let guard = py_ref.0.inner_ref_semver()??;
                Self::delegate_inner(py, guard.deref(), move |manager| {
                    TauriPredefinedMenuItem::maximize(manager, text)
                })
            }};
        }
        manager_method_impl!(manager, maximize_impl)
    }

    #[staticmethod]
    #[pyo3(signature = (manager, text=None))]
    fn fullscreen(py: Python<'_>, manager: ImplManager, text: Option<&str>) -> PyResult<Self> {
        macro_rules! fullscreen_impl {
            ($wrapper:expr) => {{
                let py_ref = $wrapper.borrow(py);
                let guard = py_ref.0.inner_ref_semver()??;
                Self::delegate_inner(py, guard.deref(), move |manager| {
                    TauriPredefinedMenuItem::fullscreen(manager, text)
                })
            }};
        }
        manager_method_impl!(manager, fullscreen_impl)
    }

    #[staticmethod]
    #[pyo3(signature = (manager, text=None))]
    fn hide(py: Python<'_>, manager: ImplManager, text: Option<&str>) -> PyResult<Self> {
        macro_rules! hide_impl {
            ($wrapper:expr) => {{
                let py_ref = $wrapper.borrow(py);
                let guard = py_ref.0.inner_ref_semver()??;
                Self::delegate_inner(py, guard.deref(), move |manager| {
                    TauriPredefinedMenuItem::hide(manager, text)
                })
            }};
        }
        manager_method_impl!(manager, hide_impl)
    }

    #[staticmethod]
    #[pyo3(signature = (manager, text=None))]
    fn hide_others(py: Python<'_>, manager: ImplManager, text: Option<&str>) -> PyResult<Self> {
        macro_rules! hide_others_impl {
            ($wrapper:expr) => {{
                let py_ref = $wrapper.borrow(py);
                let guard = py_ref.0.inner_ref_semver()??;
                Self::delegate_inner(py, guard.deref(), move |manager| {
                    TauriPredefinedMenuItem::hide_others(manager, text)
                })
            }};
        }
        manager_method_impl!(manager, hide_others_impl)
    }

    #[staticmethod]
    #[pyo3(signature = (manager, text=None))]
    fn show_all(py: Python<'_>, manager: ImplManager, text: Option<&str>) -> PyResult<Self> {
        macro_rules! show_all_impl {
            ($wrapper:expr) => {{
                let py_ref = $wrapper.borrow(py);
                let guard = py_ref.0.inner_ref_semver()??;
                Self::delegate_inner(py, guard.deref(), move |manager| {
                    TauriPredefinedMenuItem::show_all(manager, text)
                })
            }};
        }
        manager_method_impl!(manager, show_all_impl)
    }

    #[staticmethod]
    #[pyo3(signature = (manager, text=None))]
    fn close_window(py: Python<'_>, manager: ImplManager, text: Option<&str>) -> PyResult<Self> {
        macro_rules! close_window_impl {
            ($wrapper:expr) => {{
                let py_ref = $wrapper.borrow(py);
                let guard = py_ref.0.inner_ref_semver()??;
                Self::delegate_inner(py, guard.deref(), move |manager| {
                    TauriPredefinedMenuItem::close_window(manager, text)
                })
            }};
        }
        manager_method_impl!(manager, close_window_impl)
    }

    #[staticmethod]
    #[pyo3(signature = (manager, text=None))]
    fn quit(py: Python<'_>, manager: ImplManager, text: Option<&str>) -> PyResult<Self> {
        macro_rules! quit_impl {
            ($wrapper:expr) => {{
                let py_ref = $wrapper.borrow(py);
                let guard = py_ref.0.inner_ref_semver()??;
                Self::delegate_inner(py, guard.deref(), move |manager| {
                    TauriPredefinedMenuItem::quit(manager, text)
                })
            }};
        }
        manager_method_impl!(manager, quit_impl)
    }

    #[staticmethod]
    #[pyo3(signature = (manager, text=None, metadata=None))]
    fn about(
        py: Python<'_>,
        manager: ImplManager,
        text: Option<&str>,
        metadata: Option<Py<AboutMetadata>>,
    ) -> PyResult<Self> {
        let metadata = match metadata {
            Some(ref metadata) => Some(metadata.get().to_tauri(py)?),
            None => None,
        };
        macro_rules! about_impl {
            ($wrapper:expr) => {{
                let py_ref = $wrapper.borrow(py);
                let guard = py_ref.0.inner_ref_semver()??;
                Self::delegate_inner(py, guard.deref(), move |manager| {
                    TauriPredefinedMenuItem::about(manager, text, metadata)
                })
            }};
        }
        manager_method_impl!(manager, about_impl)
    }

    #[staticmethod]
    #[pyo3(signature = (manager, text=None))]
    fn services(py: Python<'_>, manager: ImplManager, text: Option<&str>) -> PyResult<Self> {
        macro_rules! services_impl {
            ($wrapper:expr) => {{
                let py_ref = $wrapper.borrow(py);
                let guard = py_ref.0.inner_ref_semver()??;
                Self::delegate_inner(py, guard.deref(), move |manager| {
                    TauriPredefinedMenuItem::services(manager, text)
                })
            }};
        }
        manager_method_impl!(manager, services_impl)
    }

    fn app_handle(&self, py: Python<'_>) -> Py<ext_mod_impl::AppHandle> {
        let menu = self.0.inner_ref();
        let app_handle = menu.app_handle().py_app_handle().clone_ref(py);
        app_handle
    }

    fn id<'py>(&self, py: Python<'py>) -> Bound<'py, PyString> {
        let menu = self.0.inner_ref();
        // TODO, PERF: do we really need `PyString::intern` here?
        PyString::intern(py, &menu.id().0)
    }

    fn text(&self, py: Python<'_>) -> PyResult<String> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            let text = menu.text().map_err(TauriError::from)?;
            Ok(text)
        })
    }

    fn set_text(&self, py: Python<'_>, text: &str) -> PyResult<()> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            menu.set_text(text).map_err(TauriError::from)?;
            Ok(())
        })
    }
}

/// see also: [tauri::menu::CheckMenuItem]
#[pyclass(frozen)]
#[non_exhaustive]
pub struct CheckMenuItem(pub PyWrapper<PyWrapperT0<TauriCheckMenuItem>>);

impl CheckMenuItem {
    fn new(menu: TauriCheckMenuItem) -> Self {
        Self(PyWrapper::new0(menu))
    }

    #[inline]
    fn new_impl(
        py: Python<'_>,
        manager: &impl tauri::Manager<Runtime>,
        text: &str,
        enabled: bool,
        checked: bool,
        accelerator: Option<&str>,
        id: Option<impl Into<menu::MenuId> + Send>,
    ) -> PyResult<Self> {
        unsafe {
            py.allow_threads_unsend(manager, |manager| {
                let menu = if let Some(id) = id {
                    TauriCheckMenuItem::with_id(manager, id, text, enabled, checked, accelerator)
                } else {
                    TauriCheckMenuItem::new(manager, text, enabled, checked, accelerator)
                }?;

                tauri::Result::Ok(Self::new(menu))
            })
            .map_err(TauriError::from)
            .map_err(PyErr::from)
        }
    }
}

#[pymethods]
impl CheckMenuItem {
    #[new]
    #[pyo3(signature = (manager, text, enabled, checked, accelerator=None))]
    fn __new__(
        py: Python<'_>,
        manager: ImplManager,
        text: &str,
        enabled: bool,
        checked: bool,
        accelerator: Option<&str>,
    ) -> PyResult<Self> {
        macro_rules! new_impl {
            ($wrapper:expr) => {{
                let py_ref = $wrapper.borrow(py);
                let guard = py_ref.0.inner_ref_semver()??;
                Self::new_impl(
                    py,
                    guard.deref(),
                    text,
                    enabled,
                    checked,
                    accelerator,
                    None::<&str>,
                )
            }};
        }
        manager_method_impl!(manager, new_impl)
    }

    #[staticmethod]
    #[pyo3(signature = (manager, id, text, enabled, checked, accelerator=None))]
    fn with_id(
        py: Python<'_>,
        manager: ImplManager,
        id: String,
        text: &str,
        enabled: bool,
        checked: bool,
        accelerator: Option<&str>,
    ) -> PyResult<Self> {
        macro_rules! with_id_impl {
            ($wrapper:expr) => {{
                let py_ref = $wrapper.borrow(py);
                let guard = py_ref.0.inner_ref_semver()??;
                Self::new_impl(
                    py,
                    guard.deref(),
                    text,
                    enabled,
                    checked,
                    accelerator,
                    Some(MenuId(id)),
                )
            }};
        }
        manager_method_impl!(manager, with_id_impl)
    }

    fn app_handle(&self, py: Python<'_>) -> Py<ext_mod_impl::AppHandle> {
        let menu = self.0.inner_ref();
        let app_handle = menu.app_handle().py_app_handle().clone_ref(py);
        app_handle
    }

    fn id<'py>(&self, py: Python<'py>) -> Bound<'py, PyString> {
        let menu = self.0.inner_ref();
        // TODO, PERF: do we really need `PyString::intern` here?
        PyString::intern(py, &menu.id().0)
    }

    fn text(&self, py: Python<'_>) -> PyResult<String> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            let text = menu.text().map_err(TauriError::from)?;
            Ok(text)
        })
    }

    fn set_text(&self, py: Python<'_>, text: &str) -> PyResult<()> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            menu.set_text(text).map_err(TauriError::from)?;
            Ok(())
        })
    }

    fn is_enabled(&self, py: Python<'_>) -> PyResult<bool> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            let enabled = menu.is_enabled().map_err(TauriError::from)?;
            Ok(enabled)
        })
    }

    fn set_enabled(&self, py: Python<'_>, enabled: bool) -> PyResult<()> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            menu.set_enabled(enabled).map_err(TauriError::from)?;
            Ok(())
        })
    }

    #[pyo3(signature = (accelerator))]
    fn set_accelerator(&self, py: Python<'_>, accelerator: Option<&str>) -> PyResult<()> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            menu.set_accelerator(accelerator)
                .map_err(TauriError::from)?;
            Ok(())
        })
    }

    fn is_checked(&self, py: Python<'_>) -> PyResult<bool> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            let checked = menu.is_checked().map_err(TauriError::from)?;
            Ok(checked)
        })
    }

    fn set_checked(&self, py: Python<'_>, checked: bool) -> PyResult<()> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            menu.set_checked(checked).map_err(TauriError::from)?;
            Ok(())
        })
    }
}

trait PyStrToRs {
    type Output;
    fn to_rs(&self, py: Python<'_>) -> Self::Output;
}

impl PyStrToRs for Py<PyString> {
    type Output = PyResult<String>;
    fn to_rs(&self, py: Python<'_>) -> Self::Output {
        // PERF: once we drop py39 support, we can use [PyStringMethods::to_str] directly.
        Ok(self.to_cow(py)?.into_owned())
    }
}

impl PyStrToRs for Option<Py<PyString>> {
    type Output = PyResult<Option<String>>;
    fn to_rs(&self, py: Python<'_>) -> Self::Output {
        match self {
            Some(py_str) => Ok(Some(py_str.to_rs(py)?)),
            None => Ok(None),
        }
    }
}

impl PyStrToRs for Option<Vec<Py<PyString>>> {
    type Output = PyResult<Option<Vec<String>>>;
    fn to_rs(&self, py: Python<'_>) -> Self::Output {
        match self {
            Some(py_str_vec) => {
                let mut vec = Vec::with_capacity(py_str_vec.len());
                for py_str in py_str_vec {
                    vec.push(py_str.to_rs(py)?);
                }
                Ok(Some(vec))
            }
            None => Ok(None),
        }
    }
}

/// See also [tauri::menu::AboutMetadata].
#[pyclass(frozen)]
#[non_exhaustive]
pub struct AboutMetadata {
    name: Option<Py<PyString>>,
    version: Option<Py<PyString>>,
    short_version: Option<Py<PyString>>,
    authors: Option<Vec<Py<PyString>>>,
    comments: Option<Py<PyString>>,
    copyright: Option<Py<PyString>>,
    license: Option<Py<PyString>>,
    website: Option<Py<PyString>>,
    website_label: Option<Py<PyString>>,
    credits: Option<Py<PyString>>,
    icon: Option<Py<ext_mod_impl::image::Image>>,
}

impl AboutMetadata {
    fn to_tauri<'a>(&'a self, py: Python<'_>) -> PyResult<menu::AboutMetadata<'a>> {
        let about_metadata = menu::AboutMetadata {
            name: self.name.to_rs(py)?,
            version: self.version.to_rs(py)?,
            short_version: self.short_version.to_rs(py)?,
            authors: self.authors.to_rs(py)?,
            comments: self.comments.to_rs(py)?,
            copyright: self.copyright.to_rs(py)?,
            license: self.license.to_rs(py)?,
            website: self.website.to_rs(py)?,
            website_label: self.website_label.to_rs(py)?,
            credits: self.credits.to_rs(py)?,
            icon: self.icon.as_ref().map(|icon| icon.get().to_tauri(py)),
        };
        Ok(about_metadata)
    }
}

#[pymethods]
impl AboutMetadata {
    #[new]
    #[pyo3(signature = (
        *,
        name=None,
        version=None,
        short_version=None,
        authors=None,
        comments=None,
        copyright=None,
        license=None,
        website=None,
        website_label=None,
        credits=None,
        icon=None
    ))]
    #[expect(clippy::too_many_arguments)]
    const fn __new__(
        name: Option<Py<PyString>>,
        version: Option<Py<PyString>>,
        short_version: Option<Py<PyString>>,
        authors: Option<Vec<Py<PyString>>>,
        comments: Option<Py<PyString>>,
        copyright: Option<Py<PyString>>,
        license: Option<Py<PyString>>,
        website: Option<Py<PyString>>,
        website_label: Option<Py<PyString>>,
        credits: Option<Py<PyString>>,
        icon: Option<Py<ext_mod_impl::image::Image>>,
    ) -> Self {
        Self {
            name,
            version,
            short_version,
            authors,
            comments,
            copyright,
            license,
            website,
            website_label,
            credits,
            icon,
        }
    }
}

enum IconOrNative<'a> {
    Icon(Option<tauri::image::Image<'a>>),
    Native(Option<menu::NativeIcon>),
}

/// see also: [tauri::menu::IconMenuItem]
#[pyclass(frozen)]
#[non_exhaustive]
pub struct IconMenuItem(pub PyWrapper<PyWrapperT0<TauriIconMenuItem>>);

impl IconMenuItem {
    fn new(menu: TauriIconMenuItem) -> Self {
        Self(PyWrapper::new0(menu))
    }

    #[inline]
    fn new_impl(
        py: Python<'_>,
        manager: &impl tauri::Manager<Runtime>,
        text: &str,
        enabled: bool,
        icon_or_native: IconOrNative<'_>,
        accelerator: Option<&str>,
        id: Option<impl Into<menu::MenuId> + Send>,
    ) -> PyResult<Self> {
        unsafe {
            py.allow_threads_unsend(manager, |manager| {
                let menu = if let Some(id) = id {
                    match icon_or_native {
                        IconOrNative::Icon(icon) => TauriIconMenuItem::with_id(
                            manager,
                            id,
                            text,
                            enabled,
                            icon,
                            accelerator,
                        ),
                        IconOrNative::Native(native_icon) => {
                            TauriIconMenuItem::with_id_and_native_icon(
                                manager,
                                id,
                                text,
                                enabled,
                                native_icon,
                                accelerator,
                            )
                        }
                    }
                } else {
                    match icon_or_native {
                        IconOrNative::Icon(icon) => {
                            TauriIconMenuItem::new(manager, text, enabled, icon, accelerator)
                        }
                        IconOrNative::Native(native_icon) => TauriIconMenuItem::with_native_icon(
                            manager,
                            text,
                            enabled,
                            native_icon,
                            accelerator,
                        ),
                    }
                }?;

                tauri::Result::Ok(Self::new(menu))
            })
            .map_err(TauriError::from)
            .map_err(PyErr::from)
        }
    }
}

#[pymethods]
impl IconMenuItem {
    #[new]
    #[pyo3(signature = (manager, text, enabled, icon=None, accelerator=None))]
    fn __new__(
        py: Python<'_>,
        manager: ImplManager,
        text: &str,
        enabled: bool,
        icon: Option<Py<ext_mod_impl::image::Image>>,
        accelerator: Option<&str>,
    ) -> PyResult<Self> {
        let icon = icon.as_ref().map(|icon| icon.get().to_tauri(py));
        let icon = IconOrNative::Icon(icon);
        macro_rules! new_impl {
            ($wrapper:expr) => {{
                let py_ref = $wrapper.borrow(py);
                let guard = py_ref.0.inner_ref_semver()??;
                Self::new_impl(
                    py,
                    guard.deref(),
                    text,
                    enabled,
                    icon,
                    accelerator,
                    None::<&str>,
                )
            }};
        }
        manager_method_impl!(manager, new_impl)
    }

    #[staticmethod]
    #[pyo3(signature = (manager, id, text, enabled, icon=None, accelerator=None))]
    fn with_id(
        py: Python<'_>,
        manager: ImplManager,
        id: String,
        text: &str,
        enabled: bool,
        icon: Option<Py<ext_mod_impl::image::Image>>,
        accelerator: Option<&str>,
    ) -> PyResult<Self> {
        let icon = icon.as_ref().map(|icon| icon.get().to_tauri(py));
        let icon = IconOrNative::Icon(icon);
        macro_rules! with_id_impl {
            ($wrapper:expr) => {{
                let py_ref = $wrapper.borrow(py);
                let guard = py_ref.0.inner_ref_semver()??;
                Self::new_impl(
                    py,
                    guard.deref(),
                    text,
                    enabled,
                    icon,
                    accelerator,
                    Some(MenuId(id)),
                )
            }};
        }
        manager_method_impl!(manager, with_id_impl)
    }

    #[staticmethod]
    #[pyo3(signature = (manager, text, enabled, native_icon=None, accelerator=None))]
    fn with_native_icon(
        py: Python<'_>,
        manager: ImplManager,
        text: &str,
        enabled: bool,
        native_icon: Option<NativeIcon>,
        accelerator: Option<&str>,
    ) -> PyResult<Self> {
        let native_icon = native_icon.map(|native_icon| native_icon.into_tauri());
        let native_icon = IconOrNative::Native(native_icon);
        macro_rules! with_native_icon_impl {
            ($wrapper:expr) => {{
                let py_ref = $wrapper.borrow(py);
                let guard = py_ref.0.inner_ref_semver()??;
                Self::new_impl(
                    py,
                    guard.deref(),
                    text,
                    enabled,
                    native_icon,
                    accelerator,
                    None::<&str>,
                )
            }};
        }
        manager_method_impl!(manager, with_native_icon_impl)
    }

    #[staticmethod]
    #[pyo3(signature = (manager, id, text, enabled, native_icon=None, accelerator=None))]
    fn with_id_and_native_icon(
        py: Python<'_>,
        manager: ImplManager,
        id: String,
        text: &str,
        enabled: bool,
        native_icon: Option<NativeIcon>,
        accelerator: Option<&str>,
    ) -> PyResult<Self> {
        let native_icon = native_icon.map(|native_icon| native_icon.into_tauri());
        let native_icon = IconOrNative::Native(native_icon);
        macro_rules! with_id_and_native_icon_impl {
            ($wrapper:expr) => {{
                let py_ref = $wrapper.borrow(py);
                let guard = py_ref.0.inner_ref_semver()??;
                Self::new_impl(
                    py,
                    guard.deref(),
                    text,
                    enabled,
                    native_icon,
                    accelerator,
                    Some(MenuId(id)),
                )
            }};
        }
        manager_method_impl!(manager, with_id_and_native_icon_impl)
    }

    fn app_handle(&self, py: Python<'_>) -> Py<ext_mod_impl::AppHandle> {
        let menu = self.0.inner_ref();
        let app_handle = menu.app_handle().py_app_handle().clone_ref(py);
        app_handle
    }

    fn id<'py>(&self, py: Python<'py>) -> Bound<'py, PyString> {
        let menu = self.0.inner_ref();
        // TODO, PERF: do we really need `PyString::intern` here?
        PyString::intern(py, &menu.id().0)
    }

    fn text(&self, py: Python<'_>) -> PyResult<String> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            let text = menu.text().map_err(TauriError::from)?;
            Ok(text)
        })
    }

    fn set_text(&self, py: Python<'_>, text: &str) -> PyResult<()> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            menu.set_text(text).map_err(TauriError::from)?;
            Ok(())
        })
    }

    fn is_enabled(&self, py: Python<'_>) -> PyResult<bool> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            let enabled = menu.is_enabled().map_err(TauriError::from)?;
            Ok(enabled)
        })
    }

    fn set_enabled(&self, py: Python<'_>, enabled: bool) -> PyResult<()> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            menu.set_enabled(enabled).map_err(TauriError::from)?;
            Ok(())
        })
    }

    #[pyo3(signature = (accelerator))]
    fn set_accelerator(&self, py: Python<'_>, accelerator: Option<&str>) -> PyResult<()> {
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            menu.set_accelerator(accelerator)
                .map_err(TauriError::from)?;
            Ok(())
        })
    }

    #[pyo3(signature = (icon))]
    fn set_icon(
        &self,
        py: Python<'_>,
        icon: Option<Py<ext_mod_impl::image::Image>>,
    ) -> PyResult<()> {
        let icon = icon.as_ref().map(|icon| icon.get().to_tauri(py));
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            menu.set_icon(icon).map_err(TauriError::from)?;
            Ok(())
        })
    }

    #[pyo3(signature = (native_icon))]
    fn set_native_icon(&self, py: Python<'_>, native_icon: Option<NativeIcon>) -> PyResult<()> {
        let native_icon = native_icon.map(|native_icon| native_icon.into_tauri());
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            menu.set_native_icon(native_icon)
                .map_err(TauriError::from)?;
            Ok(())
        })
    }
}

macro_rules! native_icon_impl {
    ($ident:ident => $into_tauri:ident : $($variant:ident),*) => {
        /// see also: [tauri::menu::NativeIcon]
        #[pyclass(frozen, eq, eq_int)]
        #[derive(PartialEq, Clone, Copy)]
        pub enum $ident {
            $($variant,)*
        }

        impl $ident {
            pub(crate) fn $into_tauri(self) -> tauri::menu::NativeIcon {
                match self {
                    $($ident::$variant => tauri::menu::NativeIcon::$variant,)*
                }
            }
        }
    };
}

native_icon_impl!(
    NativeIcon => into_tauri:
    Add,
    Advanced,
    Bluetooth,
    Bookmarks,
    Caution,
    ColorPanel,
    ColumnView,
    Computer,
    EnterFullScreen,
    Everyone,
    ExitFullScreen,
    FlowView,
    Folder,
    FolderBurnable,
    FolderSmart,
    FollowLinkFreestanding,
    FontPanel,
    GoLeft,
    GoRight,
    Home,
    IChatTheater,
    IconView,
    Info,
    InvalidDataFreestanding,
    LeftFacingTriangle,
    ListView,
    LockLocked,
    LockUnlocked,
    MenuMixedState,
    MenuOnState,
    MobileMe,
    MultipleDocuments,
    Network,
    Path,
    PreferencesGeneral,
    QuickLook,
    RefreshFreestanding,
    Refresh,
    Remove,
    RevealFreestanding,
    RightFacingTriangle,
    Share,
    Slideshow,
    SmartBadge,
    StatusAvailable,
    StatusNone,
    StatusPartiallyAvailable,
    StatusUnavailable,
    StopProgressFreestanding,
    StopProgress,
    TrashEmpty,
    TrashFull,
    User,
    UserAccounts,
    UserGroup,
    UserGuest
);

/// The Implementors of [tauri::menu::ContextMenu].
#[derive(FromPyObject, IntoPyObject, IntoPyObjectRef)]
#[non_exhaustive]
pub enum ImplContextMenu {
    Menu(Py<Menu>),
    Submenu(Py<Submenu>),
}

/// See also: [tauri::menu::ContextMenu].
#[pyclass(frozen)]
#[non_exhaustive]
pub struct ContextMenu;

#[doc(hidden)]
#[macro_export]
macro_rules! context_menu_impl {
    ($slf:expr, $macro:ident) => {
        match $slf {
            ImplContextMenu::Menu(v) => $macro!(v),
            ImplContextMenu::Submenu(v) => $macro!(v),
        }
    };
}

#[pymethods]
impl ContextMenu {
    #[staticmethod]
    fn popup(
        py: Python<'_>,
        slf: ImplContextMenu,
        window: Py<ext_mod_impl::window::Window>,
    ) -> PyResult<()> {
        py.allow_threads(|| {
            let window = window.get().0.inner_ref().to_owned();
            macro_rules! popup_impl {
                ($wrapper:expr) => {{
                    let context_menu = $wrapper.get().0.inner_ref();
                    context_menu.popup(window).map_err(TauriError::from)?;
                    Ok(())
                }};
            }
            context_menu_impl!(slf, popup_impl)
        })
    }

    #[staticmethod]
    fn popup_at(
        py: Python<'_>,
        slf: ImplContextMenu,
        window: Py<ext_mod_impl::window::Window>,
        position: ext_mod_impl::Position,
    ) -> PyResult<()> {
        py.allow_threads(|| {
            let window = window.get().0.inner_ref().to_owned();
            macro_rules! popup_at_impl {
                ($wrapper:expr) => {{
                    let context_menu = $wrapper.get().0.inner_ref();
                    context_menu
                        .popup_at(window, position)
                        .map_err(TauriError::from)?;
                    Ok(())
                }};
            }
            context_menu_impl!(slf, popup_at_impl)
        })
    }
}
