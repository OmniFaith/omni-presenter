//! Dropdown popup menu for inserting breaks, style directives, and formatting tags in the song editor.

use super::actions::{
    apply_color_selection, apply_font_selection, apply_shadow_selection, apply_size_selection,
    insert_text_at_selection, wrap_selection,
};
use crate::core::state::ControlPanelState;
use crate::ui::theme::AppTheme;
use eframe::egui::{self, Color32, RichText, Stroke, StrokeKind, Vec2};
use egui_phosphor::regular::MAGNIFYING_GLASS;

#[derive(Clone, Copy, PartialEq, Eq)]
enum MenuAction {
    Insert(&'static str),
    ApplyFont(&'static str),
    ApplyColor(&'static str),
    ApplySize(&'static str),
    ApplyShadow(&'static str),
    Wrap(&'static str, &'static str),
}

struct MenuItemDef {
    category_id: usize,
    category_title: &'static str,
    code: &'static str,
    label: &'static str,
    action: MenuAction,
}

const MENU_ITEMS: &[MenuItemDef] = &[
    // 1. Comandos de Slide
    MenuItemDef {
        category_id: 1,
        category_title: "Comandos de Slide (//)",
        code: "//fonte: Nome",
        label: "Tipografia do Slide",
        action: MenuAction::Insert("\n//fonte: Montserrat\n"),
    },
    MenuItemDef {
        category_id: 1,
        category_title: "Comandos de Slide (//)",
        code: "//cor: #HEX",
        label: "Cor Base do Texto",
        action: MenuAction::Insert("\n//cor: #FFD700\n"),
    },
    MenuItemDef {
        category_id: 1,
        category_title: "Comandos de Slide (//)",
        code: "//tamanho: N",
        label: "Tamanho da Fonte (pt)",
        action: MenuAction::Insert("\n//tamanho: 46\n"),
    },
    MenuItemDef {
        category_id: 1,
        category_title: "Comandos de Slide (//)",
        code: "//sombra: #HEX",
        label: "Cor da Sombra do Slide",
        action: MenuAction::Insert("\n//sombra: #00000082\n"),
    },
    MenuItemDef {
        category_id: 1,
        category_title: "Comandos de Slide (//)",
        code: "//alinhar: centro",
        label: "Alinhamento (centro|esquerda|direita)",
        action: MenuAction::Insert("\n//alinhar: centro\n"),
    },
    MenuItemDef {
        category_id: 1,
        category_title: "Comandos de Slide (//)",
        code: "//linha: 1.38",
        label: "Espaçamento Entre Linhas",
        action: MenuAction::Insert("\n//linha: 1.38\n"),
    },
    MenuItemDef {
        category_id: 1,
        category_title: "Comandos de Slide (//)",
        code: "//letra: 1.8",
        label: "Espaçamento Entre Letras",
        action: MenuAction::Insert("\n//letra: 1.8\n"),
    },

    // 2. Quebras de Formato
    MenuItemDef {
        category_id: 2,
        category_title: "Quebras de Formato",
        code: "|",
        label: "Quebra Suave Padrão",
        action: MenuAction::Insert(" | "),
    },
    MenuItemDef {
        category_id: 2,
        category_title: "Quebras de Formato",
        code: "|16:9|",
        label: "Quebra Widescreen (16:9)",
        action: MenuAction::Insert(" |16:9| "),
    },
    MenuItemDef {
        category_id: 2,
        category_title: "Quebras de Formato",
        code: "|4:3|",
        label: "Quebra Formato Clássico (4:3)",
        action: MenuAction::Insert(" |4:3| "),
    },
    MenuItemDef {
        category_id: 2,
        category_title: "Quebras de Formato",
        code: "|9:16|",
        label: "Quebra Vertical (Stories / Reels)",
        action: MenuAction::Insert(" |9:16| "),
    },
    MenuItemDef {
        category_id: 2,
        category_title: "Quebras de Formato",
        code: "|21:9|",
        label: "Quebra Ultrawide (21:9)",
        action: MenuAction::Insert(" |21:9| "),
    },
    MenuItemDef {
        category_id: 2,
        category_title: "Quebras de Formato",
        code: "---",
        label: "Divisão de Estrofe (Novo Slide)",
        action: MenuAction::Insert("\n\n---\n\n"),
    },

    // 3. Tags na Seleção
    MenuItemDef {
        category_id: 3,
        category_title: "Tags de Seleção ([ ])",
        code: "[fonte: ...]",
        label: "Fonte na Seleção",
        action: MenuAction::ApplyFont("Montserrat"),
    },
    MenuItemDef {
        category_id: 3,
        category_title: "Tags de Seleção ([ ])",
        code: "[cor: ...]",
        label: "Cor na Seleção",
        action: MenuAction::ApplyColor("#FFD700"),
    },
    MenuItemDef {
        category_id: 3,
        category_title: "Tags de Seleção ([ ])",
        code: "[tamanho: ...]",
        label: "Tamanho na Seleção",
        action: MenuAction::ApplySize("46"),
    },
    MenuItemDef {
        category_id: 3,
        category_title: "Tags de Seleção ([ ])",
        code: "[sombra: ...]",
        label: "Sombra na Seleção",
        action: MenuAction::ApplyShadow("#00000082"),
    },

    // 4. Estrutura & Operador
    MenuItemDef {
        category_id: 4,
        category_title: "Estrutura & Operador",
        code: "// nota",
        label: "Comentário Oculto de Operador",
        action: MenuAction::Insert("\n// "),
    },
    MenuItemDef {
        category_id: 4,
        category_title: "Estrutura & Operador",
        code: "[Verso]",
        label: "Etiqueta de Verso",
        action: MenuAction::Insert("\n[Verso]\n"),
    },
    MenuItemDef {
        category_id: 4,
        category_title: "Estrutura & Operador",
        code: "[Refrão]",
        label: "Etiqueta de Refrão",
        action: MenuAction::Insert("\n[Refrão]\n"),
    },
    MenuItemDef {
        category_id: 4,
        category_title: "Estrutura & Operador",
        code: "[Ponte]",
        label: "Etiqueta de Ponte",
        action: MenuAction::Insert("\n[Ponte]\n"),
    },
    MenuItemDef {
        category_id: 4,
        category_title: "Estrutura & Operador",
        code: "[Pré-Refrão]",
        label: "Etiqueta de Pré-Refrão",
        action: MenuAction::Insert("\n[Pré-Refrão]\n"),
    },
    MenuItemDef {
        category_id: 4,
        category_title: "Estrutura & Operador",
        code: "[Final]",
        label: "Etiqueta de Final",
        action: MenuAction::Insert("\n[Final]\n"),
    },

    // 5. Marcações de Ênfase
    MenuItemDef {
        category_id: 5,
        category_title: "Marcações de Ênfase",
        code: "**texto**",
        label: "Negrito na Projeção",
        action: MenuAction::Wrap("**", "**"),
    },
    MenuItemDef {
        category_id: 5,
        category_title: "Marcações de Ênfase",
        code: "*texto*",
        label: "Itálico na Projeção",
        action: MenuAction::Wrap("*", "*"),
    },
    MenuItemDef {
        category_id: 5,
        category_title: "Marcações de Ênfase",
        code: "__texto__",
        label: "Sublinhado",
        action: MenuAction::Wrap("__", "__"),
    },
    MenuItemDef {
        category_id: 5,
        category_title: "Marcações de Ênfase",
        code: "~~texto~~",
        label: "Tachado",
        action: MenuAction::Wrap("~~", "~~"),
    },
    MenuItemDef {
        category_id: 5,
        category_title: "Marcações de Ênfase",
        code: "==texto==",
        label: "Destaque (Cor do Tema)",
        action: MenuAction::Wrap("==", "=="),
    },
];

/// Renders the componentized dropdown popup menu for conditional format breaks and options.
pub fn render_options_menu(
    resp: &egui::Response,
    state: &mut ControlPanelState,
    tokens: &AppTheme,
) -> bool {
    let mut changed = false;
    egui::Popup::menu(resp).show(|ui| {
        ui.set_width(420.0);
        ui.spacing_mut().item_spacing = Vec2::new(4.0, 4.0);

        let search_id = ui.make_persistent_id("editor_menu_search");
        let cat_id = ui.make_persistent_id("editor_menu_category");
        let mut query = ui.data_mut(|d| d.get_temp::<String>(search_id).unwrap_or_default());
        let mut active_cat = ui.data_mut(|d| d.get_temp::<usize>(cat_id).unwrap_or(0));

        // Campo de busca elegante no topo
        ui.horizontal(|ui| {
            ui.label(RichText::new(MAGNIFYING_GLASS).size(13.0).color(tokens.icon_muted));
            let edit_res = ui.add(
                egui::TextEdit::singleline(&mut query)
                    .hint_text("Buscar comando, quebra ou tag...")
                    .desired_width(ui.available_width() - 8.0)
                    .frame(egui::Frame::NONE)
                    .font(egui::FontId::proportional(12.0)),
            );
            if edit_res.changed() {
                ui.data_mut(|d| d.insert_temp(search_id, query.clone()));
            }
        });

        ui.add_space(2.0);
        ui.separator();

        // Categorias em pílulas compactas
        let categories = ["Todos", "Comandos //", "Quebras |", "Tags [ ]", "Estrutura", "Estilos"];
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            for (idx, name) in categories.iter().enumerate() {
                let is_sel = active_cat == idx;
                let bg = if is_sel { tokens.accent_soft } else { tokens.bg_pill };
                let stroke = if is_sel { tokens.accent } else { tokens.border_default.linear_multiply(0.7) };
                let text_color = if is_sel { tokens.accent } else { tokens.text_muted };
                let btn = egui::Button::new(RichText::new(*name).size(10.0).strong().color(text_color))
                    .fill(bg)
                    .stroke(Stroke::new(1.0, stroke))
                    .corner_radius(4.0);
                if ui.add(btn).clicked() {
                    active_cat = idx;
                    ui.data_mut(|d| d.insert_temp(cat_id, idx));
                }
            }
        });

        ui.add_space(3.0);

        // Área rolável compacta com altura máxima
        egui::ScrollArea::vertical()
            .id_salt("editor_menu_scroll")
            .max_height(340.0)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(0.0, 2.0);

                let q = query.trim().to_lowercase();
                let filtered: Vec<&MenuItemDef> = MENU_ITEMS
                    .iter()
                    .filter(|item| {
                        if active_cat != 0 && item.category_id != active_cat {
                            return false;
                        }
                        if q.is_empty() {
                            return true;
                        }
                        item.code.to_lowercase().contains(&q) || item.label.to_lowercase().contains(&q)
                    })
                    .collect();

                if filtered.is_empty() {
                    ui.add_space(16.0);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("Nenhuma opção encontrada")
                                .size(11.5)
                                .color(tokens.text_muted),
                        );
                    });
                    ui.add_space(16.0);
                } else {
                    let mut current_cat: Option<usize> = None;
                    for item in filtered {
                        if active_cat == 0 && q.is_empty() && current_cat != Some(item.category_id) {
                            current_cat = Some(item.category_id);
                            menu_section_title(ui, item.category_title, tokens);
                        }

                        if menu_item(ui, item.code, item.label, tokens).clicked() {
                            match item.action {
                                MenuAction::Insert(snippet) => {
                                    changed |= insert_text_at_selection(state, snippet);
                                }
                                MenuAction::ApplyFont(font) => {
                                    changed |= apply_font_selection(state, font);
                                }
                                MenuAction::ApplyColor(color) => {
                                    changed |= apply_color_selection(state, color);
                                }
                                MenuAction::ApplySize(size) => {
                                    changed |= apply_size_selection(state, size);
                                }
                                MenuAction::ApplyShadow(shadow) => {
                                    changed |= apply_shadow_selection(state, shadow);
                                }
                                MenuAction::Wrap(pre, suf) => {
                                    changed |= wrap_selection(state, pre, suf);
                                }
                            }
                            ui.data_mut(|d| d.insert_temp(search_id, String::new()));
                            ui.close();
                        }
                    }
                }
            });
    });
    changed
}

pub fn menu_section_title(ui: &mut egui::Ui, title: &str, tokens: &AppTheme) {
    ui.add_space(4.0);
    ui.label(
        RichText::new(title.to_uppercase())
            .size(9.5)
            .strong()
            .color(tokens.accent),
    );
    ui.add_space(1.0);
}

pub fn menu_item(ui: &mut egui::Ui, code: &str, label: &str, tokens: &AppTheme) -> egui::Response {
    let desired_size = Vec2::new(ui.available_width(), 30.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    let hover = response.hovered();

    if hover {
        ui.painter().rect_filled(rect, 6.0, tokens.bg_card_hover);
        ui.painter().rect_stroke(
            rect,
            6.0,
            Stroke::new(1.0, tokens.border_subtle),
            StrokeKind::Inside,
        );
    }

    // Coluna 1: Badge de Código fixa em 140px de largura
    let badge_w = 140.0;
    let badge_rect = egui::Rect::from_min_size(
        egui::pos2(rect.min.x + 6.0, rect.center().y - 10.0),
        Vec2::new(badge_w, 20.0),
    );

    ui.painter().rect(
        badge_rect,
        4.0,
        if hover { tokens.bg_pill_active } else { tokens.bg_pill },
        Stroke::new(
            1.0,
            if hover {
                tokens.accent.linear_multiply(0.5)
            } else {
                tokens.border_default.linear_multiply(0.7)
            },
        ),
        StrokeKind::Inside,
    );
    ui.painter().text(
        badge_rect.center(),
        egui::Align2::CENTER_CENTER,
        code,
        egui::FontId::monospace(10.0),
        tokens.accent,
    );

    // Coluna 2: Descrição com início garantido a 156px (sem nunca sobrepor)
    let text_x = rect.min.x + badge_w + 16.0;
    ui.painter().text(
        egui::pos2(text_x, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(11.5),
        if hover { tokens.text_primary } else { tokens.text_secondary },
    );

    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Renders a color menu item with a circular swatch dot and hex badge.
pub fn color_menu_item(
    ui: &mut egui::Ui,
    color: Color32,
    hex: &str,
    label: &str,
    tokens: &AppTheme,
) -> egui::Response {
    let desired_size = Vec2::new(ui.available_width(), 30.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    let hover = response.hovered();

    if hover {
        ui.painter().rect_filled(rect, 6.0, tokens.bg_card_hover);
        ui.painter().rect_stroke(
            rect,
            6.0,
            Stroke::new(1.0, tokens.border_subtle),
            StrokeKind::Inside,
        );
    }

    // Swatch circular colorido
    let dot_center = egui::pos2(rect.min.x + 16.0, rect.center().y);
    ui.painter().circle_filled(dot_center, 6.5, color);
    ui.painter().circle_stroke(
        dot_center,
        6.5,
        Stroke::new(1.0, tokens.border_default),
    );

    // Badge com código hex
    let badge_rect = egui::Rect::from_min_size(
        egui::pos2(rect.min.x + 30.0, rect.center().y - 10.0),
        Vec2::new(64.0, 20.0),
    );
    ui.painter().rect(
        badge_rect,
        4.0,
        if hover { tokens.bg_pill_active } else { tokens.bg_pill },
        Stroke::new(1.0, tokens.border_default.linear_multiply(0.7)),
        StrokeKind::Inside,
    );
    ui.painter().text(
        badge_rect.center(),
        egui::Align2::CENTER_CENTER,
        hex,
        egui::FontId::monospace(10.0),
        tokens.text_primary,
    );

    // Rótulo descritivo
    let text_x = badge_rect.max.x + 10.0;
    ui.painter().text(
        egui::pos2(text_x, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(11.5),
        if hover { tokens.text_primary } else { tokens.text_secondary },
    );

    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}
