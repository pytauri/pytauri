use pyo3::{
    prelude::*,
    types::{PyDict, PyFloat, PyString},
};
use pyo3_utils::{
    from_py_dict::{derive_from_py_dict, FromPyDict as _, NotRequired},
    py_wrapper::{PyWrapper, PyWrapperT0},
};
use tauri::window;

use crate::{
    ext_mod::{webview::Color, PhysicalPositionI32, PhysicalRect, PhysicalSizeU32},
    tauri_runtime::Runtime,
    utils::non_exhaustive_panic,
};

type TauriWindow = window::Window<Runtime>;

/// See also: [tauri::window::Window]
#[pyclass(frozen)]
#[non_exhaustive]
pub struct Window(pub PyWrapper<PyWrapperT0<TauriWindow>>);

impl Window {
    pub(crate) fn new(window: TauriWindow) -> Self {
        Self(PyWrapper::new0(window))
    }
}

/// See also: [tauri::window::Monitor]
#[pyclass(frozen)]
#[non_exhaustive]
pub struct Monitor {
    #[pyo3(get)]
    name: Option<Py<PyString>>,
    #[pyo3(get)]
    size: PhysicalSizeU32,
    #[pyo3(get)]
    position: PhysicalPositionI32,
    #[pyo3(get)]
    work_area: Py<PhysicalRect>,
    #[pyo3(get)]
    scale_factor: Py<PyFloat>,
}

impl Monitor {
    pub(crate) fn from_tauri(py: Python<'_>, monitor: window::Monitor) -> PyResult<Self> {
        let name = monitor.name().map(|n| PyString::new(py, n).into());
        let size = PhysicalSizeU32::from_tauri(py, *monitor.size())?;
        let position = PhysicalPositionI32::from_tauri(py, *monitor.position())?;
        let work_area = Py::new(py, PhysicalRect::from_tauri(py, *monitor.work_area())?)?;
        let scale_factor = PyFloat::new(py, monitor.scale_factor()).into();
        Ok(Self {
            name,
            size,
            position,
            work_area,
            scale_factor,
        })
    }
}

macro_rules! effect_impl {
    ($ident:ident => : $($variant:ident),*) => {
        /// See also: [tauri::window::Effect]
        #[pyclass(frozen, eq, eq_int)]
        #[derive(PartialEq, Clone, Copy)]
        #[non_exhaustive]
        pub enum $ident {
            $($variant,)*
        }

        impl From<tauri::window::Effect> for $ident {
            fn from(val: tauri::window::Effect) -> Self {
                #[expect(deprecated)]
                match val {
                    $(tauri::window::Effect::$variant => $ident::$variant,)*
                }
            }
        }

        impl From<$ident> for tauri::window::Effect {
            fn from(val: $ident) -> Self {
                #[expect(deprecated)]
                match val {
                    $($ident::$variant => tauri::window::Effect::$variant,)*
                }
            }
        }
    };
}

effect_impl!(
    Effect => :
    AppearanceBased,
    Light,
    Dark,
    MediumLight,
    UltraDark,
    Titlebar,
    Selection,
    Menu,
    Popover,
    Sidebar,
    HeaderView,
    Sheet,
    WindowBackground,
    HudWindow,
    FullScreenUI,
    Tooltip,
    ContentBackground,
    UnderWindowBackground,
    UnderPageBackground,
    Mica,
    MicaDark,
    MicaLight,
    Tabbed,
    TabbedDark,
    TabbedLight,
    Blur,
    Acrylic
);

macro_rules! effect_state_impl {
    ($ident:ident => : $($variant:ident),*) => {
        /// See also: [tauri::window::EffectState]
        #[pyclass(frozen, eq, eq_int)]
        #[derive(PartialEq, Clone, Copy)]
        #[non_exhaustive]
        pub enum $ident {
            $($variant,)*
        }

        impl From<tauri::window::EffectState> for $ident {
            fn from(val: tauri::window::EffectState) -> Self {
                match val {
                    $(tauri::window::EffectState::$variant => $ident::$variant,)*
                }
            }
        }

        impl From<$ident> for tauri::window::EffectState {
            fn from(val: $ident) -> Self {
                match val {
                    $($ident::$variant => tauri::window::EffectState::$variant,)*
                }
            }
        }
    };
}

effect_state_impl!(EffectState => : FollowsWindowActiveState, Active, Inactive);

/// See also: [tauri::window::EffectsBuilder]
pub struct Effects {
    effects: NotRequired<Vec<Effect>>,
    state: NotRequired<EffectState>,
    radius: NotRequired<f64>,
    color: NotRequired<Color>,
}

derive_from_py_dict!(Effects {
    #[pyo3(default)]
    effects,
    #[pyo3(default)]
    state,
    #[pyo3(default)]
    radius,
    #[pyo3(default)]
    color,
});

impl<'py> FromPyObject<'py> for Effects {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let dict = ob.downcast::<PyDict>()?;
        Self::from_py_dict(dict)
    }
}

impl Effects {
    // NOTE: We do not want to use [tauri::utils::config::WindowEffectsConfig],
    // because it comes from `tauri_utils` (which may be unstable).
    pub(crate) fn into_tauri(self) -> window::EffectsBuilder {
        let mut builder = window::EffectsBuilder::new();
        let Self {
            effects,
            state,
            radius,
            color,
        } = self;

        if let Some(effects) = effects.0 {
            let effects = effects.into_iter().map(Into::into);
            builder = builder.effects(effects);
        }
        if let Some(state) = state.0 {
            builder = builder.state(state.into());
        }
        if let Some(radius) = radius.0 {
            builder = builder.radius(radius);
        }
        if let Some(color) = color.0 {
            builder = builder.color(color.0);
        }
        builder
    }
}

macro_rules! progress_bar_status_impl {
    ($ident:ident => : $( $(#[$meta:meta])* $variant:ident ),*) => {
        /// See also: [tauri::window::ProgressBarStatus]
        #[pyclass(frozen, eq, eq_int)]
        #[derive(PartialEq, Clone, Copy)]
        #[non_exhaustive]
        pub enum $ident {
            $(
                $(#[$meta])*
                $variant,
            )*
        }

        impl From<tauri::window::ProgressBarStatus> for $ident {
            fn from(val: tauri::window::ProgressBarStatus) -> Self {
                match val {
                    $(tauri::window::ProgressBarStatus::$variant => $ident::$variant,)*
                }
            }
        }

        impl From<$ident> for tauri::window::ProgressBarStatus {
            fn from(val: $ident) -> Self {
                match val {
                    $($ident::$variant => tauri::window::ProgressBarStatus::$variant,)*
                }
            }
        }
    };
}

// See also: [tauri::window::ProgressBarStatus]
progress_bar_status_impl!(
    ProgressBarStatus => :
    #[pyo3(name = "None_")]
    None,
    Normal,
    Indeterminate,
    Paused,
    Error
);

/// See also: [tauri::window::ProgressBarState]
pub struct ProgressBarState {
    status: NotRequired<ProgressBarStatus>,
    progress: NotRequired<u64>,
}

derive_from_py_dict!(ProgressBarState {
    #[pyo3(default)]
    status,
    #[pyo3(default)]
    progress,
});

impl<'py> FromPyObject<'py> for ProgressBarState {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let dict = ob.downcast::<PyDict>()?;
        Self::from_py_dict(dict)
    }
}

impl ProgressBarState {
    pub(crate) fn into_tauri(self) -> window::ProgressBarState {
        let Self { status, progress } = self;

        let status = status.0.map(Into::into);
        let progress = progress.0;

        window::ProgressBarState { status, progress }
    }
}

macro_rules! title_bar_style_impl {
    ($ident:ident => : $($variant:ident),*) => {
        /// See also: [tauri::TitleBarStyle]
        #[pyclass(frozen, eq, eq_int)]
        #[derive(PartialEq, Clone, Copy)]
        #[non_exhaustive]
        pub enum $ident {
            $($variant,)*
            _NonExhaustive,
        }

        impl From<tauri::TitleBarStyle> for $ident {
            fn from(val: tauri::TitleBarStyle) -> Self {
                match val {
                    $(tauri::TitleBarStyle::$variant => $ident::$variant,)*
                    _ => { $ident::_NonExhaustive }
                }
            }
        }

        impl From<$ident> for tauri::TitleBarStyle {
            fn from(val: $ident) -> Self {
                match val {
                    $($ident::$variant => tauri::TitleBarStyle::$variant,)*
                    $ident::_NonExhaustive => non_exhaustive_panic(),
                }
            }
        }
    };
}

title_bar_style_impl!(
    TitleBarStyle => :
    Visible,
    Transparent,
    Overlay
);
