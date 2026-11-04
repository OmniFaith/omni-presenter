#!/usr/bin/env python3
"""Prepare the next minor release notes from commits after a published tag."""

import argparse
import json
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TAG = re.compile(r"v(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?")
START = "<!-- generated-changes:start -->"
END = "<!-- generated-changes:end -->"


def next_tag(tag):
    match = TAG.fullmatch(tag)
    if not match:
        raise ValueError(f"Unsupported release tag: {tag}")
    major, minor, _, prerelease = match.groups()
    suffix = f"-{prerelease.split('.')[0]}" if prerelease else ""
    return f"v{major}.{int(minor) + 1}.0{suffix}"


def latest_published_tag():
    output = subprocess.check_output([
        "gh", "release", "list", "--exclude-drafts", "--limit", "100",
        "--json", "tagName,publishedAt", "--order", "desc",
    ], cwd=ROOT, text=True)
    releases = [release for release in json.loads(output) if TAG.fullmatch(release["tagName"])]
    releases.sort(key=lambda release: release.get("publishedAt") or "", reverse=True)
    return releases[0]["tagName"] if releases else None


def prepare(tag, root=ROOT):
    next_version = next_tag(tag)
    subprocess.run(["git", "merge-base", "--is-ancestor", f"refs/tags/{tag}", "HEAD"],
                   cwd=root, check=True)
    log = subprocess.check_output(
        ["git", "log", "--no-merges", "--format=%h%x09%s", f"refs/tags/{tag}..HEAD"],
        cwd=root, text=True)
    changes = []
    for line in log.splitlines():
        commit, separator, subject = line.partition("\t")
        if separator and not subject.startswith("chore(release-notes):"):
            subject = subject.replace("<", "&lt;").replace(">", "&gt;")
            changes.append(f"- {subject} (`{commit}`)")
    generated = "\n".join(changes) if changes else "- No new commits since the previous release."
    block = f"{START}\n{generated}\n{END}"
    path = root / "release-notes" / f"{next_version}.md"
    if path.exists():
        content = path.read_text(encoding="utf-8")
        if content.count(START) != 1 or content.count(END) != 1 or content.index(START) > content.index(END):
            raise ValueError(f"{path} already exists without valid generated-changes markers; it was not modified")
        before, remainder = content.split(START, 1)
        _, after = remainder.split(END, 1)
        content = before + block + after
    else:
        content = (f"# Omni Presenter {next_version[1:]}\n\n"
                   "## Highlights\n\nAdd user-facing highlights here.\n\n"
                   f"## Changes since {tag}\n\n{block}\n\n"
                   "## Known issues\n\nAdd known issues and migration notes here.\n\n"
                   "## Installation\n\nDownload the package for your operating system and architecture, "
                   "verify it against `SHA256SUMS.txt`, and see the included `operations.md`.\n")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")
    print(path.relative_to(root))
    return path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tag", help="published tag (defaults to the most recently published GitHub release)")
    args = parser.parse_args()
    tag = args.tag or latest_published_tag()
    if not tag:
        print("No published versioned release found; nothing to prepare.")
        return
    prepare(tag)


if __name__ == "__main__":
    try:
        main()
    except (ValueError, subprocess.CalledProcessError) as error:
        raise SystemExit(str(error)) from None
