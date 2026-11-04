# Builds and releases

The `.github/workflows/release.yml` workflow builds portable packages and creates a **draft** GitHub Release. Review and publish the draft on GitHub. See the [versioning guide](versioning.md) for choosing version numbers and prerelease labels.

## Platforms

| Operating system | Architecture | Rust target | Packages |
| --- | --- | --- | --- |
| Linux (glibc) | x86-64 | `x86_64-unknown-linux-gnu` | `.tar.gz`, `.AppImage`, `.flatpak` |
| Linux (glibc) | ARM64 | `aarch64-unknown-linux-gnu` | `.tar.gz`, `.AppImage`, `.flatpak` |
| Windows | x86-64 | `x86_64-pc-windows-msvc` | `.zip`, `-setup.exe` installer |
| Windows | ARM64 | `aarch64-pc-windows-msvc` | `.zip`, `-setup.exe` installer |
| macOS | Intel | `x86_64-apple-darwin` | `.tar.gz` |
| macOS | Apple Silicon | `aarch64-apple-darwin` | `.tar.gz` |

GitHub ARM runners must be available to the repository. Linux binaries and AppImages use the Ubuntu 24.04 glibc baseline and may not work on older distributions. Windows and macOS packages are not signed or notarized and may trigger operating-system warnings. `ffmpeg`, `ffprobe`, and `yt-dlp` are external dependencies for the features described in [operations.md](operations.md). Host-installed programs are not automatically available inside the Flatpak sandbox, so these video features are not guaranteed there.

The Windows installer installs for the current user without administrator privileges and creates Start Menu and desktop shortcuts. The default installation directory is `%LOCALAPPDATA%\Programs\Omni Presenter`. Shortcuts use `%LOCALAPPDATA%\Omni Presenter\workspace` as the working directory for the database and other project files. Uninstalling preserves this workspace. Launching the executable directly or using the portable `.zip` may use a different working directory and database.

Install the Flatpak with `flatpak install --user ./package-name.flatpak` and launch it with `flatpak run org.omnipresenter.OmniPresenter`; installation may download the runtime from Flathub. Its database is stored in the application's private data directory (`~/.var/app/org.omnipresenter.OmniPresenter/data/omni-presenter`), separate from other package formats. To use the AppImage, make it executable (`chmod +x package-name.AppImage`) and launch it.

## Prepare a release

1. Choose a version using the [versioning guide](versioning.md). Update `package.version` in `Cargo.toml` and run `cargo check` to update `Cargo.lock`. Review and commit both files.
2. Review the automatically prepared `release-notes/vX.Y.Z.md` (or, for example, `release-notes/vX.Y.Z-alpha.1.md`) and edit its user-facing highlights and known issues. If no file exists for the chosen version, copy `release-notes/TEMPLATE.md`. Keep previous files as release history. See the [versioning guide](versioning.md) for how upcoming notes are generated.
3. Commit and push these changes, including the workflow, to GitHub. Install the [GitHub CLI](https://cli.github.com/) and authenticate with `gh auth login`. Then run:

   ```sh
   python3 tools/release.py build
   ```

   The command reads the version from `Cargo.toml`, validates the notes, creates and pushes the tag to `origin`, waits for GitHub Actions, and downloads the twelve packages and checksum manifest to `dist/`. It verifies their hashes and replaces files with the same names. The other platforms are built on GitHub runners, not locally; internet access, repository access, and runner availability are required. On Windows, use `python`. The Git working tree must be clean. An existing tag may be reused only if it points to the current commit. Otherwise, choose a new version and add its release notes. Use `--remote name` for another remote pointing to the repository selected by `gh`. If a tag push fails, the local tag is kept for manual recovery.
4. Follow **Actions → Build and draft release**. The workflow verifies the tag, version, and notes, builds the six targets, and creates a draft only when all builds succeed. Tags with suffixes, such as `v1.2.3-alpha.1`, are marked as prereleases.
5. Review the notes and thirteen attachments (twelve packages and `SHA256SUMS.txt`) in GitHub Releases. Test the packages as appropriate, then publish the draft.

To start a release without waiting for or downloading the results, use `python3 tools/release.py publish`.

To retry manually, open **Actions → Build and draft release → Run workflow** and provide an existing tag. The workflow uses the code and notes at that tag. It does not replace an existing release; remove a previous draft before retrying. For code fixes, create a new version and tag rather than moving a published tag.

## Verify downloads

Download `SHA256SUMS.txt` and the packages to the same directory. On Linux, `sha256sum -c SHA256SUMS.txt --ignore-missing` verifies the files present. On macOS, `shasum -a 256 -c SHA256SUMS.txt` checks every listed file, so download them all or filter the line for the desired package. In PowerShell, compare `Get-FileHash .\package-name.zip -Algorithm SHA256` with the matching line in `SHA256SUMS.txt`. Obtain the manifest from the official release page: hashes detect changes but are not a digital signature.

## Local build

With Python 3.11+, Rust, and the platform development libraries installed, run:

```sh
git submodule update --init --recursive
python3 tools/release.py build --local
```

The local command reads the version from `Cargo.toml` (including suffixes such as `-alpha`), detects the Rust host target, and builds, packages, and hashes that platform only. On Linux it also generates AppImage and Flatpak packages: install `flatpak`, `flatpak-builder`, `curl`, and `rsvg-convert` (`librsvg2-bin` on Debian/Ubuntu or `librsvg` on Arch), in addition to development libraries. Internet access is needed to download linuxdeploy and the Flathub runtimes. Run Linux packaging on a machine matching the target architecture. On Windows, install [NSIS](https://nsis.sourceforge.io/Download) to build the installer and use `python` instead of `python3`. Release notes must exist before building. To choose an installed and configured target, use `python3 tools/release.py build --target TRIPLE`. Output is written to `dist/`; the local checksum manifest includes only the packages just generated. The working directory determines where portable builds create their database.
