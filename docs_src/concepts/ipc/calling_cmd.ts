import { pyInvoke, rawPyInvoke } from "tauri-plugin-pytauri-api";
// or if tauri config `app.withGlobalTauri = true`:
//
// ```js
// const { pyInvoke, rawPyInvoke } = window.__TAURI__.pytauri;
// ```

const output = await pyInvoke<[string]>("command", { foo: "foo", bar: 42 });
