//! tauri runtime configuration for pytauri.

/// The current [tauri::Runtime] for pytauri.
#[cfg(not(all(feature = "__test", not(feature = "__no_test"))))]
pub type Runtime = tauri::Wry;

#[cfg(all(feature = "__test", not(feature = "__no_test")))]
pub type Runtime = tauri::test::MockRuntime;
