# PyTauri-Wheel+ Vanilla TS

`pytauri-wheel` provides precompiled dynamic libraries, so you no longer need a Rust compiler. You can run Tauri applications with just Python.

---

```bash
git clone https://github.com/pytauri/pytauri.git
cd pytauri

# build frontend assets
pnpm install
pnpm -r run build

# activate virtual environment
uv venv
source .venv/bin/activate
# or powershell: .venv\Scripts\Activate.ps1

# This step will compile `pytauri-wheel` locally (requires Rust compiler),
# or you can download the precompiled `pytauri-wheel` from PyPi.
uv pip install --reinstall -e python/pytauri-wheel

cd examples/tauri-app-wheel

# install the example package
uv pip install --reinstall -e ./python
```

## Run in Development Mode

```bash
pnpm dev  # launch Vite dev server
```

then in another terminal:

```bash
# Set environment variable to tell `tauri_app_wheel` to
# use vite dev server as the frontend dist,
# see `python/src/tauri_app_wheel/__init__.py` for details
export TAURI_APP_WHEEL_DEV=1
# or in powershell: $env:TAURI_APP_WHEEL_DEV=1

source .venv/bin/activate
# or powershell: .venv\Scripts\Activate.ps1

cd examples/tauri-app-wheel

python -m tauri_app_wheel
```

## Run in Production Mode

```bash
pnpm build  # build frontend assets
python -m tauri_app_wheel
```

## Build SDist and Wheel

```bash
pnpm build  # build frontend assets
uv build ./python
```
