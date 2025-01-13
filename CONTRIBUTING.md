<!-- The content will be also use in `docs/CONTRIBUTING/CONTRIBUTING.md` by `pymdownx.snippets` -->
<!-- Do not use any **relative link** and  **GitHub-specific syntax** ！-->
<!-- Do not rename or move the file -->

# Contributing

Contributions are welcome, and they are greatly appreciated! Every little bit helps, and credit will always be given.

## Environment setup

Make sure you have installed `Rust`, `Python`, `uv`, `Node.js`, `pnpm`, `tauri-cli` and Tauri Prerequisites as documented.

Also, you need `bash`. If you are on Windows, you can use [Git for Windows](https://gitforwindows.org/).

Fork the pytauri repository on GitHub.

```bash
#!/bin/bash

# clone your fork locally
git clone git@github.com:your_name_here/pytauri.git
cd pytauri
# create a branch for local development
git checkout -b branch-name

# install dev dependencies and build frontend assets
pnpm install
pnpm -r run build

# activate virtual environment
uv venv --python-preference=only-system
source .venv/bin/activate
# or Windows: `source .venv/Scripts/activate`

# install dev dependencies and tools
uv sync

# Init pre-commit (installed by `uv sync`)
# https://pre-commit.com/#3-install-the-git-hook-scripts
pre-commit install
pre-commit run --all-files
```

That's all! Now, you can start to develop.

## IDE setup

We strongly recommend using `VSCode` with the extensions in `.vscode/extensions.json`.

These extensions will help you to format, lint, type-check, and debug your code.

### Debug

TODO

- check `.vscode/launch.json` and [codelldb][] for debugging `py/rs` from python.
- check [vscode/python-debugging](https://code.visualstudio.com/docs/python/debugging#_debugging-by-attaching-over-a-network-connection) for debugging `py/rs` from rust.

## Source code

- **python**: members in `/pyproject.toml`
- **rust**: menbers in `/Cargo.toml`
- **frontend**: members in `/package.json`

## Testing

We use [pytest](https://docs.pytest.org/en/stable/) and `cargo test` to test our code.

## Documentation

### Python and Toturial

We use [mkdocs](https://www.mkdocs.org), [mkdocs-material](https://squidfunk.github.io/mkdocs-material), [mkdocstrings](https://mkdocstrings.github.io) and [mike](https://github.com/jimporter/mike) to build our documentation.

The documentation source code is in `docs/`, `docs_src/`, `mkdocs.yml`, and `utils/` (check `mkdocs.yml` to find others).

Live-reloading main docs:

```bash
mkdocs serve
```

Live-reloading versioned docs:

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

Commit messages must follow [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/),
or `pre-commit` will reject your commit.

!!! info
    If you don't know how to finish these, it's okay, feel free to initiate a PR, we will help you continue.

## CI checks

We will check your commits on GitHub Actions, and your PR will only be merged if it passes the CI checks.

You can run these checks locally by executing `pre-commit run --all-files` in **bash**.

> Usually, you don't need to do this manually, because `pre-commit` will automatically run these checks on each commit as long as you have installed the git hooks via `pre-commit install`.

!!! tip
    Some slow checks are not run locally by default. If you really want to run them, pass `--hook-stage=manual`. You can also look at `.pre-commit-config.yaml` and run the individual checks yourself if you prefer.

---

## 😢

!!! warning
    The following 👇 content is for the maintainers of this project, may be you don't need to read it.

---

## Deploy Docs

please refer to `.github/workflows/docs.yml`.

- Every push to the `main` branch will trigger the `dev` version docs deployment.
- Every `v*` semver tag will trigger the corresponding version docs deployment.

    !!! note
        Remember make a Github Release (not package release) for the version docs deployment.

## PR Checks

please refer to `.github/workflows/lint-test.yml`.

- Every PR push will trigger the CI checks.

## Publish and Release 🚀

Please refer to `.github/workflows/publish-*.yml`.

- Every `py|rs|js/package-name/v*` semver tag will trigger the corresponding package publish.

First, check-out to a **new branch**, edit `CHANGELOG.md` to record the changes and bump the version.

!!! warning
    Remember also update the dependencies version for **workspace members**.

Then, push the **new branch** with the **signed tag** to GitHub, and create a PR to the `main` branch.

> Again, the tag must be **signed**!!!

!!! warning
    The `bump version` PR must have **only one commit with the corresponding tag**; otherwise, it will be rejected.

Review the PR, if it's ok, **rebase** it to `main` branch **in local**.

!!! warning "**DO NOT rebase with tag on GitHub.**"

    Refer to:

    > <https://docs.github.com/authentication/managing-commit-signature-verification/about-commit-signature-verification#signature-verification-for-rebase-and-merge>
    >
    > When you use this option, GitHub creates modified commits using the original commit data and content.

    This will cause the commits merged into main to be inconsistent with the tagged commits.

    If you unfortunately do this, you must delete the tag and re-tag the merged commit.

Check if everything is ok, for example:

- **check if the tag is on the `main` branch**.
- **check if the version specified by the tag is correct**.
- check if the dependencies version of workspace members are updated.
- check if the link in `CHANGELOG.md` is correct.

If so, make a `approve` in environment `pypi`/`crates-io`/`npmjs` for the workflow.

After that, the `publish-*.yml` workflow will build and publish the package.

Finally, edit the `draft release` created by `publish-*.yml` workflow, and publish the release.
