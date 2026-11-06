#[derive(Clone, Debug, PartialEq)]
pub struct SetupWizardState {
    pub step: usize,
    pub wrap_text: bool,
    pub auto_fit: bool,
    pub presentation_aspect_ratio: f32,
    pub dock_preview_bg: bool,
}

impl Default for SetupWizardState {
    fn default() -> Self {
        Self {
            step: 0,
            wrap_text: true,
            auto_fit: true,
            presentation_aspect_ratio: 16.0 / 9.0,
            dock_preview_bg: false,
        }
    }
}

