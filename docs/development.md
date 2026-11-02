# Development and localization

## Build and test

Use a current Rust toolchain and the platform libraries described in [operations](operations.md). From the repository root:

```sh
git submodule update --init --recursive
cargo build
cargo test
python3 tools/i18n_audit.py --catalogs
```

`cargo test` includes domain, service, UI and integration tests. Some tests are explicitly ignored because they require a graphical compositor, multicast network, Chromium or Playwright; inspect their `#[ignore]` annotations before running them in a suitable environment. Media tests may exercise locally installed tools. `tests/architecture_tests.rs` checks that Rust source modules stay within the project's line limit; it is not a substitute for reviewing coupling or behavior.

For portable builds, versioned release notes and GitHub Releases, see the [release guide](releases.md).

## Localization

English (`locales/en/`) is the source catalog; Brazilian Portuguese (`locales/pt-BR/`) and Spanish (`locales/es/`) provide corresponding Fluent files. `build.rs` embeds the resources. The registry negotiates supported locale identifiers and falls back to English per message.

When adding user-visible text:

1. Add a descriptive, stable message ID to the appropriate Fluent file in each locale. Keep variable names and select arguments compatible across translations.
2. Use `Language::text` for simple messages or `Language::format` / `format_args` for messages with arguments. Native UI code can use helpers in `src/ui/i18n.rs`; browser resources use the shared locale data.
3. Run the catalog audit and the Rust tests. The catalog schema test checks message IDs and argument consistency.
4. Review the actual desktop and browser states, including errors and plural or dynamic content. Catalog parity alone does not prove that every visible string uses a catalog.

Use `python3 tools/i18n_audit.py --help` for targeted audits such as `--area workspace` or `--area services`. These scans are a review aid and cannot identify every dynamic or indirectly rendered string.

## Extensions and assets

Extension packages are versioned JSON structures defined in `src/core/plugins/mod.rs`. They can describe components, commands, shaders, interface themes, templates and settings; `assets/plugins/stage-tools.json` is a bundled example. Runtime and validation logic live under `src/core/plugins/` and `src/services/plugins.rs`. Review those implementations and their tests before relying on a package field or execution behavior.

Bible JSON files and locale resources are embedded during the build. Follow the [Bible asset guide](../assets/bible/README.md) when updating translations. When modifying browser assets, check their routing in `src/services/control/http.rs`, `src/services/web.rs` or the relevant chat service as well as the client itself.
