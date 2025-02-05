import sys

from pydantic import BaseModel
from pytauri import AppHandle, Commands
from pytauri_plugin_notification import NotificationExt

commands: Commands = Commands()


class Person(BaseModel):
    name: str


class Greeting(BaseModel):
    message: str


@commands.command()
async def greet(body: Person, app_handle: AppHandle) -> Greeting:
    notification_builder = NotificationExt.builder(app_handle)
    notification_builder.show(title="Greeting", body=f"Hello, {body.name}!")

    return Greeting(
        message=f"Hello, {body.name}! You've been greeted from Python {sys.version}!"
    )
