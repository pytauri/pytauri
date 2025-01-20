import { invoke } from "@tauri-apps/api/core";
import { pyInvoke, Channel } from "tauri-plugin-pytauri-api";
// or if tauri config `app.withGlobalTauri = true`:
//
// ```js
// const { pyInvoke } = window.__TAURI__.pytauri;
// ```

let greetInputEl: HTMLInputElement | null;
let greetMsgEl: HTMLElement | null;


async function greet() {
  if (greetMsgEl && greetInputEl) {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    const rsGreeting = await invoke<string>("greet", {
      name: greetInputEl.value,
    });
    const pyGreeting = await pyInvoke<string>("greet", {
      name: greetInputEl.value,
    });
    greetMsgEl.textContent = rsGreeting + "\n" + pyGreeting;
  }
}

window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  document.querySelector("#greet-form")?.addEventListener("submit", (e) => {
    e.preventDefault();
    greet();
  });

  const timeLabel = document.querySelector("#time-label");

  const timeChannel = new Channel<string>();
  timeChannel.addJsonListener((time) => {
    if (timeLabel) {
      timeLabel.textContent = time;
    }
  });

  pyInvoke("start_timer", timeChannel);
});
