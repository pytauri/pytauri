use std::ops::Deref;

use pyo3::{marker::Ungil, prelude::*, types::PyString};
use pyo3_utils::{
    py_wrapper::{PyWrapper, PyWrapperT0},
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

/// see also: [tauri::menu::MenuId].
///
/// Remember use [MenuID::intern] to create a new instance.
pub type MenuID = PyString;
/// see also: [tauri::menu::MenuEvent]
///
/// Remember use [MenuEvent::intern] to create a new instance.
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

impl MenuItemKind {
    #[inline]
    fn delegate_inner_ref<R>(&self, f: impl FnOnce(&dyn IsMenuItem<Runtime>) -> R) -> R {
        match self {
            MenuItemKind::MenuItem(v) => f(&*v.get().0.inner_ref()),
            MenuItemKind::Submenu(v) => f(&*v.get().0.inner_ref()),
            MenuItemKind::Predefined(v) => f(&*v.get().0.inner_ref()),
            MenuItemKind::Check(v) => f(&*v.get().0.inner_ref()),
            MenuItemKind::Icon(v) => f(&*v.get().0.inner_ref()),
        }
    }
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
        self.delegate_inner_ref(|item| menu.append(item))
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
        self.delegate_inner_ref(|item| menu.prepend(item))
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
        self.delegate_inner_ref(|item| menu.insert(item, position))
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
        self.delegate_inner_ref(|item| menu.remove(item))
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
        }
        .map_err(TauriError::from)
        .map_err(PyErr::from)
    }
}

// All methods must release the GIL, because `menu` call `run_on_main_thread` internally, which may block.
#[pymethods]
impl Menu {
    #[new]
    fn __new__(py: Python<'_>, manager: ImplManager) -> PyResult<Self> {
        manager_method_impl!(py, &manager, |py, manager| Self::new_impl(
            py,
            manager,
            None::<&str>,
            None
        ))?
    }

    #[staticmethod]
    fn with_id(py: Python<'_>, manager: ImplManager, id: String) -> PyResult<Self> {
        manager_method_impl!(py, &manager, |py, manager| Self::new_impl(
            py,
            manager,
            Some(MenuId(id)),
            None
        ))?
    }

    #[staticmethod]
    fn with_items(
        py: Python<'_>,
        manager: ImplManager,
        items: Vec<MenuItemKind>,
    ) -> PyResult<Self> {
        manager_method_impl!(py, &manager, |py, manager| Self::new_impl(
            py,
            manager,
            None::<&str>,
            Some(items)
        ))?
    }

    #[staticmethod]
    fn with_id_and_items(
        py: Python<'_>,
        manager: ImplManager,
        id: String,
        items: Vec<MenuItemKind>,
    ) -> PyResult<Self> {
        manager_method_impl!(py, &manager, |py, manager| Self::new_impl(
            py,
            manager,
            Some(MenuId(id)),
            Some(items)
        ))?
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
        // TODO, PERF: release the GIL?
        let app_handle = menu.app_handle().py_app_handle().clone_ref(py);
        app_handle
    }

    fn id<'py>(&self, py: Python<'py>) -> Bound<'py, MenuID> {
        let menu = self.0.inner_ref();
        MenuID::intern(py, &menu.id().0)
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
        }
        .map_err(TauriError::from)
        .map_err(PyErr::from)
    }
}

// All methods must release the GIL, because `menu` call `run_on_main_thread` internally, which may block.
#[pymethods]
impl Submenu {
    #[new]
    fn __new__(py: Python<'_>, manager: ImplManager, text: &str, enabled: bool) -> PyResult<Self> {
        manager_method_impl!(py, &manager, |py, manager| Self::new_impl(
            py,
            manager,
            text,
            enabled,
            None::<&str>,
            None
        ))?
    }

    #[staticmethod]
    fn with_id(
        py: Python<'_>,
        manager: ImplManager,
        id: String,
        text: &str,
        enabled: bool,
    ) -> PyResult<Self> {
        manager_method_impl!(py, &manager, |py, manager| Self::new_impl(
            py,
            manager,
            text,
            enabled,
            Some(MenuId(id)),
            None
        ))?
    }

    #[staticmethod]
    fn with_items(
        py: Python<'_>,
        manager: ImplManager,
        text: &str,
        enabled: bool,
        items: Vec<MenuItemKind>,
    ) -> PyResult<Self> {
        manager_method_impl!(py, &manager, |py, manager| Self::new_impl(
            py,
            manager,
            text,
            enabled,
            None::<&str>,
            Some(items)
        ))?
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
        manager_method_impl!(py, &manager, |py, manager| Self::new_impl(
            py,
            manager,
            text,
            enabled,
            Some(MenuId(id)),
            Some(items),
        ))?
    }

    fn app_handle(&self, py: Python<'_>) -> Py<ext_mod_impl::AppHandle> {
        let menu = self.0.inner_ref();
        // TODO, PERF: release the GIL?
        let app_handle = menu.app_handle().py_app_handle().clone_ref(py);
        app_handle
    }

    fn id<'py>(&self, py: Python<'py>) -> Bound<'py, MenuID> {
        let menu = self.0.inner_ref();
        MenuID::intern(py, &menu.id().0)
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
        }
        .map_err(TauriError::from)
        .map_err(PyErr::from)
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
        manager_method_impl!(py, &manager, |py, manager| Self::new_impl(
            py,
            manager,
            text,
            enabled,
            accelerator,
            None::<&str>
        ))?
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
        manager_method_impl!(py, &manager, |py, manager| Self::new_impl(
            py,
            manager,
            text,
            enabled,
            accelerator,
            Some(MenuId(id)),
        ))?
    }

    fn app_handle(&self, py: Python<'_>) -> Py<ext_mod_impl::AppHandle> {
        let menu = self.0.inner_ref();
        // TODO, PERF: release the GIL?
        let app_handle = menu.app_handle().py_app_handle().clone_ref(py);
        app_handle
    }

    fn id<'py>(&self, py: Python<'py>) -> Bound<'py, MenuID> {
        let menu = self.0.inner_ref();
        MenuID::intern(py, &menu.id().0)
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
        manager_method_impl!(py, &manager, |py, manager| Self::delegate_inner(
            py,
            manager,
            move |manager| { TauriPredefinedMenuItem::separator(manager) }
        ))?
    }

    #[staticmethod]
    #[pyo3(signature = (manager, text=None))]
    fn copy(py: Python<'_>, manager: ImplManager, text: Option<&str>) -> PyResult<Self> {
        manager_method_impl!(py, &manager, |py, manager| Self::delegate_inner(
            py,
            manager,
            move |manager| { TauriPredefinedMenuItem::copy(manager, text) }
        ))?
    }

    #[staticmethod]
    #[pyo3(signature = (manager, text=None))]
    fn cut(py: Python<'_>, manager: ImplManager, text: Option<&str>) -> PyResult<Self> {
        manager_method_impl!(py, &manager, |py, manager| Self::delegate_inner(
            py,
            manager,
            move |manager| { TauriPredefinedMenuItem::cut(manager, text) }
        ))?
    }

    #[staticmethod]
    #[pyo3(signature = (manager, text=None))]
    fn paste(py: Python<'_>, manager: ImplManager, text: Option<&str>) -> PyResult<Self> {
        manager_method_impl!(py, &manager, |py, manager| Self::delegate_inner(
            py,
            manager,
            move |manager| { TauriPredefinedMenuItem::paste(manager, text) }
        ))?
    }

    #[staticmethod]
    #[pyo3(signature = (manager, text=None))]
    fn select_all(py: Python<'_>, manager: ImplManager, text: Option<&str>) -> PyResult<Self> {
        manager_method_impl!(py, &manager, |py, manager| Self::delegate_inner(
            py,
            manager,
            move |manager| { TauriPredefinedMenuItem::select_all(manager, text) }
        ))?
    }

    #[staticmethod]
    #[pyo3(signature = (manager, text=None))]
    fn undo(py: Python<'_>, manager: ImplManager, text: Option<&str>) -> PyResult<Self> {
        manager_method_impl!(py, &manager, |py, manager| Self::delegate_inner(
            py,
            manager,
            move |manager| { TauriPredefinedMenuItem::undo(manager, text) }
        ))?
    }

    #[staticmethod]
    #[pyo3(signature = (manager, text=None))]
    fn redo(py: Python<'_>, manager: ImplManager, text: Option<&str>) -> PyResult<Self> {
        manager_method_impl!(py, &manager, |py, manager| Self::delegate_inner(
            py,
            manager,
            move |manager| { TauriPredefinedMenuItem::redo(manager, text) }
        ))?
    }

    #[staticmethod]
    #[pyo3(signature = (manager, text=None))]
    fn minimize(py: Python<'_>, manager: ImplManager, text: Option<&str>) -> PyResult<Self> {
        manager_method_impl!(py, &manager, |py, manager| Self::delegate_inner(
            py,
            manager,
            move |manager| { TauriPredefinedMenuItem::minimize(manager, text) }
        ))?
    }

    #[staticmethod]
    #[pyo3(signature = (manager, text=None))]
    fn maximize(py: Python<'_>, manager: ImplManager, text: Option<&str>) -> PyResult<Self> {
        manager_method_impl!(py, &manager, |py, manager| Self::delegate_inner(
            py,
            manager,
            move |manager| { TauriPredefinedMenuItem::maximize(manager, text) }
        ))?
    }

    #[staticmethod]
    #[pyo3(signature = (manager, text=None))]
    fn fullscreen(py: Python<'_>, manager: ImplManager, text: Option<&str>) -> PyResult<Self> {
        manager_method_impl!(py, &manager, |py, manager| Self::delegate_inner(
            py,
            manager,
            move |manager| { TauriPredefinedMenuItem::fullscreen(manager, text) }
        ))?
    }

    #[staticmethod]
    #[pyo3(signature = (manager, text=None))]
    fn hide(py: Python<'_>, manager: ImplManager, text: Option<&str>) -> PyResult<Self> {
        manager_method_impl!(py, &manager, |py, manager| Self::delegate_inner(
            py,
            manager,
            move |manager| { TauriPredefinedMenuItem::hide(manager, text) }
        ))?
    }

    #[staticmethod]
    #[pyo3(signature = (manager, text=None))]
    fn hide_others(py: Python<'_>, manager: ImplManager, text: Option<&str>) -> PyResult<Self> {
        manager_method_impl!(py, &manager, |py, manager| Self::delegate_inner(
            py,
            manager,
            move |manager| { TauriPredefinedMenuItem::hide_others(manager, text) }
        ))?
    }

    #[staticmethod]
    #[pyo3(signature = (manager, text=None))]
    fn show_all(py: Python<'_>, manager: ImplManager, text: Option<&str>) -> PyResult<Self> {
        manager_method_impl!(py, &manager, |py, manager| Self::delegate_inner(
            py,
            manager,
            move |manager| { TauriPredefinedMenuItem::show_all(manager, text) }
        ))?
    }

    #[staticmethod]
    #[pyo3(signature = (manager, text=None))]
    fn close_window(py: Python<'_>, manager: ImplManager, text: Option<&str>) -> PyResult<Self> {
        manager_method_impl!(py, &manager, |py, manager| Self::delegate_inner(
            py,
            manager,
            move |manager| { TauriPredefinedMenuItem::close_window(manager, text) }
        ))?
    }

    #[staticmethod]
    #[pyo3(signature = (manager, text=None))]
    fn quit(py: Python<'_>, manager: ImplManager, text: Option<&str>) -> PyResult<Self> {
        manager_method_impl!(py, &manager, |py, manager| Self::delegate_inner(
            py,
            manager,
            move |manager| { TauriPredefinedMenuItem::quit(manager, text) }
        ))?
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

        manager_method_impl!(py, &manager, |py, manager| Self::delegate_inner(
            py,
            manager,
            move |manager| { TauriPredefinedMenuItem::about(manager, text, metadata) }
        ))?
    }

    #[staticmethod]
    #[pyo3(signature = (manager, text=None))]
    fn services(py: Python<'_>, manager: ImplManager, text: Option<&str>) -> PyResult<Self> {
        manager_method_impl!(py, &manager, |py, manager| Self::delegate_inner(
            py,
            manager,
            move |manager| { TauriPredefinedMenuItem::services(manager, text) }
        ))?
    }

    fn app_handle(&self, py: Python<'_>) -> Py<ext_mod_impl::AppHandle> {
        let menu = self.0.inner_ref();
        // TODO, PERF: release the GIL?
        let app_handle = menu.app_handle().py_app_handle().clone_ref(py);
        app_handle
    }

    fn id<'py>(&self, py: Python<'py>) -> Bound<'py, MenuID> {
        let menu = self.0.inner_ref();
        MenuID::intern(py, &menu.id().0)
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
        }
        .map_err(TauriError::from)
        .map_err(PyErr::from)
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
        manager_method_impl!(py, &manager, |py, manager| Self::new_impl(
            py,
            manager,
            text,
            enabled,
            checked,
            accelerator,
            None::<&str>,
        ))?
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
        manager_method_impl!(py, &manager, |py, manager| Self::new_impl(
            py,
            manager,
            text,
            enabled,
            checked,
            accelerator,
            Some(MenuId(id)),
        ))?
    }

    fn app_handle(&self, py: Python<'_>) -> Py<ext_mod_impl::AppHandle> {
        let menu = self.0.inner_ref();
        // TODO, PERF: release the GIL?
        let app_handle = menu.app_handle().py_app_handle().clone_ref(py);
        app_handle
    }

    fn id<'py>(&self, py: Python<'py>) -> Bound<'py, MenuID> {
        let menu = self.0.inner_ref();
        MenuID::intern(py, &menu.id().0)
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
        }
        .map_err(TauriError::from)
        .map_err(PyErr::from)
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

        manager_method_impl!(py, &manager, |py, manager| Self::new_impl(
            py,
            manager,
            text,
            enabled,
            icon,
            accelerator,
            None::<&str>,
        ))?
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

        manager_method_impl!(py, &manager, |py, manager| Self::new_impl(
            py,
            manager,
            text,
            enabled,
            icon,
            accelerator,
            Some(MenuId(id)),
        ))?
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
        let native_icon = native_icon.map(|native_icon| native_icon.into());
        let native_icon = IconOrNative::Native(native_icon);

        manager_method_impl!(py, &manager, |py, manager| Self::new_impl(
            py,
            manager,
            text,
            enabled,
            native_icon,
            accelerator,
            None::<&str>,
        ))?
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
        let native_icon = native_icon.map(|native_icon| native_icon.into());
        let native_icon = IconOrNative::Native(native_icon);

        manager_method_impl!(py, &manager, |py, manager| Self::new_impl(
            py,
            manager,
            text,
            enabled,
            native_icon,
            accelerator,
            Some(MenuId(id)),
        ))?
    }

    fn app_handle(&self, py: Python<'_>) -> Py<ext_mod_impl::AppHandle> {
        let menu = self.0.inner_ref();
        // TODO, PERF: release the GIL?
        let app_handle = menu.app_handle().py_app_handle().clone_ref(py);
        app_handle
    }

    fn id<'py>(&self, py: Python<'py>) -> Bound<'py, MenuID> {
        let menu = self.0.inner_ref();
        MenuID::intern(py, &menu.id().0)
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
        let native_icon = native_icon.map(|native_icon| native_icon.into());
        py.allow_threads(|| {
            let menu = self.0.inner_ref();
            menu.set_native_icon(native_icon)
                .map_err(TauriError::from)?;
            Ok(())
        })
    }
}

macro_rules! native_icon_impl {
    ($ident:ident => : $($variant:ident),*) => {
        /// see also: [tauri::menu::NativeIcon]
        #[pyclass(frozen, eq, eq_int)]
        #[derive(PartialEq, Clone, Copy)]
        pub enum $ident {
            $($variant,)*
        }

        impl From<$ident> for tauri::menu::NativeIcon {
            fn from(icon: $ident) -> Self {
                match icon {
                    $($ident::$variant => tauri::menu::NativeIcon::$variant,)*
                }
            }
        }

        impl From<tauri::menu::NativeIcon> for $ident {
            fn from(icon: tauri::menu::NativeIcon) -> Self {
                match icon {
                    $(tauri::menu::NativeIcon::$variant => $ident::$variant,)*
                }
            }
        }

    };
}

native_icon_impl!(
    NativeIcon => :
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

/// The Implementers of [tauri::menu::ContextMenu].
#[derive(FromPyObject, IntoPyObject, IntoPyObjectRef)]
#[non_exhaustive]
pub enum ImplContextMenu {
    Menu(Py<Menu>),
    Submenu(Py<Submenu>),
}

impl ImplContextMenu {
    pub(crate) fn _delegate_inner_ref<M, R>(menu: &M, f: impl FnOnce(&M) -> R) -> R
    where
        M: menu::ContextMenu,
    {
        f(menu)
    }
}

/// see [crate::manager_method_impl]
#[doc(hidden)]
#[macro_export]
macro_rules! context_menu_impl {
    // impl
    ($menu:expr, $f0:expr, $f1:expr) => {{
        use $crate::ext_mod_impl::menu::ImplContextMenu;

        let menu: &ImplContextMenu = $menu;
        match menu {
            ImplContextMenu::Menu(v) => {
                ImplContextMenu::_delegate_inner_ref(&*v.get().0.inner_ref(), $f0)
            }
            ImplContextMenu::Submenu(v) => {
                ImplContextMenu::_delegate_inner_ref(&*v.get().0.inner_ref(), $f1)
            }
        }
    }};

    // entry0
    ($menu:expr, $($f:tt)*) => {
        context_menu_impl!($menu, $($f)*, $($f)*)
    };
}

/// See also: [tauri::menu::ContextMenu].
#[pyclass(frozen)]
#[non_exhaustive]
pub struct ContextMenu;

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
            context_menu_impl!(&slf, |menu| {
                menu.popup(window)
                    .map_err(TauriError::from)
                    .map_err(PyErr::from)
            })
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
            context_menu_impl!(&slf, |menu| {
                menu.popup_at(window, position)
                    .map_err(TauriError::from)
                    .map_err(PyErr::from)
            })
        })
    }
}
