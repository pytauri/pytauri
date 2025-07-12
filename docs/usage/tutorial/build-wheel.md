# Build Python Wheel distribution

Although you have built the sdist in the previous step and can build the wheel using `setuptools-rust`, take a look at the output wheel file name `tauri-app-0.1.0-cp39-cp39-linux_x86_64.whl`.

According to <https://packaging.python.org/en/latest/specifications/platform-compatibility-tags/>, this indicates that the wheel can only be used on `CPython == 3.9`.

This means that if you want to distribute the wheel built in this way, you need to build a wheel for each CPython version.

Additionally, PyPI will not allow you to upload wheels with the `linux*` tag because such a tag does not specify the libc version required to run the wheel. PyPI only allows [manylinux] and [musllinux].

[manylinux]: https://packaging.python.org/en/latest/specifications/platform-compatibility-tags/#manylinux
[musllinux]: https://packaging.python.org/en/latest/specifications/platform-compatibility-tags/#musllinux

!!! info
    This is not an issue for users who use your sdist, as they will build the wheel in their own environment and do not need to distribute that wheel.

## Maturin

To make it easier to build distributable wheels, we recommend using [maturin](https://github.com/PyO3/maturin), which is a `setuptools-rust` with batteries included.

First, install it (`v1.8.0`) using uv:

=== "windows or macos"
    ```bash
    uv pip install maturin
    ```

=== "linux"
    ```bash
    uv pip install maturin[patchelf]
    ```

Add the following configuration, which is the same as the configuration for `setuptools-rust`:

```toml title="src-tauri/pyproject.toml"
# see: <https://www.maturin.rs/config>
[tool.maturin]
# the same as [tool.setuptools.packages.find.where]
python-source = "python"
# the same as `[project.entry-points.pytauri.ext_mod]`,
# i.e., `target` in `setup.py`
module-name = "tauri_app.ext_mod"
# see `setup.py`
features = ["pyo3/extension-module", "tauri/custom-protocol"]
# equivalent to `setuptools_scm`
sdist-generator = "git"
# equivalent to `MANIFEST.in`
include = [{ path = "frontend/**/*", format = "sdist" }]
```

## Build manylinux wheel

ref: <https://www.maturin.rs/distribution#build-wheels>

Maturin can automatically detect the current system's glibc version and assign the appropriate tag to the built wheel. Use the following command to build a manylinux wheel:

```bash
cd src-tauri
pnpm build  # build frontend assets
maturin build --release  # `--strip` <-- optional to reduce the size
cd ..
```

When you build on `ubuntu 22.04 (glibc 2.35)`, you will get a wheel file named `*-manylinux_2_35_*.whl`. The `manylinux_2_35` tag indicates that the wheel can run on systems with `glibc >= 2.35`.

If you want to support as many systems as possible, you should build the wheel on an older system. However, please note that the dependencies of `tauri v2` require you to use `ubuntu 22+`.

## Bundle system dependencies with the wheel

According to [PEP513](https://peps.python.org/pep-0513/), the manylinux wheel you built in the previous step can only link to a limited set of system libraries at runtime. To meet this requirement, maturin will copy and bundle these system libraries (including tauri's dependencies) during the build process, similar to how [AppImage](https://appimage.org/) works.

!!! tip
    Not all linked libraries will be bundled. Some libraries commonly found in various Linux distributions will be whitelisted. See more:

    - <https://github.com/pypa/auditwheel/issues/78>
    - <https://github.com/kuelumbus/rdkit-pypi/issues/75>
    - <https://github.com/PyO3/maturin/blob/f5b807eaf3f576ea08e6a574d699fc6f54e2be46/src/auditwheel/manylinux-policy.json#L454>

Building the wheel in this way means your users will no longer need to manually install dependencies. However, note that **this will increase your wheel size from 10MB to 100MB**.

If you do not want this behavior, skip patching and manually specify the manylinux tag:

```bash
maturin build --release --auditwheel skip --manylinux 2_35 # <-- your glibc version
```

Then, require your users to install these dependencies before running the wheel:

ref: <https://tauri.app/distribute/debian/#debian>

- libwebkit2gtk-4.1-0
- libgtk-3-0
- libappindicator3-1 (if your app uses the system tray)

## Build abi3 wheel

ref: <https://www.maturin.rs/bindings.html#py_limited_apiabi3>

add pyo3 feature `pyo3/abi3-py3*`:

```toml title="src-tauri/pyproject.toml"
[tool.maturin]
# ...
features = ["pyo3/extension-module", "tauri/custom-protocol", "pyo3/abi3-py39"]
```

then build your wheel:

```bash
maturin build --release  # your maturin args
```

you will get a wheel file named `*-cp39-abi3-*.whl`, which means that the wheel can run on `CPython >= 3.9`.

!!! info
    The `pytauri/standalone` feature is incompatible with the `pyo3/abi3` feature, which is why we only enable it in `[[bin]]` target.

## What's `project.entry-points.pytauri` mean?

Now it's time to explain [`[project.entry-points.pytauri]`](using-pytauri.md/#init-pyproject).

Looking at the contents of the `.whl`, you will see the following structure:

```tree
└── tauri_app-0.1.0-*.whl
    ├── tauri_app
    │   ├── __init__.py
    │   ├── __main__.py
    │   └── ext_mod.*.so/pyd
    └── ...
```

We indicate the extension module `mod ext_mod` in `lib.rs` to be compiled into the `tauri_app/ext_mod.*.so/pyd` file by:

- `setup.py (setuptools-rust)`: `target="tauri_app.ext_mod"`
- `pyproject.toml (maturin)`: `module-name = "tauri_app.ext_mod"`

!!! Warning
    `pytauri` does not have any opinion on where you place the extension module, but note that in `lib.rs` we specified the extension module name with `#[pyo3(name = "ext_mod")]`, so your extension module file name must match this name.

Finally, we tell pytauri how to find it through `project.entry-points.pytauri.ext_mod = "tauri_app.ext_mod"` in `pyproject.toml`.
