# PyTauri + Vanilla TS

```bash
cd examples/tauri-app
```

and follow the tutorial section in Documentation.

---

or you are hurry, just wanna see/run the demo:

> Make sure you have already installed `rust`, `uv`, `pnpm`, `node.js` and Tauri Prerequisites.
>
> You can find that in tutorial section in Documentation.

```bash
git clone https://github.com/pytauri/pytauri.git
cd pytauri

cd examples/tauri-app

# build frontend assets
pnpm install
pnpm -r run build

# activate virtual environment
uv venv
source .venv/bin/activate
# or powershell: .venv\Scripts\Activate.ps1

# install the example package
# (need some time to compile rust code,
#  you can pass `--verbose` to see the progress)
uv pip install --reinstall -e src-tauri

# run the example
python -m tauri_app
```

![demo](https://github.com/user-attachments/assets/14ad5b51-b333-4d80-b04b-af72c4179571)
