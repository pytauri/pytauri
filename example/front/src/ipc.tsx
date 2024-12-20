import { pyInvoke } from "tauri-plugin-pytauri-api";
// or if tauri config `app.withGlobalTauri = true`:
//
// ```js
// const {pyInvoke, fromJson} = window.__TAURI__.pytauri
// ```

export interface Person {
    name: string;
}

export interface Greeting {
    message: string;
}

export async function greet(person: Person): Promise<Greeting> {
    // NOTE: DO NOT use `greet.name` as `funcName` parameter,
    // rollup will change the identifier of `greet` function.
    // see: <https://github.com/rollup/rollup/issues/1914>
    return await pyInvoke("greet", person);
}
