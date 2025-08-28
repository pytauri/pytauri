use std::borrow::Cow;

use pyo3::prelude::*;
use pyo3_utils::serde::PySerde;

use crate::utils::non_exhaustive_panic;

/// See also: [tauri::Config]
#[expect(dead_code)] // TODO
pub(crate) type ConfigFrom = PySerde<tauri::Config>;
/// See also: [tauri::Config]
pub(crate) type ConfigInto<'a> = PySerde<Cow<'a, tauri::Config>>;

macro_rules! theme_impl {
    ($ident:ident => : $($variant:ident),*) => {
        /// See also: [tauri::Theme]
        #[pyclass(frozen, eq, eq_int)]
        #[derive(PartialEq, Clone, Copy)]
        #[non_exhaustive]
        pub enum $ident {
            $($variant,)*
            _NonExhaustive,
        }

        impl From<tauri::Theme> for $ident {
            fn from(val: tauri::Theme) -> Self {
                match val {
                    $(tauri::Theme::$variant => $ident::$variant,)*
                    _ => { $ident::_NonExhaustive }
                }
            }
        }

        impl From<$ident> for tauri::Theme {
            fn from(val: $ident) -> Self {
                match val {
                    $($ident::$variant => tauri::Theme::$variant,)*
                    $ident::_NonExhaustive => non_exhaustive_panic(),
                }
            }
        }
    };
}

theme_impl!(Theme => : Light, Dark);

macro_rules! user_attention_type_impl {
    ($ident:ident => : $($variant:ident),*) => {
        /// See also: [tauri::UserAttentionType]
        #[pyclass(frozen, eq, eq_int)]
        #[derive(PartialEq, Clone, Copy)]
        #[non_exhaustive]
        pub enum $ident {
            $($variant,)*
        }

        impl From<tauri::UserAttentionType> for $ident {
            fn from(val: tauri::UserAttentionType) -> Self {
                match val {
                    $(tauri::UserAttentionType::$variant => $ident::$variant,)*
                }
            }
        }

        impl From<$ident> for tauri::UserAttentionType {
            fn from(val: $ident) -> Self {
                match val {
                    $($ident::$variant => tauri::UserAttentionType::$variant,)*
                }
            }
        }
    };
}

user_attention_type_impl!(UserAttentionType => : Critical, Informational);

macro_rules! cursor_icon_impl {
    ($ident:ident => : $($variant:ident),*) => {
        /// See also: [tauri::CursorIcon]
        #[pyclass(frozen, eq, eq_int)]
        #[derive(PartialEq, Clone, Copy)]
        #[non_exhaustive]
        pub enum $ident {
            $($variant,)*
            _NonExhaustive,
        }

        impl From<tauri::CursorIcon> for $ident {
            fn from(val: tauri::CursorIcon) -> Self {
                match val {
                    $(tauri::CursorIcon::$variant => $ident::$variant,)*
                    _ => { $ident::_NonExhaustive }
                }
            }
        }

        impl From<$ident> for tauri::CursorIcon {
            fn from(val: $ident) -> Self {
                match val {
                    $($ident::$variant => tauri::CursorIcon::$variant,)*
                    $ident::_NonExhaustive => non_exhaustive_panic(),
                }
            }
        }
    };
}

cursor_icon_impl!(
    CursorIcon => :
    Default,
    Crosshair,
    Hand,
    Arrow,
    Move,
    Text,
    Wait,
    Help,
    Progress,
    NotAllowed,
    ContextMenu,
    Cell,
    VerticalText,
    Alias,
    Copy,
    NoDrop,
    Grab,
    Grabbing,
    AllScroll,
    ZoomIn,
    ZoomOut,
    EResize,
    NResize,
    NeResize,
    NwResize,
    SResize,
    SeResize,
    SwResize,
    WResize,
    EwResize,
    NsResize,
    NeswResize,
    NwseResize,
    ColResize,
    RowResize
);
