#!/usr/bin/env python3
"""Validate release inputs, package native builds, and write SHA-256 sums."""

import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import tarfile
import time
import tomllib
import zipfile
from datetime import datetime, timedelta, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TAG = re.compile(r"v(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?")


def current_tag():
    with (ROOT / "Cargo.toml").open("rb") as file:
        return "v" + tomllib.load(file)["package"]["version"]


def validate(tag):
    match = TAG.fullmatch(tag)
    if not match:
        raise SystemExit("Tag must be a version such as v1.2.3 or v1.2.3-alpha.1")
    version = tag[1:]
    package_version = current_tag()[1:]
    if version != package_version:
        raise SystemExit(f"Tag {tag} does not match Cargo.toml version {package_version}")
    notes = ROOT / "release-notes" / f"{tag}.md"
    if not notes.is_file() or not notes.read_text(encoding="utf-8").strip():
        raise SystemExit(f"Missing or empty release notes: {notes.relative_to(ROOT)}")
    return version


def package(tag, target):
    validate(tag)
    windows = "windows" in target
    binary = ROOT / "target" / target / "release" / ("omni-presenter.exe" if windows else "omni-presenter")
    if not binary.is_file():
        raise SystemExit(f"Build executable not found: {binary}")
    name = f"omni-presenter-{tag}-{target}"
    dist = ROOT / "dist"
    stage = dist / name
    if stage.exists():
        shutil.rmtree(stage)
    stage.mkdir(parents=True)
    shutil.copy2(binary, stage / binary.name)
    for source, destination in [
        ("LICENSE", "LICENSE"),
        ("README.md", "README.md"),
        ("docs/operations.md", "operations.md"),
        ("assets/bible-source/LICENSE", "licenses/bible-source-LICENSE"),
        ("assets/fonts/OFL.txt", "licenses/NotoEmoji-OFL.txt"),
    ]:
        dest = stage / destination
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(ROOT / source, dest)
    if windows:
        archive = dist / f"{name}.zip"
        with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED) as output:
            for path in sorted(stage.rglob("*")):
                if path.is_file():
                    output.write(path, path.relative_to(dist))
    else:
        archive = dist / f"{name}.tar.gz"
        with tarfile.open(archive, "w:gz") as output:
            output.add(stage, arcname=name)
    shutil.rmtree(stage)
    print(archive)
    return archive


def checksums(directory, archives=None):
    if archives is None:
        archives = sorted(path for pattern in ("*.zip", "*.tar.gz", "*.AppImage", "*.flatpak", "*.exe")
                          for path in directory.glob(pattern))
    if not archives:
        raise SystemExit(f"No release archives found in {directory}")
    lines = []
    for archive in archives:
        digest = hashlib.sha256()
        with archive.open("rb") as file:
            for chunk in iter(lambda: file.read(1024 * 1024), b""):
                digest.update(chunk)
        lines.append(f"{digest.hexdigest()}  {archive.name}\n")
    (directory / "SHA256SUMS.txt").write_text("".join(lines), encoding="utf-8")


def windows_installer(tag, target):
    validate(tag)
    if target not in ("x86_64-pc-windows-msvc", "aarch64-pc-windows-msvc"):
        raise SystemExit(f"Unsupported Windows installer target: {target}")
    binary = ROOT / "target" / target / "release" / "omni-presenter.exe"
    if not binary.is_file():
        raise SystemExit(f"Build executable not found: {binary}")
    compiler = shutil.which("makensis")
    if not compiler:
        for key in ("ProgramFiles(x86)", "ProgramFiles"):
            candidate = Path(os.environ.get(key, "")) / "NSIS" / "makensis.exe"
            if candidate.is_file():
                compiler = str(candidate)
                break
    if not compiler:
        raise SystemExit("NSIS (makensis) not found; install NSIS and add it to PATH")
    output = ROOT / "dist" / f"omni-presenter-{tag}-{target}-setup.exe"
    output.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run([compiler, "-WX", f"-DVERSION={tag[1:]}", f"-DPROJECT_ROOT={ROOT}",
                    f"-DBINARY={binary}", f"-DOUTPUT={output}",
                    str(ROOT / "packaging/windows/installer.nsi")], cwd=ROOT, check=True)
    print(output)
    return output


def build(target=None):
    tag = current_tag()
    validate(tag)
    if target is None:
        info = subprocess.check_output(["rustc", "-vV"], cwd=ROOT, text=True)
        target = next((line.removeprefix("host: ") for line in info.splitlines()
                       if line.startswith("host: ")), None)
        if not target:
            raise SystemExit("Could not detect the Rust host target; pass --target explicitly")
    print(f"Building {tag} for {target}", flush=True)
    subprocess.run(["cargo", "build", "--release", "--locked", "--target", target], cwd=ROOT, check=True)
    archive = package(tag, target)
    outputs = [archive]
    if target in ("x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu"):
        subprocess.run(["bash", "packaging/linux/package.sh", tag, target], cwd=ROOT, check=True)
        name = f"omni-presenter-{tag}-{target}"
        outputs.extend((ROOT / "dist" / f"{name}{extension}" for extension in (".AppImage", ".flatpak")))
    elif target in ("x86_64-pc-windows-msvc", "aarch64-pc-windows-msvc"):
        outputs.append(windows_installer(tag, target))
    checksums(ROOT / "dist", outputs)
    print(f"SHA-256: {ROOT / 'dist' / 'SHA256SUMS.txt'}")


def publish(remote):
    tag = current_tag()
    validate(tag)
    status = subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT, text=True)
    if status.strip():
        raise SystemExit("Commit or stash all changes before creating a release tag")
    subprocess.run(["git", "tag", tag], cwd=ROOT, check=True)
    try:
        subprocess.run(["git", "push", remote, f"refs/tags/{tag}"], cwd=ROOT, check=True)
    except subprocess.CalledProcessError:
        raise SystemExit(f"Push failed. Local tag {tag} was kept; push it manually or delete it before retrying.")
    print(f"Tag {tag} sent to {remote}; follow the build in GitHub Actions.")


def build_all(remote):
    tag = current_tag()
    validate(tag)
    status = subprocess.check_output(["git", "status", "--porcelain"], cwd=ROOT, text=True)
    if status.strip():
        raise SystemExit("There are uncommitted changes. Commit the release files and push them to the "
                         "GitHub default branch before running the all-platform build; "
                         "see `git status` and docs/releases.md.")
    if not shutil.which("gh"):
        raise SystemExit("GitHub CLI (gh) is required. Install it and run `gh auth login`.")
    subprocess.run(["gh", "auth", "status"], cwd=ROOT, check=True)
    info = json.loads(subprocess.check_output(
        ["gh", "repo", "view", "--json", "nameWithOwner,url,sshUrl"], cwd=ROOT, text=True))
    repo = info["nameWithOwner"]
    remote_url = subprocess.check_output(["git", "remote", "get-url", "--push", remote],
                                         cwd=ROOT, text=True).strip().removesuffix(".git")
    if remote_url not in (info["url"].removesuffix(".git"), info["sshUrl"].removesuffix(".git")):
        raise SystemExit(f"Remote {remote} does not point to the GitHub repository selected by gh ({repo}).")
    workflow = subprocess.run(["gh", "workflow", "view", "release.yml", "--repo", repo], cwd=ROOT,
                              capture_output=True, text=True)
    if workflow.returncode:
        raise SystemExit("Could not access release.yml on the GitHub default branch. "
                         "Commit and push .github/workflows/release.yml and the other release files, "
                         f"then retry. GitHub CLI: {workflow.stderr.strip()}")
    sha = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()
    started = datetime.now(timezone.utc) - timedelta(seconds=5)
    print(f"Starting release {tag} on GitHub ({repo}); all platforms will build there.", flush=True)
    publish(remote)
    deadline = time.monotonic() + 180
    run_id = None
    while time.monotonic() < deadline:
        result = subprocess.check_output(
            ["gh", "run", "list", "--repo", repo, "--workflow", "release.yml", "--branch", tag,
             "--event", "push", "--limit", "30", "--json", "databaseId,headSha,createdAt"],
            cwd=ROOT, text=True)
        for run in json.loads(result):
            created = datetime.fromisoformat(run["createdAt"].replace("Z", "+00:00"))
            if run["headSha"] == sha and created >= started:
                run_id = str(run["databaseId"])
                break
        if run_id:
            break
        time.sleep(5)
    if not run_id:
        raise SystemExit(f"No workflow run found for {tag}. Check GitHub Actions; the tag was already pushed.")
    print(f"Waiting for GitHub Actions run {run_id}...", flush=True)
    subprocess.run(["gh", "run", "watch", run_id, "--repo", repo, "--exit-status"], cwd=ROOT, check=True)
    dist = ROOT / "dist"
    dist.mkdir(parents=True, exist_ok=True)
    subprocess.run(["gh", "release", "download", tag, "--repo", repo, "--dir", str(dist), "--clobber"],
                   cwd=ROOT, check=True)
    manifest = dist / "SHA256SUMS.txt"
    if not manifest.is_file():
        raise SystemExit("The downloaded release is missing SHA256SUMS.txt")
    lines = manifest.read_text(encoding="utf-8").splitlines()
    if not lines:
        raise SystemExit("The downloaded checksum manifest is empty")
    expected = set()
    for target in ("x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu"):
        expected.update(f"omni-presenter-{tag}-{target}{ext}" for ext in (".tar.gz", ".AppImage", ".flatpak"))
    for target in ("x86_64-pc-windows-msvc", "aarch64-pc-windows-msvc"):
        expected.update(f"omni-presenter-{tag}-{target}{ext}" for ext in (".zip", "-setup.exe"))
    for target in ("x86_64-apple-darwin", "aarch64-apple-darwin"):
        expected.add(f"omni-presenter-{tag}-{target}.tar.gz")
    seen = set()
    for line in lines:
        digest, separator, name = line.partition("  ")
        if not separator or not re.fullmatch(r"[0-9a-f]{64}", digest) or Path(name).name != name:
            raise SystemExit(f"Invalid checksum entry: {line}")
        if name in seen:
            raise SystemExit(f"Duplicate checksum entry: {name}")
        seen.add(name)
        path = dist / name
        if not path.is_file():
            raise SystemExit(f"Missing downloaded release file: {name}")
        checksum = hashlib.sha256()
        with path.open("rb") as file:
            for chunk in iter(lambda: file.read(1024 * 1024), b""):
                checksum.update(chunk)
        if checksum.hexdigest() != digest:
            raise SystemExit(f"SHA-256 verification failed: {name}")
    if seen != expected:
        raise SystemExit(f"Unexpected release contents; missing: {sorted(expected - seen)}, extra: {sorted(seen - expected)}")
    print(f"Downloaded and verified {len(lines)} release files in {dist}.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    local = commands.add_parser("build", help="build all platforms on GitHub and download the results")
    local.add_argument("--target", help="build only this Rust target locally")
    mode = local.add_mutually_exclusive_group()
    mode.add_argument("--all", action="store_true", help="build all platforms on GitHub (default)")
    mode.add_argument("--local", action="store_true", help="build only the current platform locally")
    local.add_argument("--remote", default="origin", help="Git remote for the GitHub build (default: origin)")
    release = commands.add_parser("publish", help="push a version tag to start the GitHub release workflow")
    release.add_argument("--remote", default="origin", help="Git remote (default: origin)")
    check = commands.add_parser("validate")
    check.add_argument("tag")
    pack = commands.add_parser("package")
    pack.add_argument("tag")
    pack.add_argument("target")
    installer = commands.add_parser("windows-installer")
    installer.add_argument("tag")
    installer.add_argument("target")
    sums = commands.add_parser("checksums")
    sums.add_argument("directory", type=Path)
    args = parser.parse_args()
    if args.command == "build":
        if not args.local and not args.target:
            build_all(args.remote)
        elif args.all:
            if args.target:
                parser.error("--all and --target cannot be used together")
        else:
            build(args.target)
    elif args.command == "publish":
        publish(args.remote)
    elif args.command == "validate":
        validate(args.tag)
    elif args.command == "package":
        package(args.tag, args.target)
    elif args.command == "windows-installer":
        windows_installer(args.tag, args.target)
    else:
        checksums(args.directory)


if __name__ == "__main__":
    try:
        main()
    except subprocess.CalledProcessError as error:
        raise SystemExit(f"Command failed (exit {error.returncode}): {error.cmd}. See the output above for details.") from None
