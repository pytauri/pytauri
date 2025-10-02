<!-- The content will be also use in `docs/index.md` by `pymdownx.snippets` -->
<!-- Do not use any **relative link** and  **GitHub-specific syntax** ！-->
<!-- Do not rename or move the file -->

# PyTauri

![banner.png](https://raw.githubusercontent.com/pytauri/branding/6832e0defd4220b8a3f5c1f111bd164cee616bbe/assets/banner.png)

[Tauri] bindings for Python through [Pyo3]

[Tauri]: https://github.com/tauri-apps/tauri
[Pyo3]: https://github.com/PyO3/pyo3

---

[![CI: docs]][CI: docs#link] [![msrv]][msrv#link] [![requires-python]][requires-python#link] [![Discord]][Discord#link]

Documentation: <https://pytauri.github.io/pytauri/>

Source Code: <https://github.com/pytauri/pytauri/>

[CI: docs]: https://github.com/pytauri/pytauri/actions/workflows/docs.yml/badge.svg
[CI: docs#link]: https://github.com/pytauri/pytauri/actions/workflows/docs.yml
[Discord]: https://img.shields.io/discord/1411349756202188942?logo=discord&label=discord
[Discord#link]: https://discord.gg/TaXhVp7Shw
[msrv]: https://img.shields.io/crates/msrv/pytauri?logo=rust
[msrv#link]: https://rust-lang.github.io/rfcs/2495-min-rust-version.html
<!--
TODO: Switch to <https://shields.io/badges/py-pi-python-version>,
but need to add python version classifiers in `pyproject.toml` first.
See also <https://github.com/badges/shields/issues/5551>.
-->
[requires-python]: https://img.shields.io/python/required-version-toml?tomlFilePath=https%3A%2F%2Fraw.githubusercontent.com%2Fpytauri%2Fpytauri%2Frefs%2Fheads%2Fmain%2Fpython%2Fpytauri%2Fpyproject.toml&logo=python
[requires-python#link]: https://packaging.python.org/en/latest/specifications/core-metadata/#requires-python

---

This is a completely free and open-source project, but it is difficult to maintain without incentives and contributions from the community.

If you think this project is helpful, consider giving it a star [![GitHub Repo stars]][Github Repo], it would be very helpful for my work and studies. 🥺👉👈

[GitHub Repo stars]: https://img.shields.io/github/stars/pytauri/pytauri?style=social
[Github Repo]: https://github.com/pytauri/pytauri

---

## Features

> If you're in a hurry, you can find the demos in [examples/tauri-app](https://github.com/pytauri/pytauri/tree/main/examples/tauri-app).

[notification]: https://docs.rs/tauri-plugin-notification/latest/tauri_plugin_notification/

- **You’ll hardly need to write any Rust code**! (rust compiler is still required)
- Alternatively, use `pytauri-wheel`; **no Rust compiler needed, and everything can be done in Python!** (See [examples/tauri-app-wheel](https://github.com/pytauri/pytauri/tree/main/examples/tauri-app-wheel).)
- Easily integrates with `tauri-cli` to build and package standalone executables!
    - Use `Cython` to protect your source code!
- No inter-process communication (IPC) overhead, making it secure and fast thanks to [Pyo3]!
- Support Tauri official plugins(e.g., [notification]), and you can write your own plugins!

    ![demo](https://github.com/user-attachments/assets/14ad5b51-b333-4d80-b04b-af72c4179571)

- Native support for async Python (`asyncio`, `trio` and `anyio`).
- **100%** [Type Completeness](https://microsoft.github.io/pyright/#/typed-libraries?id=type-completeness).
- Ergonomic API that is as close as possible to the Tauri Rust API.
    - [Automatically generated TypeScript types and client for IPC](https://github.com/pytauri/pytauri/tree/main/examples/tauri-app/src/client)
    - Python

        ```python
        import sys

        from pydantic import BaseModel
        from pytauri import (
            AppHandle,
            Commands,
        )
        from pytauri_plugins.notification import NotificationExt

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
        ```

    - Frontend

        ```ts
        import { pyInvoke } from "tauri-plugin-pytauri-api";
        // or: `const { pyInvoke } = window.__TAURI__.pytauri;`

        export interface Person {
            name: string;
        }

        export interface Greeting {
            message: string;
        }

        export async function greet(body: Person): Promise<Greeting> {
            return await pyInvoke("greet", body);
        }
        ```

- Can be integrated with [NiceGUI], [Gradio] and [FastAPI] to achieve a full-stack Python development experience (i.e., without `Node.js`). See [examples/nicegui-app](https://github.com/pytauri/pytauri/tree/main/examples/nicegui-app).

## Release

We follow [Semantic Versioning 2.0.0](https://semver.org/).

Rust and its Python bindings, PyTauri core and its plugins will maintain the same `MAJOR.MINOR` version number.

| name | pypi | crates.io | npmjs |
|:-------:|:----:|:---------:|:-----:|
| 👉 **core** | - | - | - |
| pytauri | [![pytauri-pypi-v]][pytauri-pypi] | [![pytauri-crates-v]][pytauri-crates] | |
| pytauri-core | | [![pytauri-core-crates-v]][pytauri-core-crates] | |
| tauri-plugin-pytauri | | [![tauri-plugin-pytauri-crates-v]][tauri-plugin-pytauri-crates] | [![tauri-plugin-pytauri-api-npm-v]][tauri-plugin-pytauri-api-npm] |
| 👉 **wheel** | - | - | - |
| pytauri-wheel | [![pytauri-wheel-pypi-v]][pytauri-wheel-pypi] | | |
| 👉 **utils** | - | - | - |
| pyo3-utils | [![pyo3-utils-pypi-v]][pyo3-utils-pypi] | [![pyo3-utils-crates-v]][pyo3-utils-crates] | |
| codelldb | [![codelldb-pypi-v]][codelldb-pypi] | | |

[pytauri-pypi-v]: https://img.shields.io/pypi/v/pytauri
[pytauri-pypi]: https://pypi.org/project/pytauri
[pytauri-crates-v]: https://img.shields.io/crates/v/pytauri
[pytauri-crates]: https://crates.io/crates/pytauri
[pytauri-core-crates-v]: https://img.shields.io/crates/v/pytauri-core
[pytauri-core-crates]: https://crates.io/crates/pytauri-core
[pytauri-wheel-pypi-v]: https://img.shields.io/pypi/v/pytauri-wheel
[pytauri-wheel-pypi]: https://pypi.org/project/pytauri-wheel
[tauri-plugin-pytauri-crates-v]: https://img.shields.io/crates/v/tauri-plugin-pytauri
[tauri-plugin-pytauri-crates]: https://crates.io/crates/tauri-plugin-pytauri
[tauri-plugin-pytauri-api-npm-v]:https://img.shields.io/npm/v/tauri-plugin-pytauri-api
[tauri-plugin-pytauri-api-npm]: https://www.npmjs.com/package/tauri-plugin-pytauri-api
[pyo3-utils-pypi-v]: https://img.shields.io/pypi/v/pyo3-utils
[pyo3-utils-pypi]: https://pypi.org/project/pyo3-utils
[pyo3-utils-crates-v]: https://img.shields.io/crates/v/pyo3-utils
[pyo3-utils-crates]: https://crates.io/crates/pyo3-utils
[codelldb-pypi-v]: https://img.shields.io/pypi/v/codelldb
[codelldb-pypi]: https://pypi.org/project/codelldb

## Philosophy

### For Pythoneer

I hope `PyTauri` can become a viable alternative to [pywebview] and [Pystray], leveraging Tauri's comprehensive features to provide Python developers with a GUI framework and a batteries-included development experience similar to [electron] and [PySide].

> PyTauri is inspired by [FastAPI] and [Pydantic], aiming to offer a similar development experience.

### For Rustacean

Through [Pyo3], I hope Rust developers can better utilize the Python ecosystem, for example by building AI GUI applications with [PyTorch].

Although Rust's lifetime and ownership system makes it safer, Python's garbage collection (GC) will make life easier. 😆

[pywebview]: https://github.com/r0x0r/pywebview
[Pystray]: https://github.com/moses-palmer/pystray
[electron]: https://github.com/electron/electron
[PySide]: https://wiki.qt.io/Qt_for_Python
[FastAPI]: https://github.com/fastapi/fastapi
[Pydantic]: https://github.com/pydantic/pydantic
[PyTorch]: https://github.com/pytorch/pytorch
[NiceGUI]: https://github.com/zauberzeug/nicegui
[Gradio]: https://github.com/gradio-app/gradio

## Used By

Although PyTauri is a fairly young project, a few people have already used it to build some cool projects:

- [Digger Solo](https://solo.digger.lol/) - AI powered file manager

## Credits

PyTauri is a project that aims to provide Python bindings for [Tauri], a cross-platform webview GUI library. `Tauri` is a trademark of the Tauri Program within the Commons Conservancy, and PyTauri is neither officially endorsed nor supported by them. PyTauri is an independent, and community-driven effort that respects the original goals and values of Tauri. PyTauri does not claim any ownership of or affiliation with the Tauri Program.

## License

This project is licensed under the terms of the *Apache License 2.0*.
