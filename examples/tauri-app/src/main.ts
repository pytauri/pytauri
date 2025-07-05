import { invoke, Channel } from "@tauri-apps/api/core";
import { greet, startTimer } from "./client/apiClient";

let greetInputEl: HTMLInputElement | null;
let greetMsgEl: HTMLElement | null;


async function onGreet() {
  if (greetMsgEl && greetInputEl) {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    const rsGreeting = await invoke<string>("greet", {
      name: greetInputEl.value,
    });
    const pyGreeting = await greet({
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
    onGreet();
  });

  const timeLabel = document.querySelector("#time-label");

  const timeChannel = new Channel<string>(
    (time) => {
      if (timeLabel) {
        timeLabel.textContent = time;
      }
    }
  );

  startTimer(timeChannel.toJSON());
});
