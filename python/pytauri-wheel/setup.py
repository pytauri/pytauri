"""See: <https://setuptools-rust.readthedocs.io/en/latest/setuppy_tutorial.html>"""

from os import getenv

from packaging.tags import sys_tags
from setuptools import setup
from setuptools_rust import RustExtension

PYTAURI_STANDALONE = getenv("PYTAURI_STANDALONE") == "1"
"""Instead of building pytauri as a extension module file, it will be loaded in memory through Rust's `append_ext_mod`"""

setup(
    rust_extensions=[
        RustExtension(
            # set `target` the same as `[project.entry-points.pytauri.ext_mod]` in `pyproject.toml`
            target="pytauri_wheel.ext_mod",
            # It is recommended to set other features in `Cargo.toml`, except following features:
            features=[
                # see: <https://pyo3.rs/v0.23.3/building-and-distribution.html#the-extension-module-feature>,
                # required to build the extension module
                "pyo3/extension-module",
                # This feature tells Tauri to use embedded frontend assets instead of using a frontend development server.
                # Usually this feature is enabled by `tauri-cli`, here we enable it manually.
                "tauri/custom-protocol",
            ],
        )
    ]
    if not PYTAURI_STANDALONE
    else [],
    # ref:
    # - <https://stackoverflow.com/a/75010995>
    # - <https://github.com/pypa/setuptools/blob/e7c42a0efab982c355667f7cf7ced3bc72f3c7c7/setuptools/command/bdist_wheel.py#L146-L150>
    # - <https://setuptools-rust.readthedocs.io/en/v1.11.0/building_wheels.html#building-for-abi3>
    options={
        "bdist_wheel": {
            # `next(sys_tags()).platform` is usually `manylinux_x_y_{arch}`.
            #
            # `setuptools` usually only assigns `linux_{arch}`, so we need to specify it manually.
            # Although `pytauri-wheel` does not actually comply with the manylinux policy (it link to system webkit lib),
            # we will document these system libs in the README, so it's not a problem.
            "plat_name": next(sys_tags()).platform,
        },
    },
)
