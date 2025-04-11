"""See: <https://setuptools-rust.readthedocs.io/en/latest/setuppy_tutorial.html>"""

from collections.abc import Sequence
from importlib.util import cache_from_source
from logging import getLogger
from os import getenv
from os.path import abspath, normcase
from pathlib import Path

from Cython.Build import (  # pyright: ignore[reportMissingTypeStubs]
    cythonize,  # pyright: ignore[reportUnknownVariableType]
)
from setuptools import setup
from setuptools.command.install import install as _install
from setuptools_rust import RustExtension

########## Inputs ##########

PYTAURI_STANDALONE = getenv("PYTAURI_STANDALONE") == "1"
"""Instead of building pytauri as a extension module file, it will be loaded in memory through Rust's `append_ext_mod`"""

USE_CYTHON = getenv("USE_CYTHON") == "1"
"""Whether to use Cython to compile the Python code into C extension module to protect the source code."""


SRC_PREFIX = "python/"
# The glob pattern of the source files you want to protect,
# or you can set it to `"tauri_app/**/*.py"` to protect all Python files in the project.
INCLUDE_FILE_PATTERNS = ("tauri_app/private.py",)
# Usually we dont need to protect the `**/__init__.py` files.
# NOTE: you must exclude `**/__main__.py`: <https://groups.google.com/g/cython-users/c/V-i0a8r-x00>.
EXCLUDE_FILE_PATTERNS = ("tauri_app/**/__init__.py", "tauri_app/**/__main__.py")

##############################


_logger = getLogger(__name__)


class install(_install):  # noqa: N801
    """Subclass `setuptools.command.install` to exclude protected files in the Wheel.

    ref: <https://setuptools.pypa.io/en/latest/userguide/extension.html>
    """

    def run(self) -> None:
        """Remove protected files after installation and before writing into the Wheel,
        to prevent them from being packaged into the Wheel."""

        super().run()  # pyright: ignore[reportUnknownMemberType]

        # skip if `pip install -e`
        build_py_obj = self.distribution.get_command_obj("build_py")
        build_py_obj.ensure_finalized()
        if build_py_obj.editable_mode:
            return

        # ref: <https://github.com/pypa/setuptools/blob/6ead555c5fb29bc57fe6105b1bffc163f56fd558/setuptools/_distutils/command/install_lib.py#L115-L124>
        assert self.install_lib is not None
        install_lib = Path(self.install_lib)

        def norm_files_set(patterns: Sequence[str]) -> set[str]:
            """Normalized set of file paths"""
            files_set: set[str] = set()
            for pattern in patterns:
                for file in install_lib.glob(pattern):
                    files_set.add(normcase(abspath(file)))
            return files_set

        include_files_set = norm_files_set(INCLUDE_FILE_PATTERNS)
        exclude_files_set = norm_files_set(EXCLUDE_FILE_PATTERNS)

        for file in include_files_set.difference(exclude_files_set):
            protected_file = Path(file)
            _logger.info(f"Removing protected file: {protected_file}")

            # remove the protected files from the Wheel
            protected_file.unlink()
            # remove C files generated by Cython
            protected_file.with_suffix(".c").unlink(missing_ok=True)
            # why `.cpp`: <https://cython.readthedocs.io/en/latest/src/userguide/wrapping_CPlusPlus.html#specify-c-language-in-setup-py>
            protected_file.with_suffix(".cpp").unlink(missing_ok=True)
            # remove *potential* python bytecode files (e.g., `pip install --compile-bytecode`)
            # ref:
            #   - <https://docs.python.org/3/using/cmdline.html#cmdoption-O>
            #   - <https://peps.python.org/pep-0488/>
            #   - <https://docs.python.org/3/library/importlib.html#importlib.util.cache_from_source>
            for optimization in ("", 1, 2):
                bytecode_file = cache_from_source(
                    protected_file, optimization=optimization
                )
                Path(bytecode_file).unlink(missing_ok=True)


setup(
    ####################
    # pyo3 extension module
    ####################
    rust_extensions=[
        RustExtension(
            # set `target` the same as `[project.entry-points.pytauri.ext_mod]` in `pyproject.toml`
            target="tauri_app.ext_mod",
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
    ####################
    # Cython
    ####################
    cmdclass={"install": install} if USE_CYTHON else {},
    # See: <https://cython.readthedocs.io/en/latest/src/quickstart/build.html#building-a-cython-module-using-setuptools>
    ext_modules=cythonize(  # pyright: ignore[reportUnknownArgumentType]
        module_list=[SRC_PREFIX + pattern for pattern in INCLUDE_FILE_PATTERNS],
        exclude=[SRC_PREFIX + pattern for pattern in EXCLUDE_FILE_PATTERNS],
    )
    if USE_CYTHON
    else [],
)
