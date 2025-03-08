"""Generate the code reference pages and navigation.

Copy form: https://mkdocstrings.github.io/recipes/

NOTE: Keep the following directory structure:

📁 repo/
├── 📁 docs/
│   └── 📄 index.md
├── 📁 utils/
│   └── 📄 gen_ref_pages.py
├── 📁 python/
│   └── 📁 project/
│       └── 📁 src/
│           └── 📁 project/
│               ├── 📄 __init__.py
│               └ ...
└── 📄 mkdocs.yml
"""

import re
from pathlib import Path

import mkdocs_gen_files

ROOT = Path(__file__).parent.parent

# Don't change the name "reference/py",
# it's also used in `mkdocs.yml`.
REFERENCE_PREFIX = Path("reference/py")

# matches strings that start with an underscore followed by any character except another underscore.
# - exclude: _private
# - but include: hello, __hello, or __hello__
# see: https://mkdocstrings.github.io/python/usage/configuration/members/#filters
EXCLUDE_PATTERN = re.compile(r"^_[^_]")

nav = mkdocs_gen_files.nav.Nav()

for project in (ROOT / "python").iterdir():
    if not project.is_dir():
        continue
    if not (project / "pyproject.toml").exists():
        continue

    # TODO, FIXME, XXX: Separate this package.
    # Currently it is deprecated, in the future we may contribute it to the community.
    if project.parts[-1] == "pyfuture":
        continue

    project_src = project / "src"
    for path in sorted(project_src.rglob("*.py")):
        # e.g., `pytauri/__init__.py`
        relative_path = path.relative_to(project_src)

        module_path = relative_path.with_suffix("")
        doc_path = relative_path.with_suffix(".md")

        full_doc_path = Path(REFERENCE_PREFIX, doc_path)

        parts = tuple(module_path.parts)

        # TODO: optimize the performance of this part.
        # exclude private packages.
        # parts[0] is the top-level package, so we don't apply the pattern to it.
        # parts[-1] is the `.py` file (i.e., a module, not a package), so we don't apply the pattern to it.
        if any(EXCLUDE_PATTERN.search(part) for part in parts[1:-1]):
            continue

        index_md = "index.md"
        if parts[-1] == "__init__":
            parts = parts[:-1]
            doc_path = doc_path.with_name(index_md)
            full_doc_path = full_doc_path.with_name(index_md)
        # exclude private modules
        elif EXCLUDE_PATTERN.search(parts[-1]):
            continue

        nav[parts] = doc_path.as_posix()

        with mkdocs_gen_files.open(full_doc_path, "w") as fd:
            ident = ".".join(parts)
            fd.writelines(f"::: {ident}")

        # The base edit path is set in the `mkdocs.yml`:
        # e.g., `https://github.com/pytauri/pytauri/edit/main/docs/`.
        # Since these api reference(code) are not actually in the `docs` directory,
        # but are inlined in the `*.py` code, we need `"../"` to remove the `docs/` path
        mkdocs_gen_files.set_edit_path(
            full_doc_path, Path("../") / path.relative_to(ROOT)
        )

    # ref: <https://github.com/oprypin/mkdocs-literate-nav>
    with mkdocs_gen_files.open(REFERENCE_PREFIX / "SUMMARY.md", "w") as nav_file:
        nav_file.writelines(nav.build_literate_nav())
