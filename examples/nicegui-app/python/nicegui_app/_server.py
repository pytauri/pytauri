from socket import socket
from threading import Event
from typing import Optional

import uvicorn
from typing_extensions import override

__all__ = ["FrontServer"]


class FrontServer(uvicorn.Server):
    """Subclass `uvicorn.Server` to allow listening for startup and shutdown events."""

    def __init__(self, config: uvicorn.Config) -> None:
        super().__init__(config)
        # NOTE: use `threading.Event` instead of `anyio.Event` for cross-thread communication
        self._startup_event = Event()
        self._shutdown_event = Event()
        self._serve_exception: Optional[Exception] = None

    @property
    def serve_exception(self) -> Optional[Exception]:
        """The exception raised during serving the application."""
        return self._serve_exception

    @override
    async def startup(self, sockets: Optional[list[socket]] = None) -> None:
        """Set the startup event after the server is started."""
        await super().startup(sockets)
        self._startup_event.set()

    @override
    async def serve(self, sockets: list[socket] | None = None) -> None:
        """The main entry point to serve the application."""
        try:
            await super().serve(sockets)
        except Exception as exc:
            self._serve_exception = exc
            raise
        finally:
            # set all the events whatever in case of exception
            self._startup_event.set()
            self._shutdown_event.set()

    def wait_for_startup(self) -> None:
        """Block until the server is started."""
        self._startup_event.wait()

    def request_shutdown(self) -> None:
        """Request and block to wait for the server to shutdown."""
        # Ref:
        # - <https://github.com/zauberzeug/nicegui/discussions/1957#discussioncomment-7484548>
        # - <https://github.com/encode/uvicorn/discussions/1103#discussioncomment-6187606>
        self.should_exit = True

        self._shutdown_event.wait()
