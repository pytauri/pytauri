#!/usr/bin/env bash

# TODO: more package tests
pytest python/pyfuture/tests/ --cov --cov-report=xml --cov-report=html
