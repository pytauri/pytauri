# Prerequisites

!!! info "For Pythoneer"
    For pure Python (i.e., without Rust) development, you might prefer to use [pytauri-wheel](../pytauri-wheel/index.md) ✨.

---

## Platforms

- Tier 1: my(author's) primary development environment
    - Windows 10
- Tier 2: will got bugs fixed and tested on ci
    - Linux (WSL2)
- Tier 3: will not be tested, may not work
    - MacOS
    - Windows 7

!!! info
    I do not have MacOS, so I cannot debug issues on MacOS. That is why it is in Tier 3.

    If you would like better support for MacOS, please consider becoming a maintainer.

## Python

We support all Python versions that are currently in their lifecycle[^requires-python]. The current version is `>= 3.9`.

<https://www.python.org/downloads/>

!!! warning
    For linux development, [you need to ensure that your Python installation contains a shared library](https://pyo3.rs/v0.23.3/index.html#using-python-from-rust).

    To install the Python shared library on Ubuntu: `#!bash sudo apt install python3-dev`.

    To install the Python shared library on RPM based distributions (e.g. Fedora, Red Hat, SuSE), install the `python3-devel` package.

[^requires-python]:
    We typically (though sometimes earlier) drop support for a python version in the next pytauri minor release after it reaches end-of-life.

## Rust

The *latest*[^msrv] stable version.

<https://www.rust-lang.org/tools/install>

!!! tip "for python developers"
    You can upgrade your existing Rust version by running `#!bash rustup update stable`.

[^msrv]:
    We may bump the [MSRV](https://rust-lang.github.io/rfcs/2495-min-rust-version.html) in minor/patch releases.

## System Dependencies

- [Tauri Prerequisites](https://tauri.app/start/prerequisites/#system-dependencies)
