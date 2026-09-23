#!/usr/bin/env python3
"""Check foundation files and local inline Markdown file links, without downloads."""

from pathlib import Path
import re
import subprocess
import sys
from urllib.parse import unquote, urlsplit


ROOT = Path(__file__).resolve().parent.parent
REQUIRED_FILES = (
    "README.md",
    "LICENSE",
    "CONTRIBUTING.md",
    "SECURITY.md",
    "CHANGELOG.md",
    "docs/project.md",
    "docs/architecture.md",
    "docs/browser-support.md",
    "docs/releases.md",
    "docs/roadmap.md",
    ".editorconfig",
    ".gitattributes",
    ".gitignore",
    ".github/workflows/ci.yml",
    "scripts/check_repository.py",
)
TEXT_SUFFIXES = {".md", ".py", ".rs", ".qg", ".toml", ".yml", ".yaml", ".json", ".txt"}
TEXT_NAMES = {"LICENSE", ".editorconfig", ".gitattributes", ".gitignore"}
INLINE_LINK = re.compile(r"!?\[[^\]\n]*\]\(<?([^\s<>]+?)>?(?:\s+\"[^\"\n]*\")?\)")


def markdown_link_errors(path: Path, text: str) -> list[str]:
    """Check file targets only; skip examples inside fenced blocks and inline code."""
    errors = []
    fence_char = ""
    fence_length = 0
    for line_number, line in enumerate(text.splitlines(), start=1):
        fence = re.match(r"^ {0,3}(`{3,}|~{3,})", line)
        if fence:
            marker = fence.group(1)
            if not fence_char:
                fence_char, fence_length = marker[0], len(marker)
            elif marker[0] == fence_char and len(marker) >= fence_length:
                fence_char = ""
            continue
        if fence_char:
            continue
        prose = re.sub(r"(`+).*?\1", "", line)
        for match in INLINE_LINK.finditer(prose):
            target = match.group(1)
            try:
                url = urlsplit(target)
            except ValueError:
                errors.append(f"{path}:{line_number}: malformed link: {target}")
                continue
            if url.scheme or url.netloc or not url.path:
                continue
            local_path = unquote(url.path)
            base = ROOT if local_path.startswith("/") else (ROOT / path).parent
            destination = (base / local_path.lstrip("/")).resolve()
            if not destination.is_relative_to(ROOT) or not destination.exists():
                errors.append(f"{path}:{line_number}: missing local link target: {target}")
    return errors


def main() -> int:
    errors = []
    for name in REQUIRED_FILES:
        path = ROOT / name
        if not path.is_file() or not path.read_bytes().strip():
            errors.append(f"{name}: required file is missing or empty")

    try:
        result = subprocess.run(
            ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
            cwd=ROOT,
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
    except (OSError, subprocess.CalledProcessError):
        print("Run this checker from a Git checkout with Git installed.", file=sys.stderr)
        return 1

    paths = sorted({Path(name.decode("utf-8")) for name in result.stdout.split(b"\0") if name})
    checked = 0
    for path in paths:
        if path.suffix not in TEXT_SUFFIXES and path.name not in TEXT_NAMES:
            continue
        full_path = ROOT / path
        if not full_path.is_file():
            continue  # A tracked file may be deleted locally before it is staged.
        checked += 1
        data = full_path.read_bytes()
        try:
            text = data.decode("utf-8")
        except UnicodeDecodeError:
            errors.append(f"{path}: expected UTF-8 text")
            continue
        if b"\r" in data:
            errors.append(f"{path}: use LF line endings")
        if data and not data.endswith(b"\n"):
            errors.append(f"{path}: missing final newline")
        for line_number, line in enumerate(text.splitlines(), start=1):
            if line != line.rstrip(" \t"):
                errors.append(f"{path}:{line_number}: trailing whitespace")
        if path.suffix == ".md":
            errors.extend(markdown_link_errors(path, text))

    if errors:
        print("Repository checks failed:", file=sys.stderr)
        for error in errors:
            print(f"  {error}", file=sys.stderr)
        return 1
    print(f"Foundation checks passed ({checked} text files).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
