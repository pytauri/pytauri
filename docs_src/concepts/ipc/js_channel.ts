import { pyInvoke } from "tauri-plugin-pytauri-api";
import { Channel } from "@tauri-apps/api/core";
// const { pyInvoke } = window.__TAURI__.pytauri;
// const { Channel } = window.__TAURI__.core;

const channel = new Channel<string>((msg) => console.log(msg));

await pyInvoke("command", channel);
