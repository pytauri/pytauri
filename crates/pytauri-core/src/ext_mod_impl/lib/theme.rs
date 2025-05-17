use pyo3::prelude::*;

macro_rules! theme_impl {
    ($ident:ident => : $($variant:ident),*) => {
        /// see also: [tauri::Theme]
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
                    $ident::_NonExhaustive => panic!("NonExhaustive is reserved for `#[non_exhaustive]`"),
                }
            }
        }
    };
}

theme_impl!(Theme => : Light, Dark);
