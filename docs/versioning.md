# Versioning policy

Omni Presenter uses [Semantic Versioning](https://semver.org/) in the form `MAJOR.MINOR.PATCH`, optionally followed by a prerelease label. The version in `Cargo.toml`, the Git tag (with a `v` prefix), and the release-notes filename must match: for example, `0.7.1-alpha`, `v0.7.1-alpha`, and `release-notes/v0.7.1-alpha.md`.

## Choosing a version

| Change | Before 1.0 | After 1.0 |
| --- | --- | --- |
| Fixes or small compatible improvements | Increment PATCH, e.g. `0.7.1` → `0.7.2` | Increment PATCH, e.g. `1.2.1` → `1.2.2` |
| New functionality or significant changes | Increment MINOR and reset PATCH, e.g. `0.7.2` → `0.8.0` | Increment MINOR for backward-compatible features, e.g. `1.2.2` → `1.3.0` |
| Incompatible changes | Increment MINOR, e.g. `0.7.2` → `0.8.0`; describe migration needs | Increment MAJOR and reset MINOR and PATCH, e.g. `1.3.0` → `2.0.0` |

Versions beginning with `0` are initial development releases: interfaces and behavior may change, and SemVer does not promise compatibility. The table is the project's convention for communicating changes during that period. Note breaking changes and migration instructions in the release notes.

After a release is published, the release-notes automation prepares the next **minor** version by default: for example, `0.7.1` → `0.8.0`, or `0.7.1-alpha` → `0.8.0-alpha`. This is a starting point for the next planned release, not a requirement to label every fix as a minor release. Maintainers can choose a different version when appropriate and must update `Cargo.toml` and the matching notes before releasing.

Release `1.0.0` when the maintainers consider the core workflow ready for general use, with a documented and reasonably stable compatibility contract for user data and supported integrations. Version `1.0.0` is a readiness milestone, not a promise that the application is bug-free. Document compatibility and migration limitations in the release notes.

## Prereleases

Append a hyphen and label to identify a version that is not yet the final release:

- `0.8.0-alpha.1`: early testing; functionality and interfaces may change.
- `0.8.0-beta.1`: broader testing after the main functionality is in place.
- `0.8.0-rc.1`: release candidate intended for final validation.
- `0.8.0`: final release of that version.

Increase the numeric suffix for subsequent candidates of the same stage, such as `alpha.1` → `alpha.2`. A version without a suffix sorts after its prereleases. The existing `-alpha` suffix without a number is valid; `0.7.1-alpha` sorts before `0.7.1-alpha.1`. Choose labels based on actual readiness.

## Tags and release notes

Use a new version and tag for a new commit after a release has been created. A Git tag identifies the exact source used for its artifacts; do not silently move an existing published tag. If a build fails, retry the existing tag only when the source does not need to change. If a fix is needed, commit it under a new version and tag. Describe user-facing changes, known issues, and migration steps in `release-notes/vVERSION.md` using the [template](../release-notes/TEMPLATE.md). See [builds and releases](releases.md) for the release procedure.

The **Prepare next release notes** GitHub Actions workflow creates the next minor notes file after publication and updates its generated changes on subsequent pushes to the default branch. It derives entries from commit subjects since the most recently published version tag, excluding its own automated commits. Write useful commit subjects and review the generated text: commit messages are not necessarily complete or suitable for end users. Edit the highlights, known issues, and installation sections freely; keep the `generated-changes` markers around the automatically managed section. An existing notes file without those markers is never overwritten. The workflow commits updates to the default branch and therefore needs permission to push there; pull the branch before starting further work. Branch protection may require adjusting repository permissions or applying the generated notes manually with `python3 tools/prepare_release_notes.py --tag vX.Y.Z`.
