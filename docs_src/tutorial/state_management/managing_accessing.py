from dataclasses import dataclass
from typing import Union

from pytauri import App, AppHandle, Manager
from pytauri.webview import WebviewWindow


# ⭐ `Manager` uses `type(state)` as the key to store state,
# so it's best to define a separate newtype to represent the state.
@dataclass
class MyState:
    value: int = 0


def manage_and_access_state(manager: Union[App, AppHandle, WebviewWindow]) -> None:
    state = MyState()

    # ⭐ Store the state
    Manager.manage(manager, state)

    # ⭐ Later, we can access this state elsewhere.
    state1: MyState = Manager.state(manager, MyState)
    state1.value = 42
    assert state1 is state
