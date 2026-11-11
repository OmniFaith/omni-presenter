use eframe::egui::{self, Color32, Frame, Margin, Stroke};

/// Design Tokens & Theme system for the application UI
#[derive(Debug, Clone, PartialEq)]
pub struct AppTheme {
    // Window & Panels
    pub bg_app: Color32,
    pub bg_panel: Color32,
    pub bg_card: Color32,
    pub bg_card_active: Color32,
    pub bg_card_hover: Color32,
    pub bg_pill: Color32,
    pub bg_pill_active: Color32,

    // Borders & Dividers
    pub border_subtle: Color32,
    pub border_default: Color32,
    pub border_active: Color32,
    pub splitter_bg: Color32,
    pub splitter_grip: Color32,
    pub splitter_grip_hover: Color32,

    // Text Colors
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub text_on_accent: Color32,

    // Icons
    pub icon_default: Color32,
    pub icon_active: Color32,
    pub icon_muted: Color32,

    // Accents & Status Colors
    pub accent: Color32,
    pub accent_soft: Color32,
    pub status_success_bg: Color32,
    pub status_success_fg: Color32,
    pub status_warning_bg: Color32,
    pub status_warning_fg: Color32,
    pub status_danger: Color32,

    // Radii
    pub radius_sm: f32,
    pub radius_md: f32,
    pub radius_lg: f32,
}

impl Default for AppTheme {
    fn default() -> Self {
        Self::light()
    }
}

impl AppTheme {
    pub fn color_values(&self) -> std::collections::BTreeMap<String, [u8;4]> {
        let mut values = std::collections::BTreeMap::new();
        macro_rules! colors { ($($field:ident),*) => { $(values.insert(stringify!($field).into(), self.$field.to_srgba_unmultiplied());)* }; }
        colors!(bg_app, bg_panel, bg_card, bg_card_active, bg_card_hover, bg_pill, bg_pill_active,
            border_subtle, border_default, border_active, splitter_bg, splitter_grip, splitter_grip_hover,
            text_primary, text_secondary, text_muted, text_on_accent, icon_default, icon_active, icon_muted,
            accent, accent_soft, status_success_bg, status_success_fg, status_warning_bg, status_warning_fg, status_danger);
        values
    }
    pub fn from_definition(definition: &crate::core::interface_theme::InterfaceTheme, dark: bool) -> Self {
        if definition.id == "classic" {
            return if dark { Self::dark() } else { Self::light() };
        }
        let mut theme = if dark { Self::dark() } else { Self::light() };
        let palette = if dark { &definition.dark } else { &definition.light };
        macro_rules! colors { ($($field:ident),* $(,)?) => { $(
            if let Some(color) = palette.get(stringify!($field)).and_then(|value| crate::core::interface_theme::parse_color(value)) {
                theme.$field = Color32::from_rgba_unmultiplied(color[0], color[1], color[2], color[3]);
            }
        )* }; }
        colors!(bg_app, bg_panel, bg_card, bg_card_active, bg_card_hover, bg_pill, bg_pill_active,
            border_subtle, border_default, border_active, splitter_bg, splitter_grip, splitter_grip_hover,
            text_primary, text_secondary, text_muted, text_on_accent, icon_default, icon_active, icon_muted,
            accent, accent_soft, status_success_bg, status_success_fg, status_warning_bg, status_warning_fg, status_danger);
        if palette.contains_key("accent") && !palette.contains_key("accent_soft") { theme.accent_soft = theme.bg_panel.lerp_to_gamma(theme.accent, if dark { 0.18 } else { 0.1 }); }
        theme.radius_sm = (definition.rounding - 2.0).max(0.0);
        theme.radius_md = definition.rounding;
        theme.radius_lg = (definition.rounding + 2.0).min(24.0);
        theme
    }

    /// Resolves the effective `AppTheme` and dark-mode boolean for the active operator from settings.
    pub fn resolve(
        settings: &crate::core::preferences::AppSettings,
        ctx: &egui::Context,
    ) -> (Self, bool) {
        let p = settings.operator();
        let dark = p
            .theme_mode
            .is_dark(ctx.system_theme() == Some(egui::Theme::Dark), p.dark_mode);
        let target_theme_id = if p.interface_theme_id == "midnight" {
            "darker_pro"
        } else {
            p.interface_theme_id.as_str()
        };
        let definition = settings
            .interface_themes
            .iter()
            .find(|theme| theme.id == target_theme_id)
            .cloned()
            .or_else(|| crate::core::interface_theme::resolve_builtin(target_theme_id))
            .or_else(|| {
                settings
                    .plugins
                    .iter()
                    .filter(|plugin| plugin.enabled)
                    .flat_map(|plugin| &plugin.package.interface_themes)
                    .find(|theme| theme.id == target_theme_id)
                    .cloned()
            })
            .unwrap_or_default();
        (Self::from_definition(&definition, dark), dark)
    }
    /// Modern clean light theme aesthetic matching target design
    pub fn light() -> Self {
        Self {
            bg_app: Color32::from_rgb(242, 245, 250),
            bg_panel: Color32::WHITE,
            bg_card: Color32::WHITE,
            bg_card_active: Color32::from_rgb(232, 239, 252),
            bg_card_hover: Color32::from_rgb(248, 249, 252),
            bg_pill: Color32::from_rgb(243, 245, 248),
            bg_pill_active: Color32::from_rgb(228, 232, 240),

            border_subtle: Color32::from_rgb(240, 242, 246),
            border_default: Color32::from_rgb(235, 238, 244),
            border_active: Color32::from_rgb(205, 212, 222),
            splitter_bg: Color32::from_rgb(250, 251, 253),
            splitter_grip: Color32::from_rgb(215, 220, 228),
            splitter_grip_hover: Color32::from_rgb(140, 150, 165),

            text_primary: Color32::from_rgb(20, 24, 33),
            text_secondary: Color32::from_rgb(100, 112, 132),
            text_muted: Color32::from_rgb(128, 140, 158),
            text_on_accent: Color32::WHITE,

            icon_default: Color32::from_rgb(75, 85, 99),
            icon_active: Color32::from_rgb(20, 24, 33),
            icon_muted: Color32::from_rgb(133, 145, 164),

            accent: Color32::from_rgb(49, 102, 218),
            accent_soft: Color32::from_rgb(235, 242, 252),
            status_success_bg: Color32::from_rgb(236, 253, 245),
            status_success_fg: Color32::from_rgb(5, 150, 105),
            status_warning_bg: Color32::from_rgb(254, 243, 199),
            status_warning_fg: Color32::from_rgb(180, 83, 9),
            status_danger: Color32::from_rgb(220, 38, 38),

            radius_sm: 6.0,
            radius_md: 8.0,
            radius_lg: 10.0,
        }
    }

    /// Dark theme alternative
    pub fn dark() -> Self {
        Self {
            bg_app: Color32::from_rgb(18, 20, 24),
            bg_panel: Color32::from_rgb(22, 24, 30),
            bg_card: Color32::from_rgb(24, 27, 35),
            bg_card_active: Color32::from_rgb(34, 40, 52),
            bg_card_hover: Color32::from_rgb(30, 34, 44),
            bg_pill: Color32::from_rgb(30, 34, 44),
            bg_pill_active: Color32::from_rgb(45, 52, 68),

            border_subtle: Color32::from_rgb(34, 38, 48),
            border_default: Color32::from_rgb(42, 46, 58),
            border_active: Color32::from_rgb(60, 68, 86),
            splitter_bg: Color32::from_rgb(18, 20, 24),
            splitter_grip: Color32::from_rgb(45, 50, 62),
            splitter_grip_hover: Color32::from_rgb(120, 130, 150),

            text_primary: Color32::from_rgb(240, 245, 255),
            text_secondary: Color32::from_rgb(140, 145, 160),
            text_muted: Color32::from_rgb(100, 105, 120),
            text_on_accent: Color32::WHITE,

            icon_default: Color32::from_rgb(160, 165, 180),
            icon_active: Color32::from_rgb(132, 171, 255),
            icon_muted: Color32::from_rgb(110, 115, 130),

            accent: Color32::from_rgb(100, 149, 250),
            accent_soft: Color32::from_rgb(32, 47, 74),
            status_success_bg: Color32::from_rgb(15, 65, 35),
            status_success_fg: Color32::from_rgb(60, 240, 140),
            status_warning_bg: Color32::from_rgb(75, 48, 15),
            status_warning_fg: Color32::from_rgb(255, 185, 70),
            status_danger: Color32::from_rgb(240, 80, 80),

            radius_sm: 6.0,
            radius_md: 8.0,
            radius_lg: 10.0,
        }
    }

    /// Generates frame for outer container columns
    pub fn container_frame(&self) -> Frame {
        Frame::new()
            .fill(self.bg_card)
            .stroke(Stroke::new(1.0, self.border_default))
            .corner_radius(self.radius_lg)
            .inner_margin(Margin::symmetric(10, 8))
    }

    /// Generates frame for interactive item cards/rows
    pub fn card_frame(&self, is_active: bool) -> Frame {
        let bg = if is_active {
            self.bg_card_active
        } else {
            Color32::TRANSPARENT
        };
        Frame::new()
            .fill(bg)
            .stroke(Stroke::NONE)
            .corner_radius(self.radius_md)
            .inner_margin(Margin::symmetric(9, 6))
    }

    /// Configures egui::Visuals according to this theme
    pub fn apply_to_visuals(&self, visuals: &mut egui::Visuals) {
        visuals.faint_bg_color = self.bg_pill;
        visuals.window_fill = self.bg_card;
        visuals.panel_fill = self.bg_panel;
        visuals.extreme_bg_color = self.bg_app;
        visuals.override_text_color = Some(self.text_primary);
        visuals.widgets.noninteractive.bg_fill = self.bg_card;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, self.border_default);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, self.text_primary);
        visuals.widgets.inactive.bg_fill = self.bg_pill;
        visuals.widgets.inactive.weak_bg_fill = self.bg_pill;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, self.border_default);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, self.text_primary);
        visuals.widgets.hovered.bg_fill = self.bg_card_hover;
        visuals.widgets.hovered.weak_bg_fill = self.bg_card_hover;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, self.border_active);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, self.text_primary);
        visuals.widgets.active.bg_fill = self.bg_pill_active;
        visuals.widgets.active.weak_bg_fill = self.bg_pill_active;
        visuals.widgets.active.bg_stroke = Stroke::new(1.0, self.accent);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, self.text_primary);
        visuals.selection.bg_fill = self.accent_soft;
        visuals.selection.stroke = Stroke::new(1.0, self.accent);

        // Modern context menu and popup visual styling matching target design
        visuals.menu_corner_radius = egui::CornerRadius::same(8);
        visuals.window_corner_radius = egui::CornerRadius::same(8);
        visuals.window_stroke = Stroke::new(1.0, self.border_default);
        visuals.popup_shadow = egui::epaint::Shadow {
            offset: [0, 4],
            blur: 16,
            spread: 0,
            color: Color32::from_black_alpha(20),
        };
        visuals.window_shadow = egui::epaint::Shadow {
            offset: [0, 4],
            blur: 16,
            spread: 0,
            color: Color32::from_black_alpha(20),
        };
    }

    pub fn icon_button(
        &self,
        ui: &mut egui::Ui,
        icon: &str,
        tooltip: &str,
        active: bool,
        size: egui::Vec2,
    ) -> egui::Response {
        let enabled = ui.is_enabled();
        let (rect, response) = ui.allocate_exact_size(
            size,
            if enabled {
                egui::Sense::click()
            } else {
                egui::Sense::hover()
            },
        );
        let hover = if enabled {
            ui.ctx()
                .animate_bool(response.id.with("hover"), response.hovered())
        } else {
            0.0
        };
        let selected = ui.ctx().animate_bool(response.id.with("selected"), active);
        let press = if enabled {
            ui.ctx().animate_bool(
                response.id.with("press"),
                response.is_pointer_button_down_on(),
            )
        } else {
            0.0
        };
        let bg = if !enabled {
            self.bg_panel.linear_multiply(0.4)
        } else if press > 0.05 {
            self.bg_card_hover
                .lerp_to_gamma(self.accent_soft, press * 0.45)
        } else {
            self.bg_panel
                .lerp_to_gamma(self.bg_card_hover, hover)
                .lerp_to_gamma(self.accent_soft, selected)
        };
        let stroke = if !enabled {
            egui::Stroke::NONE
        } else if press > 0.05 {
            egui::Stroke::new(1.0, self.accent.linear_multiply(0.75))
        } else if selected > 0.05 {
            egui::Stroke::new(1.0, self.accent.linear_multiply(0.55))
        } else if hover > 0.05 {
            egui::Stroke::new(
                1.0,
                self.border_subtle
                    .lerp_to_gamma(self.accent.linear_multiply(0.4), hover),
            )
        } else {
            egui::Stroke::NONE
        };
        let draw_rect = rect.shrink(press * 0.75);
        let radius = if size.y <= 28.0 {
            self.radius_sm
        } else {
            self.radius_md
        };
        ui.painter()
            .rect(draw_rect, radius, bg, stroke, egui::StrokeKind::Inside);
        let color = if !enabled {
            self.text_muted.linear_multiply(0.45)
        } else if press > 0.05 {
            self.accent
        } else {
            self.icon_default
                .lerp_to_gamma(self.accent, selected.max(hover * 0.7))
        };
        ui.painter().text(
            draw_rect.center(),
            egui::Align2::CENTER_CENTER,
            icon,
            egui::FontId::proportional(if size.x > 32.0 { 20.0 } else { 16.0 }),
            color,
        );
        let resp = if enabled {
            response.on_hover_cursor(egui::CursorIcon::PointingHand)
        } else {
            response
        };
        resp.on_hover_text(crate::ui::i18n::language(ui.ctx()).text(tooltip))
    }
}
