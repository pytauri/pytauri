# NOTE: DO NOT use third-party libraries in this file,
# keep the minimal dependencies.

"""Launch [CodeLLDB] to debug rust code.

[CodeLLDB]: https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb

# Usage

!!! tip

    This package only depends on the Python standard library, so you can integrate it freely.

## Configuring CodeLLDB rpc server

Please refer to the CodeLLDB documentation to set up the [rpc server](https://github.com/vadimcn/codelldb/blob/master/MANUAL.md#rpc-server),
and add the following content to `.vscode/settings.json`:

```json
{
  "lldb.rpcServer": {
    "host": "localhost",
    "port": 9552,
    "token": "secret",
  }
}
```

## Launching the CodeLLDB debugger from Python

```python
from codelldb import debug

if True:  # Replace with your condition to debug
    debug(
        host="localhost",
        port=9552,
        token="secret",
    )
```
"""

import json
import socket
from os import getpid
from textwrap import dedent
from typing import Optional

__all__ = ["debug"]


class DebugError(Exception):
    pass


def debug(host: str, port: int, token: Optional[str] = None) -> None:
    """Launch CodeLLDB to debug rust code.

    Raises:
        DebugError: Failed to launch CodeLLDB.
    """
    token_data = f"token: {token}" if token else ""
    # See: <https://github.com/vadimcn/codelldb/blob/v1.10.0/MANUAL.md#rpc-server>
    # Line-oriented YAML Syntax: <https://github.com/vadimcn/codelldb/blob/v1.10.0/MANUAL.md#debugging-externally-launched-code>
    # Arg: <https://github.com/vadimcn/codelldb/blob/v1.10.0/MANUAL.md#attaching-to-a-running-process>
    rpc_data = dedent(f"""\
        name: "CodeLLDB: Attach to Process"
        type: "lldb"
        request: "attach"
        pid: {getpid()}
        sourceLanguages:
            - rust
            - c
            - cpp
        {token_data}
    """)

    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.connect((host, port))

        s.sendall(rpc_data.encode("utf-8"))

        s.shutdown(socket.SHUT_WR)

        response = s.recv(1024)

        if not response:
            raise DebugError(
                "Failed to get response from lldb rpc server, "
                "maybe the rpc `token` is not correct."
            )

        try:
            response = json.loads(response)
            assert isinstance(response, dict)
        except Exception as e:
            raise DebugError(
                f"Failed to parse response from lldb rpc server: {response}"
            ) from e

        if response.get("success") is not True:  # pyright: ignore[reportUnknownMemberType]
            raise DebugError(
                f"Seems like lldb rpc server failed to attach to the process: {response}"
            )
