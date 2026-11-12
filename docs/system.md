# System architecture

## Runtime boundaries

`src/main.rs` creates the eframe application, installs fonts and rendering resources, and starts `PresentationApp`. The application coordinates the UI frame, persistent data, media services, physical output and browser publication.

```text
Desktop UI ───────────────┐
                          ▼
Browser control → command queue → PresentationApp → SlideDeck / ControlPanelState
                                         │                 │
                                         ▼                 ▼
                                   AppDb (SQLite)    preview and live outputs
                                                           │
                                           native renderers + browser snapshots
```

Browser commands are received by `services/control/`, queued in `WebControl` and applied on the application thread by `app/remote.rs`. A published snapshot supplies browser clients with current state; output publication uses separate preview and live renderers. The queue has a bounded capacity and a response timeout, so callers must handle busy or unavailable application responses.

`SlideDeck` represents presentation content and navigation, including separate preview and live state. `ControlPanelState` holds workspace state and settings. `PresentationApp` coordinates these objects rather than allowing an HTTP handler to mutate the desktop state directly. `AppDb` wraps the SQLite connection and groups persistence by domain under `services/db/`.

## Source layout

| Path | Responsibility |
| --- | --- |
| `src/core/` | Domain models, presentation state, layout, preferences, commands, localization and extension definitions. |
| `src/app/` | Application lifecycle, frames, remote-command application, snapshots, output and persistence coordination. |
| `src/services/` | SQLite, web servers, chat, discovery, media, search, background tasks and plugin execution. |
| `src/render/` | Native presentation and design rendering, including GPU resources and shaders. |
| `src/ui/` | Desktop components, containers, pages, editors and dialogs. |
| `assets/web/` | Browser control, chat and viewer clients served by the application. |
| `assets/plugins/` | Bundled extension package and browser runtime resources. |
| `assets/bible-source/` | Pinned third-party Bible repository; the build embeds its `json/` data. |
| `locales/` | Fluent resources embedded at build time for desktop and browser clients. |
| `tests/` | Integration-level constraints, including the Rust module-size check. |

## Persistence and output

Startup opens the working directory's `omni-presenter.db`, initializes the schema and seeds defaults as needed. Saved preferences are validated before constructing the application. Open/decoding failures stop startup with an explicit error instead of silently using an in-memory database. Settings are loaded into application state; changes and design edits are persisted through `AppDb`. See [operations](operations.md#prerequisites-and-startup).

Each service remembers its last clicked preview theme by ID, separately from its
configured default theme. Profile capture/autosave and normal shutdown persist
that selection. Startup/profile restoration prefers the last valid selection,
then the event default, then the first available theme. Remembering a preview
selection does not change live output during a session. Explicitly changing an
event's default invalidates its older remembered selection.

Native presentation rendering and browser output share domain content but use different rendering paths. The browser server publishes presentation state and serves the viewer's assets. Images, fonts, video and design assets may be exposed through generated URLs rather than local filesystem paths. Test both paths when changing typography, theme behavior or presentation content.

Media decoding, image loading and other potentially blocking work use dedicated services and background tasks. Keep filesystem, process and network work off the immediate-mode UI path when extending these features.

## Change boundaries

For a new browser action, define the command in `services/control/types.rs`, route it through the existing control service and apply it in the application layer. Add or adjust the published snapshot as needed. For a new domain feature, keep serialization and behavior in `core`, persistence in the appropriate `services/db/` module, and UI concerns in `ui`.

`ControlPanelState`, `SlideDeck`, `AppTheme` and `AppDb` are highly connected. Treat the Graphify output in `graphify-out/`, when generated, as an index for investigation rather than proof of a runtime dependency or a performance problem.
