//! Selection toolbar, componentized tools, and popups for the song lyric editor.

use super::actions::*;
use super::menu::render_options_menu;
use crate::core::state::ControlPanelState;
use crate::ui::theme::AppTheme;
use eframe::egui::{self, Color32, Stroke, StrokeKind, Vec2};
use egui_phosphor::regular::{
    BOOK_OPEN, CARET_DOWN, EYE, EYE_SLASH, LIST_BULLETS, LIST_NUMBERS,
    MAGIC_WAND, PLUS, STAR, TEXT_ALIGN_CENTER, TEXT_ALIGN_JUSTIFY,
    TEXT_ALIGN_LEFT, TEXT_ALIGN_RIGHT, TEXT_B, TEXT_INDENT, TEXT_ITALIC,
    TEXT_OUTDENT, TEXT_STRIKETHROUGH, TEXT_UNDERLINE,
};

// Re-export insert_text_at_selection for convenience and compatibility
pub use super::actions::insert_text_at_selection;

/// Renders a componentized 30x28 icon button matching the app's standard design.
pub fn component_tool_button(
    ui: &mut egui::Ui,
    id: egui::Id,
    label: &str,
    active: bool,
    tooltip: impl Into<String>,
    tokens: &AppTheme,
) -> egui::Response {
    let desired_size = Vec2::new(30.0, 28.0);
    let (rect, _) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
    let response = ui.interact(rect, id, egui::Sense::click());
    let hover = ui.ctx().animate_bool(id.with("hov"), response.hovered());
    let selected = ui.ctx().animate_bool(id.with("act"), active);

    let bg = tokens
        .bg_pill
        .lerp_to_gamma(tokens.bg_card_hover, hover)
        .lerp_to_gamma(tokens.accent_soft, selected);
    let stroke_color = tokens
        .border_default
        .lerp_to_gamma(tokens.border_subtle, hover)
        .lerp_to_gamma(tokens.accent, selected);

    ui.painter().rect(
        rect,
        6.0,
        bg,
        Stroke::new(1.0, stroke_color),
        StrokeKind::Inside,
    );

    let color = if active {
        tokens.accent
    } else {
        tokens.icon_default.lerp_to_gamma(tokens.text_primary, hover * 0.7)
    };

    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(14.0),
        color,
    );

    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(tooltip.into())
}

/// Renders a componentized pill button for dropdowns and combo-style controls.
pub fn component_pill_button(
    ui: &mut egui::Ui,
    id: egui::Id,
    icon_left: Option<&str>,
    text: &str,
    icon_right: Option<&str>,
    active: bool,
    tooltip: impl Into<String>,
    tokens: &AppTheme,
) -> egui::Response {
    let font_id = egui::FontId::proportional(12.0);
    let galley = ui.painter().layout_no_wrap(text.to_string(), font_id, tokens.text_primary);
    let mut desired_w = galley.size().x + 18.0;
    if icon_left.is_some() {
        desired_w += 14.0;
    }
    if icon_right.is_some() {
        desired_w += 14.0;
    }
    desired_w = desired_w.max(34.0);

    let (rect, _) = ui.allocate_exact_size(Vec2::new(desired_w, 28.0), egui::Sense::hover());
    let response = ui.interact(rect, id, egui::Sense::click());
    let hover = ui.ctx().animate_bool(id.with("hov"), response.hovered());
    let selected = ui.ctx().animate_bool(id.with("act"), active);

    let bg = tokens
        .bg_pill
        .lerp_to_gamma(tokens.bg_card_hover, hover)
        .lerp_to_gamma(tokens.accent_soft, selected);
    let stroke_color = tokens
        .border_default
        .lerp_to_gamma(tokens.border_subtle, hover)
        .lerp_to_gamma(tokens.accent, selected);

    ui.painter().rect(
        rect,
        6.0,
        bg,
        Stroke::new(1.0, stroke_color),
        StrokeKind::Inside,
    );

    let text_color = if active {
        tokens.accent
    } else {
        tokens.text_primary.lerp_to_gamma(tokens.accent, hover * 0.3)
    };

    let mut cur_x = rect.min.x + 8.0;
    if let Some(left) = icon_left {
        ui.painter().text(
            egui::pos2(cur_x, rect.center().y),
            egui::Align2::LEFT_CENTER,
            left,
            egui::FontId::proportional(13.0),
            if active { tokens.accent } else { tokens.icon_default },
        );
        cur_x += 16.0;
    }

    ui.painter().galley(
        egui::pos2(cur_x, rect.center().y - galley.size().y * 0.5),
        galley,
        text_color,
    );

    if let Some(right) = icon_right {
        ui.painter().text(
            egui::pos2(rect.max.x - 12.0, rect.center().y),
            egui::Align2::CENTER_CENTER,
            right,
            egui::FontId::proportional(11.0),
            if active { tokens.accent } else { tokens.icon_muted },
        );
    }

    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(tooltip.into())
}

/// Renders a componentized color swatch pill button.
pub fn component_color_pill(
    ui: &mut egui::Ui,
    id: egui::Id,
    color: Color32,
    hex: &str,
    tooltip: impl Into<String>,
    tokens: &AppTheme,
) -> egui::Response {
    let desired_w = 78.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(desired_w, 28.0), egui::Sense::hover());
    let response = ui.interact(rect, id, egui::Sense::click());
    let hover = ui.ctx().animate_bool(id.with("hov"), response.hovered());

    let bg = tokens
        .bg_pill
        .lerp_to_gamma(tokens.bg_card_hover, hover);
    let stroke_color = tokens
        .border_default
        .lerp_to_gamma(tokens.border_subtle, hover);

    ui.painter().rect(
        rect,
        6.0,
        bg,
        Stroke::new(1.0, stroke_color),
        StrokeKind::Inside,
    );

    let swatch_rect = egui::Rect::from_min_size(
        egui::pos2(rect.min.x + 6.0, rect.center().y - 7.0),
        Vec2::new(14.0, 14.0),
    );
    ui.painter().rect_filled(swatch_rect, 3.0, color);
    ui.painter().rect_stroke(
        swatch_rect,
        3.0,
        Stroke::new(1.0, tokens.border_default),
        StrokeKind::Inside,
    );

    ui.painter().text(
        egui::pos2(rect.min.x + 24.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        hex,
        egui::FontId::monospace(10.5),
        tokens.text_primary,
    );

    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(tooltip.into())
}

/// Renders a componentized shadow swatch pill button with checkerboard preview.
pub fn component_shadow_pill(
    ui: &mut egui::Ui,
    id: egui::Id,
    shadow_color: Color32,
    hex: &str,
    tooltip: impl Into<String>,
    tokens: &AppTheme,
) -> egui::Response {
    let desired_w = 98.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(desired_w, 28.0), egui::Sense::hover());
    let response = ui.interact(rect, id, egui::Sense::click());
    let hover = ui.ctx().animate_bool(id.with("hov"), response.hovered());

    let bg = tokens
        .bg_pill
        .lerp_to_gamma(tokens.bg_card_hover, hover);
    let stroke_color = tokens
        .border_default
        .lerp_to_gamma(tokens.border_subtle, hover);

    ui.painter().rect(
        rect,
        6.0,
        bg,
        Stroke::new(1.0, stroke_color),
        StrokeKind::Inside,
    );

    let swatch_rect = egui::Rect::from_min_size(
        egui::pos2(rect.min.x + 6.0, rect.center().y - 7.0),
        Vec2::new(14.0, 14.0),
    );
    let c1 = Color32::from_rgb(60, 60, 60);
    let c2 = Color32::from_rgb(120, 120, 120);
    ui.painter().rect_filled(egui::Rect::from_min_size(swatch_rect.min, Vec2::new(7.0, 7.0)), 0.0, c1);
    ui.painter().rect_filled(egui::Rect::from_min_size(egui::pos2(swatch_rect.min.x + 7.0, swatch_rect.min.y), Vec2::new(7.0, 7.0)), 0.0, c2);
    ui.painter().rect_filled(egui::Rect::from_min_size(egui::pos2(swatch_rect.min.x, swatch_rect.min.y + 7.0), Vec2::new(7.0, 7.0)), 0.0, c2);
    ui.painter().rect_filled(egui::Rect::from_min_size(egui::pos2(swatch_rect.min.x + 7.0, swatch_rect.min.y + 7.0), Vec2::new(7.0, 7.0)), 0.0, c1);
    ui.painter().rect_filled(swatch_rect, 3.0, shadow_color);
    ui.painter().rect_stroke(
        swatch_rect,
        3.0,
        Stroke::new(1.0, tokens.border_default),
        StrokeKind::Inside,
    );

    ui.painter().text(
        egui::pos2(rect.min.x + 24.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        hex,
        egui::FontId::monospace(10.0),
        tokens.text_secondary,
    );

    response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .on_hover_text(tooltip.into())
}

/// Renders a vertical separator between tool groups.
pub fn selection_separator(ui: &mut egui::Ui, tokens: &AppTheme) {
    ui.add_space(2.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(1.0, 20.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 0.5, tokens.border_default);
    ui.add_space(2.0);
}

/// Renders the complete, unified toolbar with full app componentization:
/// - Inserção & Marcadores (Visível Sempre): `[ + Inserir ▾ ]`, `[ | ]`, `[ // ]`, `[ 👁 ]`, `[ 📖 ]`
/// - Formatação Específica por Código: Fonte, Tamanho, Cor, Sombra, Caixa, Espaçamentos, Alinhamentos, Estilos Inline, Listas, Limpar.
/// - Opera SEMPRE via marcação/código: tags na seleção (`[tag: ...]`, `**negrito**`) ou comandos de slide (`//comando:`), nunca alterando o tema global da música.
pub fn render_editor_tools(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    tokens: &AppTheme,
) -> bool {
    let mut changed = false;
    let has_selection = state.song_editor.selection.0 != state.song_editor.selection.1;

    // ==========================================
    // GRUPO 1: INSERÇÃO & MARCADORES (SEMPRE VISÍVEIS)
    // ==========================================
    selection_separator(ui, tokens);

    // 1. Botão Principal [+ Inserir ▾]
    let insert_btn_id = ui.make_persistent_id("btn_insert_main_menu");
    let insert_resp = component_pill_button(
        ui,
        insert_btn_id,
        Some(PLUS),
        "Inserir",
        Some(CARET_DOWN),
        false,
        "Inserir Quebras Condicionais, Comandos de Slide, Tags e Estruturas",
        tokens,
    );
    changed |= render_options_menu(&insert_resp, state, tokens);

    // 2. Quebra Suave (|)
    let pipe_id = ui.make_persistent_id("btn_insert_soft_break");
    if component_tool_button(ui, pipe_id, "|", false, "Inserir Quebra Suave (|)", tokens).clicked() {
        changed |= insert_text_at_selection(state, " | ");
    }

    // 3. Comentário Oculto de Operador (//)
    let comment_id = ui.make_persistent_id("btn_insert_comment");
    if component_tool_button(
        ui,
        comment_id,
        "//",
        false,
        if has_selection {
            "Comentar Linhas Selecionadas (// nota)"
        } else {
            "Inserir Comentário Oculto (// nota de operador)"
        },
        tokens,
    )
    .clicked()
    {
        if has_selection {
            changed |= toggle_comment_selection(state);
        } else {
            changed |= insert_text_at_selection(state, "\n// ");
        }
    }

    // 4. Ocultar Marcadores na Prévia / Leitura Limpa (👁)
    let hide_markers = state.song_editor.hide_syntax_markers;
    let eye_icon = if hide_markers { EYE_SLASH } else { EYE };
    let eye_tip = if hide_markers {
        "Ocultar Marcadores: Ativo (//, |, [4:3] ocultados na prévia). Clique para exibir"
    } else {
        "Ocultar Marcadores: Desativado. Clique para ocultar marcações //, |, [4:3] na prévia"
    };
    let eye_id = ui.make_persistent_id("btn_toggle_hide_markers");
    if component_tool_button(ui, eye_id, eye_icon, hide_markers, eye_tip, tokens).clicked() {
        state.song_editor.hide_syntax_markers = !hide_markers;
        changed = true;
    }

    // 5. Guia de Sintaxe (📖)
    let guide_open = state.song_editor.show_syntax_guide;
    let guide_id = ui.make_persistent_id("btn_toggle_syntax_guide");
    if component_tool_button(
        ui,
        guide_id,
        BOOK_OPEN,
        guide_open,
        "Guia de Sintaxe & Marcações (Ver todos os comandos suportados)",
        tokens,
    )
    .clicked()
    {
        state.song_editor.show_syntax_guide = !guide_open;
    }

    // ==========================================
    // INDICADOR DE ESCOPO (SELEÇÃO vs SLIDE)
    // ==========================================
    selection_separator(ui, tokens);

    if has_selection {
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(76.0, 24.0), egui::Sense::hover());
        ui.painter().rect(
            rect,
            4.0,
            tokens.accent_soft,
            Stroke::new(1.0, tokens.accent),
            StrokeKind::Inside,
        );
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "✦ SELEÇÃO",
            egui::FontId::proportional(10.5),
            tokens.accent,
        );
        let _ = resp.on_hover_text("Formatação aplicada especificamente ao texto selecionado via tags [tag: ...] e estilos inline");
    } else {
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(54.0, 24.0), egui::Sense::hover());
        ui.painter().rect(
            rect,
            4.0,
            tokens.bg_pill,
            Stroke::new(1.0, tokens.border_default.linear_multiply(0.7)),
            StrokeKind::Inside,
        );
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "SLIDE",
            egui::FontId::proportional(10.0),
            tokens.text_muted,
        );
        let _ = resp.on_hover_text("Formatação aplicada à estrofe/slide atual via diretivas //comando:");
    }

    // ==========================================
    // GRUPO 2: TIPOGRAFIA & DIMENSÕES
    // ==========================================
    selection_separator(ui, tokens);

    // Fonte
    let font_btn_id = ui.make_persistent_id("btn_font_picker");
    let font_resp = component_pill_button(
        ui,
        font_btn_id,
        None,
        "Fonte",
        Some(CARET_DOWN),
        false,
        if has_selection {
            "Mudar Fonte da Seleção ([fonte: Nome]...[/fonte])"
        } else {
            "Definir Fonte do Slide (//fonte: Nome)"
        },
        tokens,
    );
    changed |= render_font_picker_menu(&font_resp, state, tokens, has_selection);

    // Tamanho
    let size_btn_id = ui.make_persistent_id("btn_size_picker");
    let size_resp = component_pill_button(
        ui,
        size_btn_id,
        None,
        "46",
        Some(CARET_DOWN),
        false,
        if has_selection {
            "Mudar Tamanho da Seleção ([tamanho: N]...[/tamanho])"
        } else {
            "Definir Tamanho do Slide (//tamanho: N)"
        },
        tokens,
    );
    changed |= render_size_picker_menu(&size_resp, state, tokens, has_selection);

    // Cor do Texto
    let color_btn_id = ui.make_persistent_id("btn_color_picker");
    let color_resp = component_color_pill(
        ui,
        color_btn_id,
        Color32::WHITE,
        "#FFFFFF",
        if has_selection {
            "Mudar Cor da Seleção ([cor: #HEX]...[/cor])"
        } else {
            "Definir Cor do Slide (//cor: #HEX)"
        },
        tokens,
    );
    changed |= render_color_picker_menu(&color_resp, state, tokens, has_selection);

    // Sombra do Texto
    let shadow_btn_id = ui.make_persistent_id("btn_shadow_picker");
    let shadow_resp = component_shadow_pill(
        ui,
        shadow_btn_id,
        Color32::from_black_alpha(130),
        "#00000082",
        if has_selection {
            "Mudar Sombra da Seleção ([sombra: #HEX]...[/sombra])"
        } else {
            "Definir Sombra do Slide (//sombra: #HEX)"
        },
        tokens,
    );
    changed |= render_shadow_picker_menu(&shadow_resp, state, tokens, has_selection);

    // Caixa de Texto (Aa)
    let case_btn_id = ui.make_persistent_id("btn_case_picker");
    let case_resp = component_pill_button(
        ui,
        case_btn_id,
        None,
        "Aa",
        Some(CARET_DOWN),
        false,
        "Alterar Caixa de Letras (MAIÚSCULAS / Primeira Maiúscula / minúsculas)",
        tokens,
    );
    changed |= render_case_picker_menu(&case_resp, state, tokens, has_selection);

    // ==========================================
    // GRUPO 3: ESPAÇAMENTOS (DIRETIVAS DE SLIDE)
    // ==========================================
    selection_separator(ui, tokens);

    let line_btn_id = ui.make_persistent_id("btn_line_spacing");
    let line_resp = component_pill_button(
        ui,
        line_btn_id,
        None,
        "↕ 1.38",
        Some(CARET_DOWN),
        false,
        "Espaçamento Entre Linhas do Slide (//linha: 1.38)",
        tokens,
    );
    changed |= render_line_spacing_menu(&line_resp, state, tokens);

    let letter_btn_id = ui.make_persistent_id("btn_letter_spacing");
    let letter_resp = component_pill_button(
        ui,
        letter_btn_id,
        None,
        "↔ 1.8",
        Some(CARET_DOWN),
        false,
        "Espaçamento Entre Letras do Slide (//letra: 1.8)",
        tokens,
    );
    changed |= render_letter_spacing_menu(&letter_resp, state, tokens);

    // ==========================================
    // GRUPO 4: ALINHAMENTO DO SLIDE
    // ==========================================
    selection_separator(ui, tokens);

    let al_left_id = ui.make_persistent_id("btn_align_left");
    if component_tool_button(ui, al_left_id, TEXT_ALIGN_LEFT, false, "Alinhar à Esquerda (//alinhar: esquerda)", tokens).clicked() {
        changed |= apply_or_update_slide_directive(state, "alinhar", "esquerda");
    }
    let al_center_id = ui.make_persistent_id("btn_align_center");
    if component_tool_button(ui, al_center_id, TEXT_ALIGN_CENTER, false, "Centralizar (//alinhar: centro)", tokens).clicked() {
        changed |= apply_or_update_slide_directive(state, "alinhar", "centro");
    }
    let al_right_id = ui.make_persistent_id("btn_align_right");
    if component_tool_button(ui, al_right_id, TEXT_ALIGN_RIGHT, false, "Alinhar à Direita (//alinhar: direita)", tokens).clicked() {
        changed |= apply_or_update_slide_directive(state, "alinhar", "direita");
    }
    let al_just_id = ui.make_persistent_id("btn_align_justify");
    if component_tool_button(ui, al_just_id, TEXT_ALIGN_JUSTIFY, false, "Justificar (//alinhar: justificado)", tokens).clicked() {
        changed |= apply_or_update_slide_directive(state, "alinhar", "justificado");
    }

    // ==========================================
    // GRUPO 5: ESTILOS INLINE (NEGRITO, ITÁLICO, ETC.)
    // ==========================================
    selection_separator(ui, tokens);

    let bold_id = ui.make_persistent_id("btn_style_bold");
    if component_tool_button(ui, bold_id, TEXT_B, false, "Negrito (**texto**)", tokens).clicked() {
        if has_selection {
            changed |= wrap_selection(state, "**", "**");
        } else {
            changed |= insert_text_at_selection(state, "****");
        }
    }

    let italic_id = ui.make_persistent_id("btn_style_italic");
    if component_tool_button(ui, italic_id, TEXT_ITALIC, false, "Itálico (*texto*)", tokens).clicked() {
        if has_selection {
            changed |= wrap_selection(state, "*", "*");
        } else {
            changed |= insert_text_at_selection(state, "**");
        }
    }

    let under_id = ui.make_persistent_id("btn_style_under");
    if component_tool_button(ui, under_id, TEXT_UNDERLINE, false, "Sublinhado (__texto__)", tokens).clicked() {
        if has_selection {
            changed |= wrap_selection(state, "__", "__");
        } else {
            changed |= insert_text_at_selection(state, "____");
        }
    }

    let strike_id = ui.make_persistent_id("btn_style_strike");
    if component_tool_button(ui, strike_id, TEXT_STRIKETHROUGH, false, "Tachado (~~texto~~)", tokens).clicked() {
        if has_selection {
            changed |= wrap_selection(state, "~~", "~~");
        } else {
            changed |= insert_text_at_selection(state, "~~~~");
        }
    }

    let star_id = ui.make_persistent_id("btn_style_star");
    if component_tool_button(ui, star_id, STAR, false, "Destaque na Cor do Tema (==texto==)", tokens).clicked() {
        if has_selection {
            changed |= wrap_selection(state, "==", "==");
        } else {
            changed |= insert_text_at_selection(state, "====");
        }
    }

    // ==========================================
    // GRUPO 6: LISTAS, RECUOS & LIMPAR
    // ==========================================
    selection_separator(ui, tokens);

    let bullet_id = ui.make_persistent_id("btn_list_bullets");
    if component_tool_button(ui, bullet_id, LIST_BULLETS, false, "Lista com Marcadores (• item)", tokens).clicked() {
        changed |= prefix_selected_lines(state, "• ");
    }

    let num_id = ui.make_persistent_id("btn_list_numbers");
    if component_tool_button(ui, num_id, LIST_NUMBERS, false, "Lista Numerada (1. item)", tokens).clicked() {
        changed |= number_selected_lines(state);
    }

    let outdent_id = ui.make_persistent_id("btn_indent_out");
    if component_tool_button(ui, outdent_id, TEXT_OUTDENT, false, "Diminuir Recuo", tokens).clicked() {
        changed |= indent_selected_lines(state, false);
    }

    let indent_id = ui.make_persistent_id("btn_indent_in");
    if component_tool_button(ui, indent_id, TEXT_INDENT, false, "Aumentar Recuo", tokens).clicked() {
        changed |= indent_selected_lines(state, true);
    }

    let wand_id = ui.make_persistent_id("btn_magic_wand");
    if component_tool_button(ui, wand_id, MAGIC_WAND, false, "Normalizar Espaços e Linhas Vazias", tokens).clicked() {
        changed |= normalize_whitespace(state);
    }

    let clear_id = ui.make_persistent_id("btn_clear_formatting");
    if component_tool_button(ui, clear_id, "Ø", false, "Limpar Formatações e Tags da Seleção", tokens).clicked() {
        changed |= strip_formatting_from_selection(state);
    }

    changed
}

fn render_font_picker_menu(
    resp: &egui::Response,
    state: &mut ControlPanelState,
    tokens: &AppTheme,
    has_selection: bool,
) -> bool {
    let mut changed = false;
    egui::Popup::menu(resp).show(|ui| {
        ui.set_min_width(220.0);
        let title = if has_selection {
            "Fonte da Seleção ([fonte: ...])"
        } else {
            "Fonte do Slide (//fonte: ...)"
        };
        super::menu::menu_section_title(ui, title, tokens);
        if super::menu::menu_item(ui, "Ø", "Padrão do Tema (Remover)", tokens).clicked() {
            if has_selection {
                changed |= apply_font_selection(state, "");
            } else {
                changed |= apply_or_update_slide_directive(state, "fonte", "Padrao");
            }
            ui.close();
        }
        ui.separator();
        for font in &[
            "Inter",
            "Montserrat",
            "Roboto",
            "Poppins",
            "Open Sans",
            "Playfair Display",
            "Cinzel",
            "Bebas Neue",
            "Lora",
            "Oswald",
        ] {
            let desc = if has_selection {
                format!("Aplicar [fonte: {font}]")
            } else {
                format!("Aplicar //fonte: {font}")
            };
            if super::menu::menu_item(ui, font, &desc, tokens).clicked() {
                if has_selection {
                    changed |= apply_font_selection(state, font);
                } else {
                    changed |= apply_or_update_slide_directive(state, "fonte", font);
                }
                ui.close();
            }
        }
    });
    changed
}

fn render_size_picker_menu(
    resp: &egui::Response,
    state: &mut ControlPanelState,
    tokens: &AppTheme,
    has_selection: bool,
) -> bool {
    let mut changed = false;
    egui::Popup::menu(resp).show(|ui| {
        ui.set_min_width(210.0);
        let title = if has_selection {
            "Tamanho da Seleção ([tamanho: N])"
        } else {
            "Tamanho do Slide (//tamanho: N)"
        };
        super::menu::menu_section_title(ui, title, tokens);
        if super::menu::menu_item(ui, "Ø", "Padrão do Tema (Remover)", tokens).clicked() {
            if has_selection {
                changed |= apply_size_selection(state, "");
            }
            ui.close();
        }
        ui.separator();
        for sz in &["28", "32", "38", "46", "52", "60", "72", "84", "96"] {
            let desc = if has_selection {
                format!("Aplicar [tamanho: {sz}]")
            } else {
                format!("Aplicar //tamanho: {sz}")
            };
            if super::menu::menu_item(ui, sz, &desc, tokens).clicked() {
                if has_selection {
                    changed |= apply_size_selection(state, sz);
                } else {
                    changed |= apply_or_update_slide_directive(state, "tamanho", sz);
                }
                ui.close();
            }
        }
    });
    changed
}

fn render_color_picker_menu(
    resp: &egui::Response,
    state: &mut ControlPanelState,
    tokens: &AppTheme,
    has_selection: bool,
) -> bool {
    let mut changed = false;
    egui::Popup::menu(resp).show(|ui| {
        ui.set_width(260.0);
        let title = if has_selection {
            "Cor da Seleção ([cor: #HEX])"
        } else {
            "Cor do Slide (//cor: #HEX)"
        };
        super::menu::menu_section_title(ui, title, tokens);
        if super::menu::menu_item(ui, "Ø", "Padrão do Tema (Remover)", tokens).clicked() {
            if has_selection {
                changed |= apply_color_selection(state, "");
            }
            ui.close();
        }
        ui.separator();
        let presets = [
            ("#FFFFFF", "Branco Puro"),
            ("#F59E0B", "Âmbar / Dourado"),
            ("#EF4444", "Vermelho Coral"),
            ("#06B6D4", "Ciano Claro"),
            ("#10B981", "Verde Esmeralda"),
            ("#8B5CF6", "Roxo Púrpura"),
            ("#EC4899", "Rosa Vibrante"),
            ("#F97316", "Laranja Solar"),
            ("#94A3B8", "Cinza Prata"),
        ];
        for (hex, name) in presets {
            let color = crate::core::markup::parse_hex_color(hex).unwrap_or(tokens.text_primary);
            if super::menu::color_menu_item(ui, color, hex, name, tokens).clicked() {
                if has_selection {
                    changed |= apply_color_selection(state, hex);
                } else {
                    changed |= apply_or_update_slide_directive(state, "cor", hex);
                }
                ui.close();
            }
        }
    });
    changed
}

fn render_shadow_picker_menu(
    resp: &egui::Response,
    state: &mut ControlPanelState,
    tokens: &AppTheme,
    has_selection: bool,
) -> bool {
    let mut changed = false;
    egui::Popup::menu(resp).show(|ui| {
        ui.set_width(260.0);
        let title = if has_selection {
            "Sombra da Seleção ([sombra: #HEX])"
        } else {
            "Sombra do Slide (//sombra: #HEX)"
        };
        super::menu::menu_section_title(ui, title, tokens);
        if super::menu::menu_item(ui, "Ø", "Sem Sombra (Remover)", tokens).clicked() {
            if has_selection {
                changed |= apply_shadow_selection(state, "");
            }
            ui.close();
        }
        ui.separator();
        let presets = [
            ("#00000082", "Sombra Padrão (51%)"),
            ("#000000CC", "Sombra Intensa (80%)"),
            ("#00000040", "Sombra Suave (25%)"),
            ("#0F172A", "Sombra Azul Noturno"),
            ("#F59E0B", "Realce Dourado"),
            ("#EF4444", "Realce Vermelho"),
        ];
        for (hex, name) in presets {
            let color = crate::core::markup::parse_hex_color(hex).unwrap_or(tokens.text_primary);
            if super::menu::color_menu_item(ui, color, hex, name, tokens).clicked() {
                if has_selection {
                    changed |= apply_shadow_selection(state, hex);
                } else {
                    changed |= apply_or_update_slide_directive(state, "sombra", hex);
                }
                ui.close();
            }
        }
    });
    changed
}

fn render_case_picker_menu(
    resp: &egui::Response,
    state: &mut ControlPanelState,
    tokens: &AppTheme,
    _has_selection: bool,
) -> bool {
    let mut changed = false;
    egui::Popup::menu(resp).show(|ui| {
        ui.set_min_width(210.0);
        super::menu::menu_section_title(ui, "Caixa de Letras", tokens);
        if super::menu::menu_item(ui, "AA", "Converter para MAIÚSCULAS", tokens).clicked() {
            changed |= transform_selection_case(state, CaseMode::Upper);
            ui.close();
        }
        if super::menu::menu_item(ui, "Aa", "Primeira Letra Maiúscula", tokens).clicked() {
            changed |= transform_selection_case(state, CaseMode::Title);
            ui.close();
        }
        if super::menu::menu_item(ui, "aa", "Converter para minúsculas", tokens).clicked() {
            changed |= transform_selection_case(state, CaseMode::Lower);
            ui.close();
        }
    });
    changed
}

fn render_line_spacing_menu(
    resp: &egui::Response,
    state: &mut ControlPanelState,
    tokens: &AppTheme,
) -> bool {
    let mut changed = false;
    egui::Popup::menu(resp).show(|ui| {
        ui.set_min_width(220.0);
        super::menu::menu_section_title(ui, "Espaçamento Entre Linhas (//linha:)", tokens);
        for val in &["1.0", "1.15", "1.25", "1.38", "1.50", "1.75", "2.0"] {
            if super::menu::menu_item(ui, val, &format!("Aplicar //linha: {val}"), tokens).clicked() {
                changed |= apply_or_update_slide_directive(state, "linha", val);
                ui.close();
            }
        }
    });
    changed
}

fn render_letter_spacing_menu(
    resp: &egui::Response,
    state: &mut ControlPanelState,
    tokens: &AppTheme,
) -> bool {
    let mut changed = false;
    egui::Popup::menu(resp).show(|ui| {
        ui.set_min_width(220.0);
        super::menu::menu_section_title(ui, "Espaçamento Entre Letras (//letra:)", tokens);
        for val in &["0.0", "0.6", "1.0", "1.8", "2.5", "4.0"] {
            if super::menu::menu_item(ui, val, &format!("Aplicar //letra: {val}"), tokens).clicked() {
                changed |= apply_or_update_slide_directive(state, "letra", val);
                ui.close();
            }
        }
    });
    changed
}
