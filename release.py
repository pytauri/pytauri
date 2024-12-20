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
from asyncio import create_subprocess_exec
from enum import Enum
from logging import basicConfig, getLogger
from os import getenv
from typing import NamedTuple

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


async def release_rs(package: str, no_dry_run: bool) -> int:
    args = ["publish", "--all-features", "--package", package, "--color", "always"]
    if no_dry_run:
        args.append("--no-verify")
    else:
        args.append("--dry-run")

    proc = await create_subprocess_exec("cargo", *args)
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
        raise NotImplementedError()

    sys.exit(asyncio.run(main()))
