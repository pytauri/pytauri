from pydantic import RootModel
from pytauri import Commands
from pytauri.ipc import Channel, JavaScriptChannelId
from pytauri.webview import WebviewWindow

commands = Commands()

Msg = RootModel[str]


@commands.command()
async def command(
    body: JavaScriptChannelId[Msg], webview_window: WebviewWindow
) -> None:
    channel: Channel[Msg] = body.channel_on(webview_window.as_ref_webview())

    # 👇 you should do this as background task, here just keep it simple as a example
    channel.send_model(Msg("message"))
