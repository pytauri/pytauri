#!/usr/bin/env bash
ruff check . --fix && \
ruff format .
