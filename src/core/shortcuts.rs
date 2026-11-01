use super::commands::Command;
use eframe::egui::{self, Key, Modifiers};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Default)]
pub struct ShortcutCapture {
    pub command: Option<Command>,
    pub replace_index: Option<usize>,
    pub error: String,
    pub query: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyBinding {
    pub key: String,
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

impl KeyBinding {
    pub fn new(key: Key, ctrl: bool, shift: bool, alt: bool) -> Self {
        Self {
            key: key.name().into(),
            ctrl,
            shift,
            alt,
        }
    }

    pub fn from_event(event: &egui::Event) -> Option<Self> {
        match event {
            egui::Event::Key {
                key,
                pressed: true,
                repeat: false,
                modifiers,
                ..
            } if !matches!(
                key,
                Key::ControlLeft
                    | Key::ControlRight
                    | Key::ShiftLeft
                    | Key::ShiftRight
                    | Key::AltLeft
                    | Key::AltRight
                    | Key::SuperLeft
                    | Key::SuperRight
            ) =>
            {
                Some(Self::new(
                    *key,
                    modifiers.ctrl || modifiers.command,
                    modifiers.shift,
                    modifiers.alt,
                ))
            }
            _ => None,
        }
    }

    pub fn matches(&self, key: Key, modifiers: Modifiers) -> bool {
        Key::from_name(&self.key) == Some(key)
            && self.ctrl == (modifiers.ctrl || modifiers.command)
            && self.shift == modifiers.shift
            && self.alt == modifiers.alt
    }

    pub fn display(&self) -> String {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl");
        }
        if self.alt {
            parts.push("Alt");
        }
        if self.shift {
            parts.push("Shift");
        }
        parts.push(match self.key.as_str() {
            "Comma" => ",",
            "Period" => ".",
            "Semicolon" => ";",
            "Slash" => "/",
            "Backslash" => "\\",
            "ArrowRight" => "→",
            "ArrowLeft" => "←",
            "ArrowUp" => "↑",
            "ArrowDown" => "↓",
            "PageUp" => "Page Up",
            "PageDown" => "Page Down",
            "Escape" => "Esc",
            _ => &self.key,
        });
        parts.join(" + ")
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct ShortcutSettings(BTreeMap<Command, Vec<KeyBinding>>);

impl<'de> Deserialize<'de> for ShortcutSettings {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StoredBinding {
            Multiple(Vec<KeyBinding>),
            Legacy(Option<KeyBinding>),
        }
        let stored = BTreeMap::<Command, StoredBinding>::deserialize(deserializer)?;
        Ok(Self(
            stored
                .into_iter()
                .map(|(command, binding)| {
                    (
                        command,
                        match binding {
                            StoredBinding::Multiple(bindings) => bindings,
                            StoredBinding::Legacy(binding) => binding.into_iter().collect(),
                        },
                    )
                })
                .collect(),
        ))
    }
}

impl ShortcutSettings {
    pub fn bindings(&self, command: Command) -> Vec<KeyBinding> {
        self.0
            .get(&command)
            .cloned()
            .unwrap_or_else(|| default_bindings(command))
    }

    pub fn assign(&mut self, command: Command, bindings: Vec<KeyBinding>) -> Result<(), Command> {
        for binding in &bindings {
            if let Some(conflict) = Command::ALL.into_iter().find(|other| {
                *other != command
                    && command.conflicts_with(*other)
                    && self.bindings(*other).contains(binding)
            }) {
                return Err(conflict);
            }
        }
        let mut unique = Vec::new();
        for binding in bindings {
            if !unique.contains(&binding) {
                unique.push(binding);
            }
        }
        self.0.insert(command, unique);
        Ok(())
    }

    pub fn add(
        &mut self,
        command: Command,
        binding: KeyBinding,
        replace_index: Option<usize>,
    ) -> Result<(), Command> {
        let mut bindings = self.bindings(command);
        if let Some(index) = replace_index.and_then(|index| bindings.get_mut(index)) {
            *index = binding;
        } else {
            bindings.push(binding);
        }
        self.assign(command, bindings)
    }

    pub fn remove(&mut self, command: Command, index: usize) {
        let mut bindings = self.bindings(command);
        if index < bindings.len() {
            bindings.remove(index);
        }
        self.0.insert(command, bindings);
    }

    pub fn reset(&mut self) {
        self.0.clear();
    }

    pub fn restore(&mut self, command: Command) -> Result<(), Command> {
        self.assign(command, default_bindings(command))?;
        self.0.remove(&command);
        Ok(())
    }

    pub fn pressed(&self, ctx: &egui::Context, command: Command) -> bool {
        self.bindings(command).iter().any(|binding| {
            ctx.input(|input| {
                input.events.iter().any(|event| match event {
                    egui::Event::Key {
                        key,
                        modifiers,
                        pressed: true,
                        repeat,
                        ..
                    } => {
                        (!repeat || matches!(command, Command::NextSlide | Command::PreviousSlide))
                            && binding.matches(*key, *modifiers)
                    }
                    _ => false,
                })
            })
        })
    }

    pub fn consume(&self, ctx: &egui::Context, command: Command) -> bool {
        let bindings = self.bindings(command);
        ctx.input_mut(|input| {
            let mut found = false;
            input.events.retain(|event| {
                let matches = matches!(event, egui::Event::Key { key, modifiers, pressed: true, repeat: false, .. } if bindings.iter().any(|binding| binding.matches(*key, *modifiers)));
                found |= matches;
                !matches
            });
            found
        })
    }
}

fn default_bindings(command: Command) -> Vec<KeyBinding> {
    if command == Command::NextSlide {
        return [Key::ArrowRight, Key::ArrowDown, Key::PageDown, Key::Space]
            .into_iter()
            .map(|key| KeyBinding::new(key, false, false, false))
            .collect();
    }
    if command == Command::PreviousSlide {
        return [Key::ArrowLeft, Key::ArrowUp, Key::PageUp]
            .into_iter()
            .map(|key| KeyBinding::new(key, false, false, false))
            .collect();
    }
    if command == Command::JumpSlide {
        return [Key::Quote, Key::Slash]
            .into_iter()
            .map(|key| KeyBinding::new(key, false, false, false))
            .collect();
    }
    let (key, ctrl, shift) = match command {
        Command::Search => (Key::F, true, false),
        Command::BibleSearch => (Key::B, true, false),
        Command::Settings => (Key::Comma, true, false),
        Command::Home => (Key::Num1, true, false),
        Command::Bible => (Key::Num2, true, false),
        Command::Chat => (Key::Num3, true, false),
        Command::Design => (Key::Num4, true, false),
        Command::JumpSlide => return Vec::new(),
        Command::NextSlide => (Key::ArrowRight, false, false),
        Command::PreviousSlide => (Key::ArrowLeft, false, false),
        Command::FirstSlide => (Key::Home, false, false),
        Command::LastSlide => (Key::End, false, false),
        Command::ToggleHide => (Key::B, false, false),
        Command::TogglePresentation => (Key::F5, false, false),
        Command::StopPresentation => (Key::Escape, false, false),
        Command::NewSong => (Key::N, true, false),
        Command::WebLyricsSearch => (Key::W, true, true),
        Command::ImportMedia => (Key::I, true, false),
        Command::NewText => (Key::T, true, true),
        Command::SaveVerse => (Key::B, true, true),
        Command::NewDesign => (Key::D, true, true),
        Command::NewTheme => (Key::M, true, true),
        Command::ToggleDock => (Key::F9, false, false),
        Command::ResetLayout => (Key::R, true, true),
        Command::UndoDesign => (Key::Z, true, false),
        Command::RedoDesign => (Key::Z, true, true),
        Command::DuplicateLayer => (Key::D, true, false),
        Command::DeleteLayer => (Key::Delete, false, false),
        Command::GroupLayers => (Key::G, true, false),
        Command::UngroupLayers => (Key::G, true, true),
    };
    vec![KeyBinding::new(key, ctrl, shift, false)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disjoint_contexts_can_share_a_binding_but_global_commands_cannot() {
        let mut settings = ShortcutSettings::default();
        let binding = KeyBinding::new(Key::Z, true, false, false);
        settings
            .assign(Command::NextSlide, vec![binding.clone()])
            .unwrap();
        assert!(settings.assign(Command::Settings, vec![binding]).is_err());
        assert!(Command::NextSlide.available_on(crate::core::state::SidebarPage::Home));
        assert!(!Command::NextSlide.available_on(crate::core::state::SidebarPage::Design));
        assert!(Command::UndoDesign.available_on(crate::core::state::SidebarPage::Design));
        settings.restore(Command::NextSlide).unwrap();
        assert_eq!(settings.bindings(Command::NextSlide).len(), 4);
    }

    #[test]
    fn every_command_ships_with_a_default_binding() {
        for command in Command::ALL {
            assert!(
                !default_bindings(command).is_empty(),
                "{command:?} has no default binding"
            );
        }
    }

    #[test]
    fn capture_ignores_modifier_presses_and_waits_for_the_combination() {
        let event = |key, modifiers| egui::Event::Key {
            key,
            physical_key: Some(key),
            pressed: true,
            repeat: false,
            modifiers,
        };
        let events = [
            event(Key::ControlLeft, Modifiers::NONE),
            event(Key::ShiftLeft, Modifiers::CTRL),
            event(Key::K, Modifiers::CTRL | Modifiers::SHIFT),
        ];
        assert_eq!(
            events.iter().find_map(KeyBinding::from_event),
            Some(KeyBinding::new(Key::K, true, true, false))
        );
    }

    #[test]
    fn overrides_roundtrip_and_conflicts_do_not_replace_existing_bindings() {
        let mut settings = ShortcutSettings::default();
        let binding = KeyBinding::new(Key::K, true, false, false);
        settings
            .assign(Command::Search, vec![binding.clone()])
            .unwrap();
        assert_eq!(
            settings.assign(Command::Settings, vec![binding.clone()]),
            Err(Command::Search)
        );
        settings.assign(Command::NextSlide, vec![]).unwrap();
        let restored: ShortcutSettings =
            serde_json::from_str(&serde_json::to_string(&settings).unwrap()).unwrap();
        assert_eq!(restored.bindings(Command::Search), vec![binding]);
        assert!(restored.bindings(Command::NextSlide).is_empty());
        assert!(!KeyBinding::new(Key::B, false, false, false).matches(Key::B, Modifiers::CTRL));
    }

    #[test]
    fn supports_multiple_bindings_individual_removal_and_legacy_preferences() {
        let mut settings = ShortcutSettings::default();
        let binding = KeyBinding::new(Key::K, true, false, false);
        settings
            .add(Command::Search, binding.clone(), None)
            .unwrap();
        assert_eq!(settings.bindings(Command::Search).len(), 2);
        settings.remove(Command::Search, 0);
        assert_eq!(settings.bindings(Command::Search), vec![binding]);
        let legacy =
            r#"{"search":{"key":"F","ctrl":true,"shift":false,"alt":false},"next_slide":null}"#;
        let restored: ShortcutSettings = serde_json::from_str(legacy).unwrap();
        assert_eq!(restored.bindings(Command::Search).len(), 1);
        assert!(restored.bindings(Command::NextSlide).is_empty());
        let restored: ShortcutSettings =
            serde_json::from_str(&serde_json::to_string(&settings).unwrap()).unwrap();
        assert_eq!(restored, settings);
    }
}
