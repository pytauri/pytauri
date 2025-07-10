use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
};

use pyo3::{prelude::*, types::PyDict};
use pyo3_utils::{
    from_py_dict::{derive_from_py_dict, FromPyDict as _, NotRequired},
    py_wrapper::{PyWrapper, PyWrapperT2},
};
use tauri_plugin_notification::{self as plugin, NotificationExt as _};

use crate::{
    ext_mod::{manager_method_impl, plugin::Plugin, ImplManager},
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

/// See also: [tauri_plugin_notification::init]
#[pyfunction]
pub fn init() -> Plugin {
    Plugin::new(|| Box::new(plugin::init::<Runtime>()))
}

/// See also: [tauri_plugin_notification::NotificationBuilder]
#[non_exhaustive]
pub struct NotificationBuilderArgs {
    id: NotRequired<i32>,
    channel_id: NotRequired<String>,
    title: NotRequired<String>,
    body: NotRequired<String>,
    /* TODO: schedule */
    large_body: NotRequired<String>,
    summary: NotRequired<String>,
    action_type_id: NotRequired<String>,
    group: NotRequired<String>,
    group_summary: bool,
    sound: NotRequired<String>,
    inbox_line: NotRequired<String>,
    icon: NotRequired<String>,
    large_icon: NotRequired<String>,
    icon_color: NotRequired<String>,
    /* TODO: attachment */
    /* TODO: extra */
    ongoing: bool,
    auto_cancel: bool,
    silent: bool,
}

derive_from_py_dict!(NotificationBuilderArgs {
    #[default]
    id,
    #[default]
    channel_id,
    #[default]
    title,
    #[default]
    body,
    #[default]
    large_body,
    #[default]
    summary,
    #[default]
    action_type_id,
    #[default]
    group,
    #[default]
    group_summary,
    #[default]
    sound,
    #[default]
    inbox_line,
    #[default]
    icon,
    #[default]
    large_icon,
    #[default]
    icon_color,
    #[default]
    ongoing,
    #[default]
    auto_cancel,
    #[default]
    silent,
});

impl NotificationBuilderArgs {
    // TODO: Maybe we can upstream this to pyo3,
    // so that we can directly use it as the type signature for `**kwargs`
    fn from_kwargs(kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<Option<Self>> {
        kwargs
            .map(NotificationBuilderArgs::from_py_dict)
            .transpose()
    }

    fn apply_to_builder(
        self,
        mut builder: plugin::NotificationBuilder<Runtime>,
    ) -> plugin::NotificationBuilder<Runtime> {
        let Self {
            id,
            channel_id,
            title,
            body,
            large_body,
            summary,
            action_type_id,
            group,
            group_summary,
            sound,
            inbox_line,
            icon,
            large_icon,
            icon_color,
            ongoing,
            auto_cancel,
            silent,
        } = self;

        if let Some(id) = id.0 {
            builder = builder.id(id);
        }
        if let Some(channel_id) = channel_id.0 {
            builder = builder.channel_id(channel_id);
        }
        if let Some(title) = title.0 {
            builder = builder.title(title);
        }
        if let Some(body) = body.0 {
            builder = builder.body(body);
        }
        if let Some(large_body) = large_body.0 {
            builder = builder.large_body(large_body);
        }
        if let Some(summary) = summary.0 {
            builder = builder.summary(summary);
        }
        if let Some(action_type_id) = action_type_id.0 {
            builder = builder.action_type_id(action_type_id);
        }
        if let Some(group) = group.0 {
            builder = builder.group(group);
        }
        if group_summary {
            builder = builder.group_summary();
        }
        if let Some(sound) = sound.0 {
            builder = builder.sound(sound);
        }
        if let Some(inbox_line) = inbox_line.0 {
            builder = builder.inbox_line(inbox_line);
        }
        if let Some(icon) = icon.0 {
            builder = builder.icon(icon);
        }
        if let Some(large_icon) = large_icon.0 {
            builder = builder.large_icon(large_icon);
        }
        if let Some(icon_color) = icon_color.0 {
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
    }
}

/// See also: [tauri_plugin_notification::NotificationBuilder]
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
    #[pyo3(signature = (**kwargs))]
    fn show(&self, kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<()> {
        let args = NotificationBuilderArgs::from_kwargs(kwargs)?;

        let mut builder = self.0.try_take_inner()??;

        if let Some(args) = args {
            builder = args.apply_to_builder(builder);
        }

        // PERF: it's short enough, so we don't release the GIL
        builder
            .show()
            .map_err(PluginError::from)
            .map_err(PyErr::from)
    }
}

/// See also: [tauri_plugin_notification::NotificationExt]
#[pyclass(frozen)]
#[non_exhaustive]
pub struct NotificationExt;

/// The Implementers of [tauri_plugin_notification::NotificationExt].
pub type ImplNotificationExt = ImplManager;

#[pymethods]
impl NotificationExt {
    #[staticmethod]
    fn builder(slf: ImplNotificationExt, py: Python<'_>) -> PyResult<NotificationBuilder> {
        manager_method_impl!(py, &slf, |_py, manager| {
            // PERF: it's short enough, so we don't release the GIL
            let builder = manager.notification().builder();
            Ok(NotificationBuilder::new(builder))
        })?
    }
}

/// See also: [tauri_plugin_notification]
#[pymodule(submodule, gil_used = false)]
pub mod notification {
    #[pymodule_export]
    pub use super::{init, NotificationBuilder, NotificationExt};

    pub use super::{ImplNotificationExt, NotificationBuilderArgs};
}
