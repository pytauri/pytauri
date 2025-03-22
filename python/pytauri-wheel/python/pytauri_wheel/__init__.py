# DO NOT import any `pytauri` API here,
# Otherwise, in **user** code, if `pytauri` is imported first and then `pytauri_wheel`,
# it will cause a circular import issue:
# `user` -> `pytauri` -> `pytauri_wheel.ext_mod` -> `pytauri.__init__` -> `pytauri` -> `pytauri.__init__` -> ...

"""PyTauri precompiled wheels.

Due to the limitations of circular imports, we cannot import the `pytauri` module in `__init__.py`,
so the related APIs are placed in the [pytauri_wheel.lib][] module.
"""
