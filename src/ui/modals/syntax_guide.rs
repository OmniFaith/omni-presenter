//! Syntax guide modal for OmniPresenter song editor formatting & directives.

use crate::core::state::ControlPanelState;
use crate::ui::modals::editor::selection::insert_text_at_selection;
use crate::ui::theme::AppTheme;
use eframe::egui::{self, RichText, Stroke, Vec2};

/// Renders the "Guia de Sintaxe & Opções" modal dialog if opened.
pub fn render_syntax_guide_modal(
    ctx: &egui::Context,
    state: &mut ControlPanelState,
    tokens: &AppTheme,
) {
    if !state.song_editor.show_syntax_guide {
        return;
    }

    let mut open = true;
    egui::Window::new("Guia de Sintaxe & Opções do Editor")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .default_width(480.0)
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);

            ui.label(
                RichText::new("Recursos e Marcações do OmniPresenter")
                    .size(13.0)
                    .strong()
                    .color(tokens.accent),
            );
            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(360.0)
                .show(ui, |ui| {
                    guide_section(ui, "Comandos de Estilo na Projeção", tokens);
                    guide_item(ui, "//fonte: Nome", "Comando de Fonte", "Define a tipografia da estrofe ou slide.", state, "\n//fonte: Montserrat\n", tokens);
                    guide_item(ui, "//cor: #HEX", "Comando de Cor", "Define a cor do texto da estrofe ou slide.", state, "\n//cor: #FFD700\n", tokens);
                    guide_item(ui, "//tamanho: N", "Comando de Tamanho", "Define o tamanho da fonte do slide.", state, "\n//tamanho: 46\n", tokens);
                    guide_item(ui, "//sombra: #HEX", "Comando de Sombra", "Define a sombra do texto no slide.", state, "\n//sombra: #00000082\n", tokens);
                    guide_item(ui, "//alinhar: centro", "Comando de Alinhamento", "Define alinhamento (centro, esquerda, direita, justificado).", state, "\n//alinhar: centro\n", tokens);
                    guide_item(ui, "//linha: 1.38", "Comando de Espaçamento", "Define espaçamento entre linhas do slide.", state, "\n//linha: 1.38\n", tokens);
                    guide_item(ui, "//letra: 1.8", "Espaçamento de Letras", "Define espaçamento entre caracteres.", state, "\n//letra: 1.8\n", tokens);
                    guide_item(ui, "[fonte: Nome]...[/fonte]", "Fonte da Seleção", "Altera a fonte apenas do trecho selecionado.", state, "[fonte: Montserrat]texto[/fonte]", tokens);
                    guide_item(ui, "[cor: #HEX]...[/cor]", "Cor da Seleção", "Altera a cor apenas do trecho selecionado.", state, "[cor: #FFD700]texto[/cor]", tokens);
                    guide_item(ui, "[tamanho: N]...[/tamanho]", "Tamanho da Seleção", "Altera tamanho apenas do trecho selecionado.", state, "[tamanho: 46]texto[/tamanho]", tokens);
                    guide_item(ui, "[sombra: #HEX]...[/sombra]", "Sombra da Seleção", "Altera sombra apenas do trecho selecionado.", state, "[sombra: #00000082]texto[/sombra]", tokens);

                    ui.add_space(4.0);
                    guide_section(ui, "Quebras de Linha & Divisão de Slide", tokens);
                    guide_item(ui, "|", "Quebra Suave Padrão", "Quebra a linha suavemente na projeção.", state, " | ", tokens);
                    guide_item(ui, "|16:9|, |4:3|, |9:16|, |21:9|", "Quebras Específicas por Formato", "Quebra apenas na proporção correspondente.", state, " |16:9| ", tokens);
                    guide_item(ui, "---", "Divisão Manual de Estrofe", "Cria um novo slide imediatamente.", state, "\n\n---\n\n", tokens);

                    ui.add_space(4.0);
                    guide_section(ui, "Comentários & Anotações de Operador", tokens);
                    guide_item(ui, "// texto", "Linha de Comentário", "Invisível na projeção. Para avisos da equipe.", state, "\n// Nota: ", tokens);

                    ui.add_space(4.0);
                    guide_section(ui, "Estrutura & Nomes de Estrofe", tokens);
                    guide_item(ui, "[Verso], [Refrão], [Ponte]", "Identificadores de Seção", "Cria etiquetas automáticas de estrofe.", state, "\n[Refrão]\n", tokens);

                    ui.add_space(4.0);
                    guide_section(ui, "Formatação & Destaque de Texto", tokens);
                    guide_item(ui, "**palavra**", "Negrito", "Aplica peso forte na palavra.", state, "**destaque**", tokens);
                    guide_item(ui, "*palavra*", "Itálico", "Aplica estilo itálico na palavra.", state, "*destaque*", tokens);
                    guide_item(ui, "__palavra__", "Sublinhado", "Aplica sublinhado no trecho.", state, "__destaque__", tokens);
                    guide_item(ui, "~~palavra~~", "Tachado", "Aplica tachado no trecho.", state, "~~destaque~~", tokens);
                });

            ui.separator();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add(
                        egui::Button::new(RichText::new("Fechar").size(11.5))
                            .fill(tokens.bg_pill)
                            .stroke(Stroke::new(1.0, tokens.border_default))
                            .corner_radius(6.0),
                    )
                    .clicked()
                {
                    state.song_editor.show_syntax_guide = false;
                }
            });
        });

    if !open {
        state.song_editor.show_syntax_guide = false;
    }
}

fn guide_section(ui: &mut egui::Ui, title: &str, tokens: &AppTheme) {
    ui.label(
        RichText::new(title)
            .size(11.5)
            .strong()
            .color(tokens.accent),
    );
}

fn guide_item(
    ui: &mut egui::Ui,
    syntax: &str,
    title: &str,
    description: &str,
    state: &mut ControlPanelState,
    insert_payload: &str,
    tokens: &AppTheme,
) {
    egui::Frame::new()
        .fill(tokens.bg_card)
        .stroke(Stroke::new(1.0, tokens.border_default))
        .corner_radius(6.0)
        .inner_margin(egui::Margin::same(8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(syntax)
                                .size(11.5)
                                .strong()
                                .color(tokens.text_primary),
                        );
                        ui.label(
                            RichText::new(format!("· {title}"))
                                .size(11.0)
                                .color(tokens.text_secondary),
                        );
                    });
                    ui.label(
                        RichText::new(description)
                            .size(10.0)
                            .color(tokens.text_secondary),
                    );
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("Inserir")
                                    .size(10.5)
                                    .color(tokens.accent),
                            )
                            .fill(tokens.bg_pill)
                            .stroke(Stroke::new(1.0, tokens.border_default))
                            .corner_radius(4.0),
                        )
                        .clicked()
                    {
                        insert_text_at_selection(state, insert_payload);
                    }
                });
            });
        });
}
