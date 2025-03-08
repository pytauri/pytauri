# Getting Started

Before starting the tutorial, we recommend installing the following tools, which are considered best practices for initializing a pytauri project. We will use these tools throughout the tutorial.

- [create-tauri-app](https://github.com/tauri-apps/create-tauri-app): `v4.5.9`
- [uv](https://github.com/astral-sh/uv): `v0.5.11`
- [tauri-cli](https://www.npmjs.com/package/@tauri-apps/cli): `v2.1.0`

!!! note
    The specified versions above are the ones used when writing this tutorial. You can use other versions, but the usage might differ from the examples in this tutorial.

## Full Example

<https://github.com/pytauri/pytauri/tree/main/examples/tauri-app>

## Create a new tauri project

ref: <https://tauri.app/start/create-project/#using-create-tauri-app>

!!! note
    In this tutorial, we will use [pnpm](https://pnpm.io/) to manage the frontend.

    However, pytauri does not have any opinion on which frontend framework you use. You can even serve the frontend resources via a server using a URL.

```console
pnpm create tauri-app

? Project name (tauri-app) ›
? Identifier (com.tauri-app.app) ›
? Choose which language to use for your frontend ›
    ❯ TypeScript / JavaScript  (pnpm, yarn, npm, deno, bun)
? Choose your package manager ›
    ❯ pnpm
? Choose your UI template ›
    ❯ Vanilla
? Choose your UI flavor ›
    ❯ TypeScript
```

You will get the following directory structure:

```tree
└── tauri-app
    ├── README.md
    ├── index.html
    ├── package.json
    ├── src
    │   ├── assets
    │   ├── main.ts
    │   └── styles.css
    ├── src-tauri
    │   ├── Cargo.toml
    │   ├── build.rs
    │   ├── capabilities
    │   ├── icons
    │   ├── src
    │   └── tauri.conf.json
    ├── tsconfig.json
    └── vite.config.ts
```

- `/tauri-app`: for the frontend
- `/tauri-app/src-tauri`: for rust and python backend

## Launch the tauri app

```bash
cd tauri-app
pnpm install  # (1)!
pnpm tauri dev  # (2)!
```

1. This command will install `tuari-cli`
2. use `tauri-cli` to start the app

!!! info
    The first run will take some time to compile the dependencies, subsequent launches will be much faster.

Congratulations! When you finally see a window with web content appear, you have successfully created a Tauri application.

## Next Steps

Next, we will demonstrate how to integrate Python into the Tauri application using pytauri.
