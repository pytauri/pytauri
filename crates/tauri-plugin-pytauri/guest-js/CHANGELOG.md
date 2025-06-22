# tauri-plugin-pytauri-api

## [Unreleased]

### BREAKING

- [#178](https://github.com/pytauri/pytauri/pull/178) - feat(plugin-api)!: remove `rawPyInvoke` and `Channel`.

    !!! tip "Migration"

        ```diff
        -import { rawPyInvoke, Channel } from "tauri-plugin-pytauri-api";
        +import { pyInvoke } from "tauri-plugin-pytauri-api";
        +import { Channel } from "@tauri-apps/api/core";
        ```

    Before `v0.7`, IPC between Python and the frontend always used `bytes`/`ArrayBuffer` for data transfer. `pyInvoke` and `Channel.addJsonListener` would force serialization and deserialization of input and output, while `rawPyInvoke` and `Channel.onmessage` would directly send and receive raw `ArrayBuffer` data. This mechanism has been improved in `v0.7`:

    - Now, `pyInvoke` automatically chooses whether to serialize based on your input type:

        - `ArrayBuffer`/`Uint8Array` will be sent directly to the backend.
        - Other types (`any`) will be converted to `ArrayBuffer` using `JSON.stringify` and `TextEncoder` before being sent to the backend.

        [Only the backend can decide whether to deserialize or accept the raw bytes data](https://pytauri.github.io/pytauri/0.7/usage/concepts/ipc/#commands).

    - Only the backend can decide whether to return deserialized JSON data or raw `ArrayBuffer` data:

        - If `Command` returns a `bytes` type or [Channel.send][pytauri.ipc.Channel.send] sends a `bytes` type, the frontend will receive an `ArrayBuffer`.
        - If `Command` returns other types (`BaseModel`/`Any`) and [Channel.send][pytauri.ipc.Channel.send] sends a `str` type, the frontend will receive automatically deserialized JSON data.

    Therefore:

    - `pyInvoke` replaces `rawPyInvoke`
    - `import { Channel } from "@tauri-apps/api/core"` replaces `import { Channel } from "tauri-plugin-pytauri-api"`.

## [0.6.0]

## [0.5.0]

### Added

- [#136](https://github.com/pytauri/pytauri/pull/136) - feat(pytauri): accessing the request headers in `Commands`:

    Added `options?: InvokeOptions` parameter to `rawPyInvoke` and `pyInvoke`.

## [0.4.0]

## [0.3.0]

## [0.2.0]

### Added

- [#50](https://github.com/pytauri/pytauri/pull/50) - feat: add `class Channel extends TauriChannel<ArrayBuffer>` for pytauri [channels ipc](https://tauri.app/develop/calling-frontend/#channels).

### Docs

- [#50](https://github.com/pytauri/pytauri/pull/50) - add tsdoc for all classes and functions.

## [0.1.0-beta.0]

[unreleased]: https://github.com/pytauri/pytauri/tree/HEAD
[0.6.0]: https://github.com/pytauri/pytauri/releases/tag/js/tauri-plugin-pytauri-api/v0.6.0
[0.5.0]: https://github.com/pytauri/pytauri/releases/tag/js/tauri-plugin-pytauri-api/v0.5.0
[0.4.0]: https://github.com/pytauri/pytauri/releases/tag/js/tauri-plugin-pytauri-api/v0.4.0
[0.3.0]: https://github.com/pytauri/pytauri/releases/tag/js/tauri-plugin-pytauri-api/v0.3.0
[0.2.0]: https://github.com/pytauri/pytauri/releases/tag/js/tauri-plugin-pytauri-api/v0.2.0
[0.1.0-beta.0]: https://github.com/pytauri/pytauri/releases/tag/js/tauri-plugin-pytauri-api/v0.1.0-beta.0
