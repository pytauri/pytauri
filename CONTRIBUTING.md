<!-- The content of this file is also included in `docs/CONTRIBUTING/CONTRIBUTING.md` via `pymdownx.snippets` -->
<!-- Do not use any **relative links** or **GitHub-specific syntax** !!! -->
<!-- Do not rename or move this file -->


# Contributing

Contributions are welcome, and they are greatly appreciated! Every little bit helps, and credit will always be given.

## Environment setup

Ensure that you have installed `Python`, `uv`, `Node.js`, `pnpm`, `git`, `tauri-cli` and the rest of [Tauri's Prerequisites](<https://tauri.app/start/prerequisites/>).

> Note: The setup script is written for Linux. If you're using Windows, consider running it in a Bash-compatible terminal via [MSYS2](<https://www.msys2.org/>).

1. [Fork the pytauri repository on GitHub.](<https://github.com/pytauri/pytauri/fork>)
2. paste your repo's URL in the script and run it:
```bash
#!/bin/bash

# clone your fork locally
git clone <repo-url>
cd pytauri

# Install dev dependencies and build the frontend assets
pnpm install
pnpm -r run build

# Activate the virtual environment
uv venv --python-preference=only-system
source .venv/bin/activate
# On Windows: source .venv/Scripts/activate

# Install dev dependencies and tools
uv sync

# Initialize pre-commit hooks (installed by `uv sync`)
# https://pre-commit.com/#3-install-the-git-hook-scripts
pre-commit install
pre-commit run --all-files

echo "---------------------------------"
echo "Setup is complete. Please create a local branch if you're contributing to the project!"
echo 'git checkout -b branch-name'
```

That's all! Now, you can start developing.

## IDE setup

We strongly recommend using **VS Code** along with the extensions listed in `.vscode/extensions.json`.

These extensions will help you format, lint, type-check, and debug your code.

### Debugging

TODO

- check `.vscode/launch.json` and [codelldb](<https://github.com/vadimcn/codelldb/blob/v1.11.5/MANUAL.md#remote-debugging>) for debugging `py/rs` from python.
- check [vscode/python-debugging](https://code.visualstudio.com/docs/python/debugging#_debugging-by-attaching-over-a-network-connection) for debugging `py/rs` from rust.

## Source code

- **python**: members in `/pyproject.toml`
- **rust**: menbers in `/Cargo.toml`
- **frontend**: members in `/package.json`

## Testing

We use [pytest](<https://docs.pytest.org/en/stable/>) and `cargo test` to run our tests.

## Documentation

### Python and Tutorial

We use [MkDocs](<https://www.mkdocs.org>), [Material for MkDocs](<https://squidfunk.github.io/mkdocs-material>), [mkdocstrings](<https://mkdocstrings.github.io>) and [mike](<https://github.com/jimporter/mike>) to build our documentation.

The documentation source files are located in `docs/`, `docs_src/`, `mkdocs.yml`, and `utils/`. (See `mkdocs.yml` for additional paths or custom configurations.).

#### Live-reloading the main documentation site:

```bash
mkdocs serve  # --dirty # 👈 optional to speed up hot-reload
```

#### Live-reloading versioned docs (with mike):

```bash
mike serve
```

!!! tip "Docs references"
    - [mkdocs/getting-started](https://www.mkdocs.org/getting-started/)
    - [mkdocs-material/getting-started](https://squidfunk.github.io/mkdocs-material/getting-started/)
    - [mkdocstrings/usage](https://mkdocstrings.github.io/python/usage/)

!!! tip
    We use `Google` style to write python docstrings, please refer to:

    - [mkdocstrings-python's documentation](https://mkdocstrings.github.io/python/usage/docstrings/google/)
    - [Napoleon's documentation](https://sphinxcontrib-napoleon.readthedocs.io/en/latest/example_google.html)
    - [Griffe's documentation](https://mkdocstrings.github.io/griffe/docstrings/)

### Rust

```bash
cargo doc
```

### Frontend

TODO

## PR

- PRs should target the `main` branch.
- Keep branches up to date by `rebase` before merging.
- Do not add multiple unrelated things in same PR.
- Do not submit PRs where you just take existing lines and reformat them without changing what they do.
- Do not change other parts of the code that are not yours for formatting reasons.
- Do not use your clone's main branch to make a PR - create a branch and PR that.

### Edit `CHANGELOG.md`

If you have made the corresponding changes, please record them in `CHANGELOG.md`.

### Commit message convention

Commit messages must follow the [Conventional Commits specification](https://www.conventionalcommits.org/en/v1.0.0/),
otherwise `pre-commit` will reject your commit.

!!! info
    Not sure how to follow all the contribution rules? No worries — feel free to open a PR, even if it's not perfect.
    We'll be happy to guide you and help you finish it.

## CI checks

Commits are checked via **GitHub Actions**, and pull requests can only be merged if all CI checks pass.

You can run most of these checks locally by executing:

```bash
pre-commit run --all-files
```

This works as long as you've installed the Git hooks via `pre-commit install`.
Usually, you don’t need to run checks manually — `pre-commit` runs them automatically on each commit.

!!! tip
    Some slower checks are skipped during normal local runs.
    If you want to run all checks, including the slow ones, use:

    ```bash
    pre-commit run --all-files --hook-stage=manual
    ```

    You can also check `.pre-commit-config.yaml` and run specific hooks individually if you prefer.


---

## 😢

!!! warning
    The section below is mainly for project maintainers. You probably don’t need to read it.

---

## Deploying The Documentation

The documentations are automatically deployed through **GitHub's workflows**, please refer to `.github/workflows/docs.yml`.

- Pushing to the `main` branch automatically deploys the **`dev` version** of the documentation.
- Pushing a semantic version tag (e.g. `v1.2.3`) triggers deployment for that **specific version** of the docs.

!!! warning
    - Remember to update `CHANGELOG.md` before pushing the version docs tag!
    - Remember to make a Github Release (not package release) manually for the version docs deployment!

## PR Checks

please refer to `.github/workflows/lint-test.yml`.

- Every PR push will trigger the CI checks.

## Publish and Release 🚀

Please refer to `.github/workflows/publish-*.yml`.

- Pushing a semantic version tag in the format `py|rs|js/package-name/v*` will trigger publishing for the corresponding package.

### 1. Prepare the release

- Check out a **new branch**.
- Update `CHANGELOG.md` with the changes.
- Bump the version accordingly.

!!! warning
    Don’t forget to update dependency versions for **workspace members** as needed.

### 2. Create the release tag

- Push the **new branch** along with a **signed tag** to GitHub.
- Open a pull request (PR) from that branch to the `main` branch.

> 🔐 The tag **must be signed**!

!!! warning
    The **version bump PR** must contain **only one commit** — the one with the version tag.
    PRs with multiple commits will be rejected.

### 3. Finalize the release

- Review the PR carefully.
- If everything looks good, **rebase and merge** it into the `main` branch **locally**.


!!! warning "**DO NOT rebase with a tag on GitHub.**"

    Refer to:

    > <https://docs.github.com/authentication/managing-commit-signature-verification/about-commit-signature-verification#signature-verification-for-rebase-and-merge>
    >
    > When you use this option, GitHub creates modified commits using the original commit data and content.

    This will cause the commits merged into `main` to be inconsistent with the tagged commits.

    If this happens, you must delete the tag and re-tag the merged commit.

---

Before approving the release, please verify the following:

- **The tag exists on the `main` branch.**
- **The version specified by the tag is correct.**
- Dependency versions for workspace members are updated.
- Links in `CHANGELOG.md` are accurate.

If everything looks good, approve the release in the environment (`pypi` / `crates-io` / `npmjs`) for the workflow.

The `publish-*.yml` workflow will then build and publish the package automatically.

Finally, edit the draft release created by the workflow and publish it.
