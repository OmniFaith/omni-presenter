use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResourceKind {
    Song,
    Text,
    Verse,
    Media,
    Design,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum ControlCommand {
    Select {
        kind: ResourceKind,
        id: String,
        #[serde(default)]
        project: bool,
    },
    ProjectPreview {
        #[serde(default)]
        index: Option<usize>,
    },
    PreviewSlide {
        index: usize,
    },
    LiveSlide {
        index: usize,
    },
    Navigate {
        direction: String,
    },
    Visibility {
        hidden: bool,
    },
    Stop,
    Theme {
        id: String,
        #[serde(default)]
        live: bool,
    },
    PlaylistAdd {
        kind: ResourceKind,
        id: String,
    },
    PlaylistSelect {
        id: String,
        #[serde(default)]
        project: bool,
    },
    PlaylistMove {
        id: String,
        target: String,
        #[serde(default)]
        after: bool,
    },
    PlaylistRemove {
        id: String,
    },
    PlaylistPlayed {
        id: String,
        played: bool,
    },
    PlaylistClear,
    PlaylistInsert {
        kind: ResourceKind,
        id: String,
        #[serde(default)]
        target: Option<String>,
        #[serde(default)]
        after: bool,
    },
    PlaylistBatch {
        operation: String,
        #[serde(default)]
        ids: Vec<String>,
        #[serde(default)]
        target: Option<String>,
        #[serde(default)]
        after: bool,
        #[serde(default)]
        played: bool,
    },
    PlaylistSelection {
        id: String,
        #[serde(default)]
        range: bool,
        #[serde(default)]
        additive: bool,
    },
    BibleVersion {
        id: String,
    },
    BibleOptions {
        merge: bool,
        cards: bool,
    },
    BibleShortcut {
        id: String,
        operation: String,
        #[serde(default)]
        target: Option<String>,
    },
    Pin {
        kind: ResourceKind,
        id: String,
    },
    QuickSelect {
        id: String,
        #[serde(default)]
        project: bool,
    },
    QuickRemove {
        id: String,
    },
    Bible {
        reference: String,
        version: String,
        #[serde(default)]
        merge: bool,
        #[serde(default)]
        project: bool,
        #[serde(default)]
        playlist: bool,
    },
    Text {
        title: String,
        content: String,
        #[serde(default)]
        project: bool,
        #[serde(default)]
        playlist: bool,
    },
    SaveSong {
        #[serde(default)]
        id: Option<i64>,
        title: String,
        artist: String,
        content: String,
        #[serde(default)]
        folder: Option<String>,
        #[serde(default)]
        tags: Option<String>,
    },
    OverlayClear,
    Timer {
        operation: String,
    },
    Player {
        operation: String,
        #[serde(default)]
        value: Option<f32>,
    },
    Service {
        id: String,
    },
    ServiceDate {
        date: String,
    },
    ServiceStepDate {
        delta: i32,
    },
    WorkspaceLayout {
        bible: bool,
        layout: crate::core::layout::LayoutTree,
    },
    PinPreview,
    WorkspaceReset {
        bible: bool,
    },
    DockVisibility {
        visible: bool,
    },
    DockPreviewMode {
        background: bool,
    },
}
