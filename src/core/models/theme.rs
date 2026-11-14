use eframe::egui::Color32;

/// Representation of a Presentation Theme with deep visual controls
#[derive(Debug, Clone, PartialEq)]
pub struct PresentationTheme {
    pub id: String,
    pub name: String,

    // Plano de Fundo: "color", "gradient", "image", "video"
    pub bg_type: String,
    pub bg_top: Color32,
    pub bg_bottom: Color32,
    pub bg_image_path: String,

    // Transição: "fade", "slide_left", "slide_right", "slide_up", "slide_down", "cut"
    pub transition: String,

    // Tipografia & Formatação
    pub font_family: String, // "Montserrat", "Inter", "Roboto", "Proportional", "Serif", "Monospace"
    pub font_size: f32,      // Tamanho base de fonte
    pub font_weight: u16, // 100..=900 (100 Thin, 300 Light, 400 Regular, 500 Medium, 600 SemiBold, 700 Bold, 800 ExtraBold, 900 Black)
    pub font_bold: bool,
    pub font_italic: bool,
    pub font_underline: bool,
    pub font_strikethrough: bool,
    pub text_color: Color32,
    pub text_case: String, // "original", "uppercase", "lowercase", "titlecase"
    pub line_spacing: f32, // Multiplicador de entrelinha (ex: 1.35)
    pub letter_spacing: f32, // Espaçamento entre letras
    pub align_h: String,   // "left", "center", "right", "justify"
    pub align_v: String,   // "top", "center", "bottom"

    // Contorno do texto (Outline / Stroke)
    pub outline_enabled: bool,
    pub outline_color: Color32,
    pub outline_width: f32,

    // Sombra do texto
    pub shadow_enabled: bool,
    pub shadow_color: Color32,
    pub shadow_blur: f32,
    pub shadow_offset_x: f32,
    pub shadow_offset_y: f32,

    // Bounding Box interativo do texto (coordenadas normalizadas 0.0 a 1.0)
    pub box_x: f32,
    pub box_y: f32,
    pub box_w: f32,
    pub box_h: f32,

    // Configurações da Caixa de Texto
    pub box_bg_enabled: bool,
    pub box_bg_color: Color32,
    pub box_border_enabled: bool,
    pub box_border_color: Color32,
    pub box_border_width: f32,
    pub box_radius: f32,
    pub box_padding: f32,

    // Bandeiras auxiliares
    pub is_gradient: bool,
    pub is_animated: bool,
    pub is_favorite: bool,
    pub sort_order: i32,
    pub options: super::theme_options::ThemeOptions,
}

impl Default for PresentationTheme {
    fn default() -> Self {
        Self {
            id: "default_theme".to_owned(),
            name: "services-new-theme".to_owned(),
            bg_type: "gradient".to_owned(),
            bg_top: Color32::from_rgb(17, 24, 39),
            bg_bottom: Color32::from_rgb(3, 7, 18),
            bg_image_path: String::new(),
            transition: "fade".to_owned(),
            font_family: "Montserrat".to_owned(),
            font_size: 44.0,
            font_weight: 700,
            font_bold: true,
            font_italic: false,
            font_underline: false,
            font_strikethrough: false,
            text_color: Color32::WHITE,
            text_case: "original".to_owned(),
            line_spacing: 1.36,
            letter_spacing: 0.6,
            align_h: "center".to_owned(),
            align_v: "center".to_owned(),
            outline_enabled: false,
            outline_color: Color32::from_black_alpha(220),
            outline_width: 2.5,
            shadow_enabled: true,
            shadow_color: Color32::from_black_alpha(220),
            shadow_blur: 10.0,
            shadow_offset_x: 0.0,
            shadow_offset_y: 3.0,
            box_x: 0.12,
            box_y: 0.18,
            box_w: 0.76,
            box_h: 0.64,
            box_bg_enabled: false,
            box_bg_color: Color32::from_black_alpha(130),
            box_border_enabled: false,
            box_border_color: Color32::from_white_alpha(120),
            box_border_width: 1.5,
            box_radius: 8.0,
            box_padding: 16.0,
            is_gradient: true,
            is_animated: false,
            is_favorite: false,
            sort_order: 0,
            options: super::theme_options::ThemeOptions {
                gradient_angle: 145.0,
                emphasis_color: [250, 204, 21],
                ..Default::default()
            },
        }
    }
}

impl PresentationTheme {
    /// Conjunto de temas padrão ricos prontos para projeção
    pub fn default_themes() -> Vec<Self> {
        vec![
            Self {
                id: "obsidian_pro".to_owned(),
                name: "theme-obsidian-pro".to_owned(),
                bg_type: "gradient".to_owned(),
                bg_top: Color32::from_rgb(26, 29, 38),
                bg_bottom: Color32::from_rgb(11, 12, 16),
                bg_image_path: String::new(),
                transition: "fade".to_owned(),
                font_family: "Inter".to_owned(),
                font_size: 48.0,
                font_weight: 800,
                font_bold: true,
                font_italic: false,
                font_underline: false,
                font_strikethrough: false,
                text_color: Color32::WHITE,
                text_case: "uppercase".to_owned(),
                line_spacing: 1.36,
                letter_spacing: 2.2,
                align_h: "center".to_owned(),
                align_v: "center".to_owned(),
                outline_enabled: false,
                outline_color: Color32::from_black_alpha(220),
                outline_width: 2.5,
                shadow_enabled: true,
                shadow_color: Color32::from_black_alpha(220),
                shadow_blur: 12.0,
                shadow_offset_x: 0.0,
                shadow_offset_y: 3.0,
                box_x: 0.12,
                box_y: 0.18,
                box_w: 0.76,
                box_h: 0.64,
                box_bg_enabled: false,
                box_bg_color: Color32::from_black_alpha(130),
                box_border_enabled: false,
                box_border_color: Color32::from_white_alpha(120),
                box_border_width: 1.5,
                box_radius: 8.0,
                box_padding: 16.0,
                is_gradient: true,
                is_animated: false,
                is_favorite: false,
                sort_order: 20,
                options: super::theme_options::ThemeOptions {
                    gradient_angle: 145.0,
                    emphasis_color: [250, 204, 21],
                    ..Default::default()
                },
            },
            Self {
                id: "midnight_sapphire".to_owned(),
                name: "theme-midnight-sapphire".to_owned(),
                bg_type: "gradient".to_owned(),
                bg_top: Color32::from_rgb(10, 28, 60),
                bg_bottom: Color32::from_rgb(14, 114, 138),
                bg_image_path: String::new(),
                transition: "fade".to_owned(),
                font_family: "Montserrat".to_owned(),
                font_size: 48.0,
                font_weight: 700,
                font_bold: true,
                font_italic: false,
                font_underline: false,
                font_strikethrough: false,
                text_color: Color32::WHITE,
                text_case: "uppercase".to_owned(),
                line_spacing: 1.36,
                letter_spacing: 1.8,
                align_h: "center".to_owned(),
                align_v: "center".to_owned(),
                outline_enabled: false,
                outline_color: Color32::from_black_alpha(200),
                outline_width: 2.5,
                shadow_enabled: true,
                shadow_color: Color32::from_black_alpha(200),
                shadow_blur: 10.0,
                shadow_offset_x: 0.0,
                shadow_offset_y: 2.5,
                box_x: 0.12,
                box_y: 0.18,
                box_w: 0.76,
                box_h: 0.64,
                box_bg_enabled: false,
                box_bg_color: Color32::from_black_alpha(130),
                box_border_enabled: false,
                box_border_color: Color32::from_white_alpha(120),
                box_border_width: 1.5,
                box_radius: 8.0,
                box_padding: 16.0,
                is_gradient: true,
                is_animated: false,
                is_favorite: false,
                sort_order: 22,
                options: super::theme_options::ThemeOptions {
                    gradient_angle: 135.0,
                    emphasis_color: [56, 189, 248],
                    ..Default::default()
                },
            },
            Self {
                id: "golden_champagne".to_owned(),
                name: "theme-golden-champagne".to_owned(),
                bg_type: "gradient".to_owned(),
                bg_top: Color32::from_rgb(186, 131, 84),
                bg_bottom: Color32::from_rgb(245, 227, 207),
                bg_image_path: String::new(),
                transition: "fade".to_owned(),
                font_family: "Montserrat".to_owned(),
                font_size: 48.0,
                font_weight: 700,
                font_bold: true,
                font_italic: false,
                font_underline: false,
                font_strikethrough: false,
                text_color: Color32::WHITE,
                text_case: "uppercase".to_owned(),
                line_spacing: 1.35,
                letter_spacing: 1.8,
                align_h: "center".to_owned(),
                align_v: "center".to_owned(),
                outline_enabled: false,
                outline_color: Color32::from_black_alpha(220),
                outline_width: 2.5,
                shadow_enabled: true,
                shadow_color: Color32::from_rgba_unmultiplied(65, 30, 10, 160),
                shadow_blur: 8.0,
                shadow_offset_x: 0.0,
                shadow_offset_y: 2.0,
                box_x: 0.12,
                box_y: 0.18,
                box_w: 0.76,
                box_h: 0.64,
                box_bg_enabled: false,
                box_bg_color: Color32::from_black_alpha(130),
                box_border_enabled: false,
                box_border_color: Color32::from_white_alpha(120),
                box_border_width: 1.5,
                box_radius: 8.0,
                box_padding: 16.0,
                is_gradient: true,
                is_animated: false,
                is_favorite: false,
                sort_order: 21,
                options: super::theme_options::ThemeOptions {
                    gradient_angle: 135.0,
                    emphasis_color: [254, 240, 138],
                    ..Default::default()
                },
            },
            Self {
                id: "warm_ember".to_owned(),
                name: "theme-warm-ember".to_owned(),
                bg_type: "gradient".to_owned(),
                bg_top: Color32::from_rgb(168, 68, 44),
                bg_bottom: Color32::from_rgb(230, 164, 110),
                bg_image_path: String::new(),
                transition: "fade".to_owned(),
                font_family: "Montserrat".to_owned(),
                font_size: 48.0,
                font_weight: 700,
                font_bold: true,
                font_italic: false,
                font_underline: false,
                font_strikethrough: false,
                text_color: Color32::WHITE,
                text_case: "uppercase".to_owned(),
                line_spacing: 1.35,
                letter_spacing: 1.8,
                align_h: "center".to_owned(),
                align_v: "center".to_owned(),
                outline_enabled: false,
                outline_color: Color32::from_black_alpha(220),
                outline_width: 2.5,
                shadow_enabled: true,
                shadow_color: Color32::from_rgba_unmultiplied(50, 15, 10, 170),
                shadow_blur: 8.0,
                shadow_offset_x: 0.0,
                shadow_offset_y: 2.0,
                box_x: 0.12,
                box_y: 0.18,
                box_w: 0.76,
                box_h: 0.64,
                box_bg_enabled: false,
                box_bg_color: Color32::from_black_alpha(130),
                box_border_enabled: false,
                box_border_color: Color32::from_white_alpha(120),
                box_border_width: 1.5,
                box_radius: 8.0,
                box_padding: 16.0,
                is_gradient: true,
                is_animated: false,
                is_favorite: false,
                sort_order: 23,
                options: super::theme_options::ThemeOptions {
                    gradient_angle: 140.0,
                    emphasis_color: [251, 191, 36],
                    ..Default::default()
                },
            },
            Self {
                id: "nordic_sage".to_owned(),
                name: "theme-nordic-sage".to_owned(),
                bg_type: "gradient".to_owned(),
                bg_top: Color32::from_rgb(60, 85, 72),
                bg_bottom: Color32::from_rgb(148, 170, 156),
                bg_image_path: String::new(),
                transition: "fade".to_owned(),
                font_family: "Montserrat".to_owned(),
                font_size: 48.0,
                font_weight: 700,
                font_bold: true,
                font_italic: false,
                font_underline: false,
                font_strikethrough: false,
                text_color: Color32::WHITE,
                text_case: "uppercase".to_owned(),
                line_spacing: 1.36,
                letter_spacing: 1.8,
                align_h: "center".to_owned(),
                align_v: "center".to_owned(),
                outline_enabled: false,
                outline_color: Color32::from_black_alpha(200),
                outline_width: 2.5,
                shadow_enabled: true,
                shadow_color: Color32::from_rgba_unmultiplied(20, 35, 25, 160),
                shadow_blur: 8.0,
                shadow_offset_x: 0.0,
                shadow_offset_y: 2.0,
                box_x: 0.12,
                box_y: 0.18,
                box_w: 0.76,
                box_h: 0.64,
                box_bg_enabled: false,
                box_bg_color: Color32::from_black_alpha(130),
                box_border_enabled: false,
                box_border_color: Color32::from_white_alpha(120),
                box_border_width: 1.5,
                box_radius: 8.0,
                box_padding: 16.0,
                is_gradient: true,
                is_animated: false,
                is_favorite: false,
                sort_order: 25,
                options: super::theme_options::ThemeOptions {
                    gradient_angle: 150.0,
                    emphasis_color: [253, 230, 138],
                    ..Default::default()
                },
            },
            Self {
                id: "clean_alabaster".to_owned(),
                name: "theme-clean-alabaster".to_owned(),
                bg_type: "gradient".to_owned(),
                bg_top: Color32::from_rgb(246, 241, 233),
                bg_bottom: Color32::from_rgb(230, 220, 206),
                bg_image_path: String::new(),
                transition: "fade".to_owned(),
                font_family: "Montserrat".to_owned(),
                font_size: 48.0,
                font_weight: 700,
                font_bold: true,
                font_italic: false,
                font_underline: false,
                font_strikethrough: false,
                text_color: Color32::from_rgb(24, 28, 36),
                text_case: "uppercase".to_owned(),
                line_spacing: 1.38,
                letter_spacing: 1.8,
                align_h: "center".to_owned(),
                align_v: "center".to_owned(),
                outline_enabled: false,
                outline_color: Color32::TRANSPARENT,
                outline_width: 0.0,
                shadow_enabled: false,
                shadow_color: Color32::TRANSPARENT,
                shadow_blur: 0.0,
                shadow_offset_x: 0.0,
                shadow_offset_y: 0.0,
                box_x: 0.12,
                box_y: 0.18,
                box_w: 0.76,
                box_h: 0.64,
                box_bg_enabled: false,
                box_bg_color: Color32::from_black_alpha(130),
                box_border_enabled: false,
                box_border_color: Color32::from_white_alpha(120),
                box_border_width: 1.5,
                box_radius: 8.0,
                box_padding: 16.0,
                is_gradient: true,
                is_animated: false,
                is_favorite: false,
                sort_order: 27,
                options: super::theme_options::ThemeOptions {
                    gradient_angle: 160.0,
                    emphasis_color: [180, 83, 9],
                    ..Default::default()
                },
            },
            Self {
                id: "bold_anthem".to_owned(),
                name: "theme-bold-anthem".to_owned(),
                bg_type: "gradient".to_owned(),
                bg_top: Color32::from_rgb(22, 27, 36),
                bg_bottom: Color32::from_rgb(10, 12, 17),
                bg_image_path: String::new(),
                transition: "fade".to_owned(),
                font_family: "Oswald".to_owned(),
                font_size: 52.0,
                font_weight: 800,
                font_bold: true,
                font_italic: false,
                font_underline: false,
                font_strikethrough: false,
                text_color: Color32::WHITE,
                text_case: "uppercase".to_owned(),
                line_spacing: 1.26,
                letter_spacing: 2.4,
                align_h: "center".to_owned(),
                align_v: "center".to_owned(),
                outline_enabled: false,
                outline_color: Color32::from_black_alpha(220),
                outline_width: 2.5,
                shadow_enabled: true,
                shadow_color: Color32::from_black_alpha(230),
                shadow_blur: 12.0,
                shadow_offset_x: 0.0,
                shadow_offset_y: 3.0,
                box_x: 0.12,
                box_y: 0.18,
                box_w: 0.76,
                box_h: 0.64,
                box_bg_enabled: false,
                box_bg_color: Color32::from_black_alpha(130),
                box_border_enabled: false,
                box_border_color: Color32::from_white_alpha(120),
                box_border_width: 1.5,
                box_radius: 8.0,
                box_padding: 16.0,
                is_gradient: true,
                is_animated: false,
                is_favorite: false,
                sort_order: 28,
                options: super::theme_options::ThemeOptions {
                    gradient_angle: 130.0,
                    emphasis_color: [252, 211, 77],
                    ..Default::default()
                },
            },
            Self {
                id: "frosted_glass_card".to_owned(),
                name: "theme-frosted-glass-card".to_owned(),
                bg_type: "gradient".to_owned(),
                bg_top: Color32::from_rgb(20, 24, 38),
                bg_bottom: Color32::from_rgb(8, 10, 16),
                bg_image_path: String::new(),
                transition: "fade".to_owned(),
                font_family: "Montserrat".to_owned(),
                font_size: 46.0,
                font_weight: 700,
                font_bold: true,
                font_italic: false,
                font_underline: false,
                font_strikethrough: false,
                text_color: Color32::WHITE,
                text_case: "uppercase".to_owned(),
                line_spacing: 1.36,
                letter_spacing: 1.6,
                align_h: "center".to_owned(),
                align_v: "center".to_owned(),
                outline_enabled: false,
                outline_color: Color32::from_black_alpha(200),
                outline_width: 2.5,
                shadow_enabled: true,
                shadow_color: Color32::from_black_alpha(200),
                shadow_blur: 8.0,
                shadow_offset_x: 0.0,
                shadow_offset_y: 2.5,
                box_x: 0.08,
                box_y: 0.14,
                box_w: 0.84,
                box_h: 0.72,
                box_bg_enabled: true,
                box_bg_color: Color32::from_rgba_unmultiplied(255, 255, 255, 18),
                box_border_enabled: true,
                box_border_color: Color32::from_white_alpha(50),
                box_border_width: 1.2,
                box_radius: 16.0,
                box_padding: 24.0,
                is_gradient: true,
                is_animated: false,
                is_favorite: false,
                sort_order: 30,
                options: super::theme_options::ThemeOptions {
                    gradient_angle: 135.0,
                    ..Default::default()
                },
            },
            Self {
                id: "broadcast_lower_third".to_owned(),
                name: "theme-broadcast-lower-third".to_owned(),
                bg_type: "color".to_owned(),
                bg_top: Color32::TRANSPARENT,
                bg_bottom: Color32::TRANSPARENT,
                bg_image_path: String::new(),
                transition: "fade".to_owned(),
                font_family: "Montserrat".to_owned(),
                font_size: 34.0,
                font_weight: 700,
                font_bold: true,
                font_italic: false,
                font_underline: false,
                font_strikethrough: false,
                text_color: Color32::WHITE,
                text_case: "original".to_owned(),
                line_spacing: 1.32,
                letter_spacing: 0.6,
                align_h: "center".to_owned(),
                align_v: "center".to_owned(),
                outline_enabled: false,
                outline_color: Color32::from_black_alpha(200),
                outline_width: 2.0,
                shadow_enabled: true,
                shadow_color: Color32::from_black_alpha(180),
                shadow_blur: 6.0,
                shadow_offset_x: 0.0,
                shadow_offset_y: 2.0,
                box_x: 0.05,
                box_y: 0.74,
                box_w: 0.90,
                box_h: 0.20,
                box_bg_enabled: true,
                box_bg_color: Color32::from_rgba_unmultiplied(12, 17, 28, 220),
                box_border_enabled: true,
                box_border_color: Color32::from_white_alpha(45),
                box_border_width: 1.2,
                box_radius: 12.0,
                box_padding: 14.0,
                is_gradient: false,
                is_animated: false,
                is_favorite: false,
                sort_order: 31,
                options: Default::default(),
            },
        ]
    }

    /// Aplica a transformação de caso (MAIÚSCULAS, minúsculas, título, original) ao texto
    pub fn transform_text(&self, text: &str) -> String {
        match self.text_case.as_str() {
            "uppercase" => text.to_uppercase(),
            "lowercase" => text.to_lowercase(),
            "titlecase" => {
                let mut out = String::with_capacity(text.len());
                for line in text.lines() {
                    let mut prev_whitespace = true;
                    for c in line.chars() {
                        if c.is_whitespace() {
                            prev_whitespace = true;
                            out.push(c);
                        } else if prev_whitespace {
                            out.extend(c.to_uppercase());
                            prev_whitespace = false;
                        } else {
                            out.extend(c.to_lowercase());
                        }
                    }
                    out.push('\n');
                }
                if out.ends_with('\n') && !text.ends_with('\n') {
                    out.pop();
                }
                out
            }
            _ => text.to_string(),
        }
    }

    /// Retorna o nome internacionalizado do tema de acordo com o idioma fornecido
    pub fn display_name(&self, language: crate::core::i18n::Language) -> String {
        // 1. Shaders / temas animados
        let shader_kind = self
            .id
            .strip_prefix("shader_")
            .or_else(|| {
                if self.bg_type == "shader" && !self.options.shader.kind.is_empty() {
                    Some(self.options.shader.kind.as_str())
                } else {
                    None
                }
            })
            .or_else(|| match self.name.as_str() {
                "Aurora · Dinâmico" | "Aurora · Animated" | "Aurora · Animado" => Some("aurora"),
                "Nebulosa · Dinâmico" | "Nebula · Animated" | "Nebulosa · Animado" => Some("nebula"),
                "Ondas · Dinâmico" | "Waves · Animated" | "Olas · Animado" => Some("waves"),
                "Seda · Dinâmico" | "Silk · Animated" | "Seda · Animado" => Some("silk"),
                "Partículas · Dinâmico" | "Particles · Animated" | "Partículas · Animado" => Some("particles"),
                "Anéis · Dinâmico" | "Rings · Animated" | "Anillos · Animado" => Some("rings"),
                "Fluido · Dinâmico" | "Fluid · Animated" | "Fluido · Animado" => Some("fluid"),
                "Grade de Luz · Dinâmico" | "Grade de luz · Dinâmico" | "Light grid · Animated" | "Light Grid · Animated" | "Cuadrícula de luz · Animado" => Some("grid"),
                "Bokeh · Dinâmico" | "Bokeh · Animated" | "Bokeh · Animado" => Some("bokeh"),
                "Prisma · Dinâmico" | "Prism · Animated" | "Prisma · Animado" => Some("prism"),
                _ => None,
            });

        if let Some(kind) = shader_kind {
            let name_key = match kind {
                "aurora" => Some("settings-aurora"),
                "nebula" => Some("settings-nebula"),
                "waves" => Some("settings-waves"),
                "silk" => Some("settings-silk"),
                "particles" => Some("settings-particles"),
                "rings" => Some("settings-rings"),
                "fluid" => Some("settings-fluid"),
                "grid" => Some("settings-light-grid"),
                "bokeh" => Some("settings-bokeh"),
                "prism" => Some("settings-prism"),
                _ => None,
            };
            if let Some(key) = name_key {
                let translated = language.text(key);
                return language.format("services-animated", &[("name", translated)]);
            }
        }

        // 2. Temas padrão estáticos por ID
        match self.id.as_str() {
            "obsidian_pro" => return language.text("theme-obsidian-pro").to_string(),
            "golden_champagne" => return language.text("theme-golden-champagne").to_string(),
            "midnight_sapphire" => return language.text("theme-midnight-sapphire").to_string(),
            "warm_ember" => return language.text("theme-warm-ember").to_string(),
            "deep_emerald" => return language.text("theme-deep-emerald").to_string(),
            "nordic_sage" => return language.text("theme-nordic-sage").to_string(),
            "scripture_serif" => return language.text("theme-scripture-serif").to_string(),
            "celestial_violet" => return language.text("theme-celestial-violet").to_string(),
            "bold_anthem" => return language.text("theme-bold-anthem").to_string(),
            "clean_alabaster" => return language.text("theme-clean-alabaster").to_string(),
            "broadcast_lower_third" => return language.text("theme-broadcast-lower-third").to_string(),
            "frosted_glass_card" => return language.text("theme-frosted-glass-card").to_string(),
            "default_theme" => return language.text("services-new-theme").to_string(),
            _ => {}
        }

        // 3. Compatibilidade com nomes legados armazenados em banco
        match self.name.as_str() {
            "Obsidian Minimalista" | "Obsidian Minimalist" | "Obsidiana Minimalista" => {
                return language.text("theme-obsidian-pro").to_string();
            }
            "Safira da Meia-Noite" | "Midnight Sapphire" | "Zafiro de Medianoche" => {
                return language.text("theme-midnight-sapphire").to_string();
            }
            "Brasas & Bronze" | "Warm Ember" | "Brasas y Bronce" => {
                return language.text("theme-warm-ember").to_string();
            }
            "Esmeralda Profunda" | "Deep Emerald" => {
                return language.text("theme-deep-emerald").to_string();
            }
            "Escrituras & Liturgia" | "Scripture & Liturgy" | "Escrituras y Liturgia" => {
                return language.text("theme-scripture-serif").to_string();
            }
            "Ametista & Crepúsculo" | "Celestial Amethyst" | "Amatista Celestial" => {
                return language.text("theme-celestial-violet").to_string();
            }
            "Louvor Vibrante" | "Bold Anthem" | "Alabanza Vibrante" => {
                return language.text("theme-bold-anthem").to_string();
            }
            "Alabastro Editorial" | "Clean Alabaster" => {
                return language.text("theme-clean-alabaster").to_string();
            }
            "Tarja para Transmissão" | "Broadcast Lower Third" | "Cintillo de Transmisión" => {
                return language.text("theme-broadcast-lower-third").to_string();
            }
            "Vidro Fumê Card" | "Frosted Glass Card" | "Tarjeta de Vidrio Esmerilado" => {
                return language.text("theme-frosted-glass-card").to_string();
            }
            "Novo Tema" | "New theme" | "Nuevo tema" => {
                return language.text("services-new-theme").to_string();
            }
            _ => {}
        }

        // 4. Se o nome for uma chave Fluent válida
        if self.name.starts_with("theme-") || self.name.starts_with("services-") {
            return language.text(&self.name).to_string();
        }
        if let Some(text) = language.lookup(&self.name) {
            return text.to_string();
        }

        // 5. Fallback para nome personalizado existente
        self.name.clone()
    }
}

// ==========================================
// Funções Utilitárias para Conversão de Cores
// ==========================================
pub fn color_to_hex(c: Color32) -> String {
    format!("#{:02X}{:02X}{:02X}{:02X}", c.r(), c.g(), c.b(), c.a())
}

pub fn hex_to_color(s: &str) -> Color32 {
    let s = s.trim().trim_start_matches('#');
    if s.len() == 6 {
        let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(255);
        let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(255);
        let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(255);
        Color32::from_rgb(r, g, b)
    } else if s.len() == 8 {
        let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(255);
        let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(255);
        let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(255);
        let a = u8::from_str_radix(&s[6..8], 16).unwrap_or(255);
        Color32::from_rgba_premultiplied(r, g, b, a)
    } else {
        Color32::WHITE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_case_transformations() {
        let mut theme = PresentationTheme::default();
        let input = "Quem é este que vem de Edom?";

        theme.text_case = "uppercase".to_owned();
        assert_eq!(theme.transform_text(input), "QUEM É ESTE QUE VEM DE EDOM?");

        theme.text_case = "lowercase".to_owned();
        assert_eq!(theme.transform_text(input), "quem é este que vem de edom?");

        theme.text_case = "titlecase".to_owned();
        assert_eq!(theme.transform_text(input), "Quem É Este Que Vem De Edom?");

        theme.text_case = "original".to_owned();
        assert_eq!(theme.transform_text(input), input);
    }

    #[test]
    fn test_hex_color_roundtrip() {
        let color = Color32::from_rgb(59, 130, 246);
        let hex = color_to_hex(color);
        let parsed = hex_to_color(&hex);
        assert_eq!(parsed.r(), color.r());
        assert_eq!(parsed.g(), color.g());
        assert_eq!(parsed.b(), color.b());
    }

    #[test]
    fn test_theme_display_name_internationalization() {
        use crate::core::i18n::Language;

        // Static theme by ID
        let mut theme = PresentationTheme::default();
        theme.id = "obsidian_pro".into();
        theme.name = "theme-obsidian-pro".into();
        assert_eq!(theme.display_name(Language::English), "Obsidian Minimalist");
        assert_eq!(theme.display_name(Language::Portuguese), "Obsidian Minimalista");
        assert_eq!(theme.display_name(Language::Spanish), "Obsidiana Minimalista");

        // Shader theme by ID
        let mut shader = PresentationTheme::default();
        shader.id = "shader_aurora".into();
        shader.name = "Aurora · Dinâmico".into();
        shader.bg_type = "shader".into();
        assert_eq!(shader.display_name(Language::English), "Aurora · Animated");
        assert_eq!(shader.display_name(Language::Portuguese), "Aurora · Dinâmico");
        assert_eq!(shader.display_name(Language::Spanish), "Aurora · Animado");

        // Legacy stored name
        let mut legacy = PresentationTheme::default();
        legacy.id = "custom_123".into();
        legacy.name = "Safira da Meia-Noite".into();
        assert_eq!(legacy.display_name(Language::English), "Deep Ocean & Cyan");
        assert_eq!(legacy.display_name(Language::Portuguese), "Oceano & Ciano");
        assert_eq!(legacy.display_name(Language::Spanish), "Océano & Cian");

        // Custom user-defined name
        let mut custom = PresentationTheme::default();
        custom.id = "my_custom_theme".into();
        custom.name = "Culto de Domingo".into();
        assert_eq!(custom.display_name(Language::English), "Culto de Domingo");
        assert_eq!(custom.display_name(Language::Portuguese), "Culto de Domingo");
        assert_eq!(custom.display_name(Language::Spanish), "Culto de Domingo");
    }
}
