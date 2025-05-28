use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
};

use pyo3::prelude::*;
use pyo3_utils::py_wrapper::{PyWrapper, PyWrapperT2};
use tauri_plugin_notification::{self as plugin, NotificationExt as _};

use crate::{
    ext_mod::{manager_method_impl, ImplManager},
    tauri_runtime::Runtime,
};

#[derive(Debug)]
struct PluginError(plugin::Error);

impl Display for PluginError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Error for PluginError {}

impl From<PluginError> for PyErr {
    fn from(value: PluginError) -> Self {
        match value.0 {
            plugin::Error::Io(e) => e.into(),
        }
    }
}

impl From<plugin::Error> for PluginError {
    fn from(value: plugin::Error) -> Self {
        Self(value)
    }
}

#[pyclass(frozen)]
#[non_exhaustive]
pub struct NotificationBuilder(pub PyWrapper<PyWrapperT2<plugin::NotificationBuilder<Runtime>>>);

impl NotificationBuilder {
    fn new(builder: plugin::NotificationBuilder<Runtime>) -> Self {
        Self(PyWrapper::new2(builder))
    }
}

#[pymethods]
impl NotificationBuilder {
    #[pyo3(signature = (
        *,
        id = None,
        channel_id = None,
        title = None,
        body = None,
        large_body = None,
        summary = None,
        action_type_id = None,
        group = None,
        group_summary = false,
        sound = None,
        inbox_line = None,
        icon = None,
        large_icon = None,
        icon_color = None,
        ongoing = false,
        auto_cancel = false,
        silent = false
    ))]
    #[expect(clippy::too_many_arguments)]
    fn show(
        &self,
        py: Python<'_>,
        id: Option<i32>,
        channel_id: Option<String>,
        title: Option<String>,
        body: Option<String>,
        /* TODO: schedule */
        large_body: Option<String>,
        summary: Option<String>,
        action_type_id: Option<String>,
        group: Option<String>,
        group_summary: bool,
        sound: Option<String>,
        inbox_line: Option<String>,
        icon: Option<String>,
        large_icon: Option<String>,
        icon_color: Option<String>,
        /* TODO: attachment */
        /* TODO: extra */
        ongoing: bool,
        auto_cancel: bool,
        silent: bool,
    ) -> PyResult<()> {
        // TODO (perf): Do we really need `py.allow_threads` here?
        // I mean, I don't know how long `NotificationBuilder::show` will take,
        // maybe it's short enough?
        py.allow_threads(|| {
            let mut builder = self.0.try_take_inner()??;

            if let Some(id) = id {
                builder = builder.id(id);
            }
            if let Some(channel_id) = channel_id {
                builder = builder.channel_id(channel_id);
            }
            if let Some(title) = title {
                builder = builder.title(title);
            }
            if let Some(body) = body {
                builder = builder.body(body);
            }
            if let Some(large_body) = large_body {
                builder = builder.large_body(large_body);
            }
            if let Some(summary) = summary {
                builder = builder.summary(summary);
            }
            if let Some(action_type_id) = action_type_id {
                builder = builder.action_type_id(action_type_id);
            }
            if let Some(group) = group {
                builder = builder.group(group);
            }
            if group_summary {
                builder = builder.group_summary();
            }
            if let Some(sound) = sound {
                builder = builder.sound(sound);
            }
            if let Some(inbox_line) = inbox_line {
                builder = builder.inbox_line(inbox_line);
            }
            if let Some(icon) = icon {
                builder = builder.icon(icon);
            }
            if let Some(large_icon) = large_icon {
                builder = builder.large_icon(large_icon);
            }
            if let Some(icon_color) = icon_color {
                builder = builder.icon_color(icon_color);
            }
            if ongoing {
                builder = builder.ongoing();
            }
            if auto_cancel {
                builder = builder.auto_cancel();
            }
            if silent {
                builder = builder.silent();
            }

            builder
                .show()
                .map_err(Into::<PluginError>::into)
                .map_err(Into::<PyErr>::into)
        })
    }
}

#[pyclass(frozen)]
#[non_exhaustive]
pub struct NotificationExt;

pub type ImplNotificationExt = ImplManager;

#[pymethods]
impl NotificationExt {
    // TODO: Add `struct Notification` as an intermediate layer, currently blocked by:
    // <https://github.com/tauri-apps/plugins-workspace/issues/2161>

    #[staticmethod]
    fn builder(slf: ImplNotificationExt, py: Python<'_>) -> PyResult<NotificationBuilder> {
        manager_method_impl!(py, &slf, |_py, manager| {
            // PERF: it's short enough, so we don't release the GIL
            let builder = manager.notification().builder();
            Ok(NotificationBuilder::new(builder))
        })?
    }
}

#[pymodule(submodule, gil_used = false)]
pub mod notification {
    #[pymodule_export]
    pub use super::{NotificationBuilder, NotificationExt};

    pub use super::ImplNotificationExt;
}
