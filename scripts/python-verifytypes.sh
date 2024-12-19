#!/usr/bin/env bash
pnpm pyright --verifytypes pytauri_demo --ignoreexternal && \
pnpm pyright --verifytypes codelldb --ignoreexternal && \
pnpm pyright --verifytypes pyfuture --ignoreexternal && \
pnpm pyright --verifytypes pyo3_utils --ignoreexternal && \
pnpm pyright --verifytypes pytauri --ignoreexternal && \
pnpm pyright --verifytypes pytauri_plugin_notification --ignoreexternal
