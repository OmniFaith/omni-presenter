use super::state::{SettingsTab, SidebarPage};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Command {
    Search,
    BibleSearch,
    Settings,
    Home,
    Bible,
    Chat,
    Design,
    NextSlide,
    PreviousSlide,
    FirstSlide,
    LastSlide,
    JumpSlide,
    ToggleHide,
    TogglePresentation,
    StopPresentation,
    NewSong,
    WebLyricsSearch,
    ImportMedia,
    NewText,
    SaveVerse,
    NewDesign,
    NewTheme,
    ToggleDock,
    ResetLayout,
    UndoDesign,
    RedoDesign,
    DuplicateLayer,
    DeleteLayer,
    GroupLayers,
    UngroupLayers,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CommandScope {
    Global,
    Presentation,
    Design,
}

impl Command {
    pub fn scope(self) -> CommandScope {
        if self.is_design_edit() {
            CommandScope::Design
        } else if matches!(
            self,
            Self::NextSlide
                | Self::PreviousSlide
                | Self::FirstSlide
                | Self::LastSlide
                | Self::JumpSlide
                | Self::ToggleHide
        ) {
            CommandScope::Presentation
        } else {
            CommandScope::Global
        }
    }

    pub fn conflicts_with(self, other: Self) -> bool {
        self.scope() == other.scope()
            || self.scope() == CommandScope::Global
            || other.scope() == CommandScope::Global
    }

    pub fn available_on(self, page: SidebarPage) -> bool {
        match self.scope() {
            CommandScope::Global => true,
            CommandScope::Design => page == SidebarPage::Design,
            CommandScope::Presentation => page != SidebarPage::Design,
        }
    }
    pub fn group(self) -> CommandGroup {
        match self {
            Self::Search
            | Self::BibleSearch
            | Self::Settings
            | Self::Home
            | Self::Bible
            | Self::Chat
            | Self::Design => CommandGroup::Navigation,
            Self::NextSlide
            | Self::PreviousSlide
            | Self::FirstSlide
            | Self::LastSlide
            | Self::JumpSlide
            | Self::ToggleHide
            | Self::TogglePresentation
            | Self::StopPresentation
            | Self::ToggleDock => CommandGroup::Presentation,
            Self::NewSong
            | Self::WebLyricsSearch
            | Self::ImportMedia
            | Self::NewText
            | Self::SaveVerse
            | Self::NewDesign
            | Self::NewTheme => CommandGroup::Creation,
            Self::UndoDesign
            | Self::RedoDesign
            | Self::DuplicateLayer
            | Self::DeleteLayer
            | Self::GroupLayers
            | Self::UngroupLayers => CommandGroup::Design,
            Self::ResetLayout => CommandGroup::Workspace,
        }
    }
    pub const ALL: [Self; 30] = [
        Self::Search,
        Self::BibleSearch,
        Self::Settings,
        Self::Home,
        Self::Bible,
        Self::Chat,
        Self::Design,
        Self::NextSlide,
        Self::PreviousSlide,
        Self::FirstSlide,
        Self::LastSlide,
        Self::JumpSlide,
        Self::ToggleHide,
        Self::TogglePresentation,
        Self::StopPresentation,
        Self::NewSong,
        Self::WebLyricsSearch,
        Self::ImportMedia,
        Self::NewText,
        Self::SaveVerse,
        Self::NewDesign,
        Self::NewTheme,
        Self::ToggleDock,
        Self::ResetLayout,
        Self::UndoDesign,
        Self::RedoDesign,
        Self::DuplicateLayer,
        Self::DeleteLayer,
        Self::GroupLayers,
        Self::UngroupLayers,
    ];

    pub fn label_key(self) -> &'static str {
        match self {
            Self::Search => "command-search",
            Self::BibleSearch => "command-bible-search",
            Self::Settings => "command-settings",
            Self::Home => "command-home",
            Self::Bible => "command-bible",
            Self::Chat => "command-chat",
            Self::Design => "command-design",
            Self::NextSlide => "command-next",
            Self::PreviousSlide => "command-previous",
            Self::FirstSlide => "command-first",
            Self::LastSlide => "command-last",
            Self::JumpSlide => "command-jump-slide",
            Self::ToggleHide => "command-hide",
            Self::TogglePresentation => "command-present",
            Self::StopPresentation => "command-stop",
            Self::NewSong => "command-new-song",
            Self::WebLyricsSearch => "command-web-lyrics-search",
            Self::ImportMedia => "command-import-media",
            Self::NewText => "command-new-text",
            Self::SaveVerse => "command-save-verse",
            Self::NewDesign => "command-new-design",
            Self::NewTheme => "command-new-theme",
            Self::ToggleDock => "command-dock",
            Self::ResetLayout => "command-reset-layout",
            Self::UndoDesign => "command-undo",
            Self::RedoDesign => "command-redo",
            Self::DuplicateLayer => "command-duplicate",
            Self::DeleteLayer => "command-delete-layer",
            Self::GroupLayers => "command-group-layers",
            Self::UngroupLayers => "command-ungroup-layers",
        }
    }

    pub fn is_design_edit(self) -> bool {
        matches!(
            self,
            Self::UndoDesign
                | Self::RedoDesign
                | Self::DuplicateLayer
                | Self::DeleteLayer
                | Self::GroupLayers
                | Self::UngroupLayers
        )
    }

    pub fn page(self) -> Option<SidebarPage> {
        match self {
            Self::Home => Some(SidebarPage::Home),
            Self::Bible => Some(SidebarPage::Bible),
            Self::Chat => Some(SidebarPage::Chat),
            Self::Design => Some(SidebarPage::Design),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CommandGroup {
    Navigation,
    Presentation,
    Creation,
    Design,
    Workspace,
}

impl CommandGroup {
    pub const ALL: [Self; 5] = [
        Self::Navigation,
        Self::Presentation,
        Self::Creation,
        Self::Design,
        Self::Workspace,
    ];

    pub fn label_key(self) -> &'static str {
        match self {
            Self::Navigation => "shortcuts-navigation",
            Self::Presentation => "shortcuts-presentation",
            Self::Creation => "shortcuts-creation",
            Self::Design => "shortcuts-design",
            Self::Workspace => "shortcuts-workspace",
        }
    }
}

pub const SETTINGS_PAGES: [(SettingsTab, &str); 9] = [
    (SettingsTab::Outputs, "settings-outputs-title"),
    (SettingsTab::Interface, "settings-interface-title"),
    (SettingsTab::Shortcuts, "shortcuts-title"),
    (SettingsTab::Plugins, "plugins-title"),
    (SettingsTab::Bible, "settings-bibles-title"),
    (SettingsTab::Connections, "settings-connections-title"),
    (SettingsTab::Profiles, "settings-profiles-title"),
    (SettingsTab::Transfer, "transfer-title"),
    (SettingsTab::About, "settings-about-title"),
];
