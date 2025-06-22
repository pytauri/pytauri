import { message } from '@tauri-apps/plugin-dialog';
import { pyInvoke } from "tauri-plugin-pytauri-api";
import { Channel } from "@tauri-apps/api/core";
// or if tauri config `app.withGlobalTauri = true`:
//
// ```js
// const { ask } = window.__TAURI__.dialog;
// const { pyInvoke } = window.__TAURI__.pytauri;
// ```

let greetInputEl: HTMLInputElement | null;
let greetMsgEl: HTMLElement | null;


async function greet() {
  if (greetMsgEl && greetInputEl) {
    const pyGreeting = await pyInvoke<string>("greet", {
      name: greetInputEl.value,
    });

    await message(pyGreeting, `Hi`);

    greetMsgEl.textContent = pyGreeting;
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

  const timeChannel = new Channel<string>((time) => {
    if (timeLabel) {
      timeLabel.textContent = time;
    }
  });

  pyInvoke("start_timer", timeChannel);
});
