# Why are these icons needed?

See <https://github.com/tauri-apps/tauri/issues/4097>, we need to provide placeholder `icon.ico` and `icon.png`, or `tauri-build` will panic.

TODO, FIXME: This seems to be a bug caused by <https://github.com/tauri-apps/tauri/blob/339a075e33292dab67766d56a8b988e46640f490/crates/tauri-codegen/src/context.rs#L211-L244>, and we need to report this issue to the Tauri team.
