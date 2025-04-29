# ruff: noqa: D101, D102, D103

"""Release pytauri workspace package.

Accepts a string as a parameter, e.g. "rs:pyo3-utils:v0.1.0", with parts separated by `/`.

- The first part is rs|py|js
- The second part is the package name
- The third part is the semver version number
"""

import argparse
import asyncio
import sys
from argparse import ArgumentTypeError
from asyncio import create_subprocess_exec
from enum import Enum
from logging import basicConfig, getLogger
from os import getenv
from shutil import which
from typing import NamedTuple, NoReturn

logger = getLogger(__name__)


class Kind(Enum):
    RS = "rs"
    PY = "py"
    JS = "js"


class ReleaseTag(NamedTuple):
    kind: Kind
    package: str
    version: str
    """Version number without leading `v`."""

    @staticmethod
    def parse(release_tag: str):
        # got it from `GITHUB_REF` env var
        release_tag = release_tag.removeprefix("refs/tags/")

        kind, package, version = release_tag.split("/")

        if version[0] != "v":
            raise ArgumentTypeError(
                f"version number should start with 'v', got: {version}"
            )

        return ReleaseTag(Kind(kind), package, version[1:])

    def write_to_github_output(self) -> None:
        # see: <https://docs.github.com/zh/actions/writing-workflows/choosing-what-your-workflow-does/passing-information-between-jobs>
        github_output = getenv("GITHUB_OUTPUT")
        if github_output is None:
            logger.warning(
                "`$GITHUB_OUTPUT` is not set, skipping setting github output."
            )
            return
        with open(github_output, "w") as f:
            print(f"kind={self.kind.value}", file=f)
            print(f"package={self.package}", file=f)
            print(f"version={self.version}", file=f)


parser = argparse.ArgumentParser(description="Release pytauri workspace package.")
parser.add_argument(
    "release_tag",
    type=ReleaseTag.parse,
    help="release string, e.g. '[refs/tags/]rs/pyo3-utils/v0.1.0'",
)
parser.add_argument(
    "--no-dry-run",
    action="store_true",
)


_ASSERT_NEVER_REPR_MAX_LENGTH = 100


def _assert_never(arg: NoReturn, /) -> NoReturn:
    value = repr(arg)
    if len(value) > _ASSERT_NEVER_REPR_MAX_LENGTH:
        value = value[:_ASSERT_NEVER_REPR_MAX_LENGTH] + "..."
    raise AssertionError(f"Expected code to be unreachable, but got: {value}")


async def release_rs(package: str, no_dry_run: bool) -> int:
    # <https://doc.rust-lang.org/cargo/reference/publishing.html>
    args = [
        "publish",
        "--all-features",
        "--package",
        package,
        "--color",
        "always",
        # We need to release crates in parallel, so add `--no-verify`.
        # TODO, FIXME: But the downside is that we cannot verify if
        # the versions of workspace dependencies crate are correctly updated.
        "--no-verify",
    ]

    if not no_dry_run:
        args.append("--dry-run")

    proc = await create_subprocess_exec("cargo", *args)
    await proc.wait()

    assert proc.returncode is not None
    return proc.returncode


async def release_py(package: str, no_dry_run: bool) -> int:
    # <https://docs.astral.sh/uv/guides/publish/>
    #
    # NOTE: use `uv build`(without `--wheel` and `--sdist`) to build `wheel` from `sdist`,
    # so that we can notice broken `sdist`. This is important for wheel with extension modules.
    # ref: <https://docs.astral.sh/uv/concepts/projects/build/#using-uv-build>
    args = ["build", "--package", package, "--no-sources", "--color", "always"]
    if no_dry_run:
        raise RuntimeError(
            "python package should only be released by `pypa/gh-action-pypi-publish`"
        )

    proc = await create_subprocess_exec("uv", *args)
    await proc.wait()

    assert proc.returncode is not None
    return proc.returncode


async def release_js(package: str, no_dry_run: bool) -> int:
    # <https://pnpm.io/cli/publish>

    args = [
        "publish",
        "--filter",
        package,
        "--access",
        "public",
        "--color",
        "--fail-if-no-match",
        # NOTE: `--no-git-checks` is necessary,
        # because we run publishing on tag, instead of on a branch (i.e. not `main`)
        "--no-git-checks",
    ]

    if not no_dry_run:
        args.append("--dry-run")

    # on windows, `pnpm` is actually `pnpm.cmd`,
    # so we need to use `which` to find the actual program
    program = which("pnpm")
    if program is None:
        raise FileNotFoundError("`pnpm` is not found in PATH")

    proc = await create_subprocess_exec(program, *args)
    await proc.wait()

    assert proc.returncode is not None
    return proc.returncode


if __name__ == "__main__":
    basicConfig(level="INFO")

    args = parser.parse_args()
    assert isinstance(args.release_tag, ReleaseTag)
    assert isinstance(args.no_dry_run, bool)

    release_tag = args.release_tag
    no_dry_run = args.no_dry_run

    logger.info(f"kind={release_tag.kind.value}")
    logger.info(f"package={release_tag.package}")
    logger.info(f"version={release_tag.version}")
    release_tag.write_to_github_output()

    async def main() -> int:
        if release_tag.kind == Kind.RS:
            return await release_rs(release_tag.package, no_dry_run)
        elif release_tag.kind == Kind.PY:
            return await release_py(release_tag.package, no_dry_run)
        elif release_tag.kind == Kind.JS:
            return await release_js(release_tag.package, no_dry_run)
        else:
            _assert_never(release_tag.kind)

    sys.exit(asyncio.run(main()))
