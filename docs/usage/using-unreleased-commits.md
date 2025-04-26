# Using Unreleased Commits

Since `v0.5`, all `pytauri` packages support installation from a Git repository via branch, commit `SHA`, or PR (pull request).

## Install Rust crate from source

> ref: <https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html#the-patch-section>

Append this to your `Cargo.toml` file:

```toml
[patch.crates-io]
pytauri = { git = "https://github.com/pytauri/pytauri.git", branch = "main" }
pytauri-core = { git = "https://github.com/pytauri/pytauri.git", branch = "main" }
tauri-plugin-pytauri = { git = "https://github.com/pytauri/pytauri.git", branch = "main" }
# other pytauri dependencies which you need ...
```

This will force all your dependencies to use `pytauri` from Git instead of `crates.io`.

## Install Python package from source

> ref: <https://docs.astral.sh/uv/concepts/projects/dependencies/#dependency-sources>

Append this to your `pyproject.toml` file:

```toml
[tool.uv.sources]
pytauri = { git = 'https://github.com/pytauri/pytauri.git', branch = "main", subdirectory = "python/pytauri" }
# other pytauri dependencies which you need ...
```

!!! tip
    You can check the `[tool.uv.workspace]` section in [pyproject.toml] to find the `subdirectory` for each package.

    [pyproject.toml]: https://github.com/pytauri/pytauri/blob/main/pyproject.toml

## Install JS package from source

> Inspired by: <https://vite.dev/guide/#using-unreleased-commits>

Thanks to <https://pkg.pr.new/>, you can install JS package from specific branch, commit `SHA`, or PR with:

```bash
# or pnpm, yarn, bun, whatever
npm i https://pkg.pr.new/tauri-plugin-pytauri-api@main
```

!!! tip
    To replace the pytauri version used by dependencies transitively, you should use [npm overrides] or [pnpm overrides].

    [npm overrides]: https://docs.npmjs.com/cli/v11/configuring-npm/package-json#overrides
    [pnpm overrides]: https://pnpm.io/settings#overrides
