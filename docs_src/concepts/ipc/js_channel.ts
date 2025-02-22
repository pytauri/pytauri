import { pyInvoke, Channel } from "tauri-plugin-pytauri-api";
// const { pyInvoke, Channel } = window.__TAURI__.pytauri;

const channel = new Channel<string>();
channel.addJsonListener((msg) => console.log(msg));

await pyInvoke("command", channel);
