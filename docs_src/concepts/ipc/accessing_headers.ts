import { pyInvoke } from "tauri-plugin-pytauri-api";

const buffer = new ArrayBuffer(16);
const output = await pyInvoke<null>("command", buffer, {
  headers: {
    foo: "bar"
  }
});
