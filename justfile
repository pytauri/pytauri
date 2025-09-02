# TODO, FIXME: UV script feature will replace this justfile once implemented,
# see <https://github.com/astral-sh/uv/issues/5903>.


set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

default:
    @just --list


python-fmt:
    ruff check . --fix
    ruff format .

python-test:
    pytest \
        python/pyfuture/tests/ \
        --cov --cov-report=xml --cov-report=html

python-verifytypes:
    pnpm pyright --verifytypes codelldb --ignoreexternal
    pnpm pyright --verifytypes pyfuture --ignoreexternal
    pnpm pyright --verifytypes pyo3_utils --ignoreexternal
    pnpm pyright --verifytypes pytauri --ignoreexternal
    pnpm pyright --verifytypes pytauri_plugins --ignoreexternal
    pnpm pyright --verifytypes pytauri_utils --ignoreexternal
    pnpm pyright --verifytypes pytauri_wheel --ignoreexternal
