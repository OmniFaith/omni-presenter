use super::player_widgets::{
    format_time, render_custom_scrubber, render_custom_volume_slider, render_hero_play_button,
    render_icon_circle_button,
};
use crate::core::i18n::Language;
use crate::core::layout::ContainerKind;
use crate::core::slides::SlideDeck;
use crate::core::state::{ActivePlayerMedia, ControlPanelState, PlayerMediaKind};
use crate::ui::components::render_header_bar;
use crate::ui::theme::AppTheme;
use eframe::egui::{self, Color32, RichText, Vec2};
use egui_phosphor::regular::{
    ARROW_SQUARE_OUT, ARROWS_IN, ARROWS_LEFT_RIGHT, ARROWS_OUT, MONITOR, MUSIC_NOTE, MUSIC_NOTES,
    REPEAT, SKIP_BACK, SKIP_FORWARD, SPEAKER_HIGH, SPEAKER_LOW, SPEAKER_SLASH, STOP, VIDEO, X,
};

pub fn render_player_column(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    deck: &mut SlideDeck,
    is_presentation_open: &mut bool,
) {
    let theme = state.theme.clone();
    let language = state.settings.language;
    render_header_bar(
        ui,
        ContainerKind::Player,
        language.text("control-player"),
        &theme,
    );
    ui.add_space(4.0);

    if let Some(media) = &mut state.active_player_media {
        if media.kind == PlayerMediaKind::Video && media.is_projecting {
            let is_still_projecting = *is_presentation_open
                && deck.live_media.as_ref().map(|m| &m.path) == Some(&media.path);
            if !is_still_projecting {
                media.is_projecting = false;
                state.audio_player.pause();
            }
        }
    }

    state.audio_player.update();
    state.refresh_media_duration();
    if let Some(media) = &mut state.active_player_media {
        if media.kind == PlayerMediaKind::Audio {
            media.is_playing = state.audio_player.is_playing();
        }
        if media.is_playing {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(33));
        }
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            if let Some(media) = state.active_player_media.clone() {
                render_active_player(ui, state, deck, is_presentation_open, &media, &theme);
            } else {
                render_empty_player(ui, &theme, language);
            }
        });
}

fn render_empty_player(ui: &mut egui::Ui, theme: &AppTheme, language: Language) {
    ui.add_space(14.0);
    ui.vertical_centered(|ui| {
        let (icon_rect, _) = ui.allocate_exact_size(Vec2::splat(32.0), egui::Sense::hover());
        let painter = ui.painter();
        painter.circle_filled(icon_rect.center(), 16.0, theme.bg_pill);
        painter.text(
            icon_rect.center(),
            egui::Align2::CENTER_CENTER,
            MUSIC_NOTES,
            egui::FontId::proportional(15.0),
            theme.icon_muted,
        );

        ui.add_space(6.0);
        ui.label(
            RichText::new(language.text("control-empty-player"))
                .size(11.5)
                .strong()
                .color(theme.text_secondary),
        );
        ui.add_space(2.0);
        ui.label(
            RichText::new(language.text("control-player-hint"))
                .size(10.0)
                .color(theme.text_muted),
        );
    });
    ui.add_space(12.0);
}

fn render_active_player(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    deck: &mut SlideDeck,
    is_presentation_open: &mut bool,
    media: &ActivePlayerMedia,
    theme: &AppTheme,
) {
    let language = state.settings.language;
    let is_video = media.kind == PlayerMediaKind::Video;

    let close_anim_id = egui::Id::NULL
        .with("player_auto_close_fade")
        .with(&media.path);
    let mut is_closing = ui
        .ctx()
        .data(|d| d.get_temp::<bool>(close_anim_id).unwrap_or(false));

    let is_live_video = media.is_projecting
        && *is_presentation_open
        && deck.live_media.as_ref().map(|m| &m.path) == Some(&media.path);

    if is_video && media.is_playing && !is_closing {
        let cur = media.video_position();
        let dur = media.duration_secs;
        let finished = (dur > 0.0
            && cur >= dur - 0.08
            && media.video_start_time.elapsed().as_secs_f32() > 0.3)
            || (is_live_video && crate::services::video::is_finished(ui.ctx(), &media.path));
        if finished {
            let has_next_slide = is_live_video
                && deck.live_slides().len() > 1
                && deck.live_index() + 1 < deck.live_slides().len();

            if has_next_slide {
                crate::services::video::stop(ui.ctx(), &media.path);
                deck.next_slide();
                crate::services::media::sync_live_media_playback(state, deck, *is_presentation_open);
                ui.ctx().request_repaint();
                return;
            }

            is_closing = true;
            ui.ctx().data_mut(|d| d.insert_temp(close_anim_id, true));
            if is_live_video {
                deck.live_media = None;
                *is_presentation_open = false;
            }
        }
    }

    let fade = ui
        .ctx()
        .animate_bool_with_time(close_anim_id.with("fade"), !is_closing, 0.4);
    if is_closing {
        ui.ctx().request_repaint();
        ui.set_opacity(fade);
        if fade <= 0.02 {
            stop_and_close_video(ui.ctx(), state, deck, is_presentation_open, media);
            return;
        }
    }

    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(8, 6))
        .show(ui, |ui| {
            // Posição e duração total
            let (cur_pos, total_dur) = match media.kind {
                PlayerMediaKind::Audio => {
                    let cur = state.audio_player.position_secs();
                    let dur = if media.duration_secs > 0.0 {
                        media.duration_secs
                    } else {
                        state
                            .audio_player
                            .current_track()
                            .map(|t| t.duration_secs)
                            .unwrap_or(1.0)
                    };
                    (cur, dur.max(1.0))
                }
                PlayerMediaKind::Video => {
                    let cur = media.video_position();
                    (cur, media.duration_secs.max(1.0))
                }
            };

            // 1. Vitrine Compacta (Arte + Título + Fechar integrados na mesma linha)
            if is_video {
                render_compact_video_showcase(ui, state, deck, is_presentation_open, media, theme);
            } else {
                render_compact_audio_showcase(ui, state, media, total_dur, theme);
            }

            ui.add_space(4.0);

            // 2. Linha do Tempo / Barra de Progresso com timestamps inline (00:04 [━━●━━] 20:16)
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format_time(cur_pos))
                        .size(9.5)
                        .strong()
                        .color(theme.text_secondary),
                );

                let scrubber_w = (ui.available_width() - 44.0).max(40.0);
                if let Some(seek_to) =
                    render_custom_scrubber(ui, cur_pos, total_dur, scrubber_w, theme, language)
                {
                    state.seek_player(seek_to);
                    if media.kind == PlayerMediaKind::Video {
                        crate::services::video::seek(ui.ctx(), &media.path, seek_to);
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let remaining = (total_dur - cur_pos).max(0.0);
                    ui.label(
                        RichText::new(format_time(total_dur))
                            .size(9.5)
                            .color(theme.text_muted),
                    )
                    .on_hover_text(
                        language.format("control-remaining", &[("time", &format_time(remaining))]),
                    );
                });
            });

            ui.add_space(4.0);

            // 3. Controles Principais + Volume (Layout Compacto e Adaptável)
            render_compact_controls_and_volume(
                ui,
                state,
                deck,
                is_presentation_open,
                media,
                cur_pos,
                total_dur,
                theme,
            );

            if is_video {
                ui.add_space(4.0);
                render_video_fit_segmented_bar(ui, state, media, theme);
            }
        });
}

fn render_compact_audio_showcase(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    media: &ActivePlayerMedia,
    total_dur: f32,
    theme: &AppTheme,
) {
    let language = state.settings.language;
    ui.horizontal(|ui| {
        let (cover_rect, _) = ui.allocate_exact_size(Vec2::splat(34.0), egui::Sense::hover());
        let painter = ui.painter();
        let center = cover_rect.center();
        painter.rect_filled(cover_rect, 6.0, Color32::from_rgb(24, 28, 38));
        painter.circle_filled(center, 14.5, Color32::from_rgb(14, 16, 22));
        painter.circle_stroke(
            center,
            12.0,
            egui::Stroke::new(0.5, Color32::from_rgb(38, 44, 58)),
        );
        painter.circle_stroke(
            center,
            9.5,
            egui::Stroke::new(0.5, Color32::from_rgb(30, 36, 48)),
        );
        painter.circle_filled(center, 5.5, theme.accent);
        painter.circle_filled(center, 1.8, Color32::from_rgb(240, 243, 248));

        ui.add_space(6.0);

        // Informações da faixa (Título no topo, tags embaixo)
        let close_btn_w = 22.0;
        let info_w = (ui.available_width() - close_btn_w).max(40.0);

        ui.allocate_ui_with_layout(
            Vec2::new(info_w, 34.0),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::Label::new(
                            RichText::new(&media.title)
                                .size(11.5)
                                .strong()
                                .color(theme.text_primary),
                        )
                        .truncate(),
                    )
                    .on_hover_text(language.format("control-file", &[("path", &media.path)]));

                    if media.is_playing {
                        ui.add_space(3.0);
                        // Barras de equalizador animadas dinamicamente
                        let (eq_rect, _) =
                            ui.allocate_exact_size(Vec2::new(12.0, 10.0), egui::Sense::hover());
                        let eq_painter = ui.painter();
                        let time = ui.input(|i| i.time) as f32;
                        let bar_w = 1.8;
                        let gap = 1.2;
                        let freqs = [8.5, 12.0, 7.0, 11.0];
                        let phases = [0.0, 1.3, 2.7, 4.1];

                        for i in 0..4 {
                            let h = 2.5 + 6.5 * ((time * freqs[i] + phases[i]).sin() * 0.5 + 0.5);
                            let bx = eq_rect.min.x + i as f32 * (bar_w + gap);
                            let by = eq_rect.max.y - h;
                            let b_rect = egui::Rect::from_min_max(
                                egui::pos2(bx, by),
                                egui::pos2(bx + bar_w, eq_rect.max.y),
                            );
                            eq_painter.rect_filled(b_rect, 0.8, theme.accent);
                        }
                    }
                });

                ui.add_space(2.0);

                let ext = std::path::Path::new(&media.path)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or(language.text("control-audio-label"))
                    .to_uppercase();

                ui.horizontal(|ui| {
                    let (b_rect, _) =
                        ui.allocate_exact_size(Vec2::new(42.0, 14.0), egui::Sense::hover());
                    ui.painter().rect_filled(b_rect, 3.0, theme.accent_soft);
                    ui.painter().text(
                        b_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        format!("{MUSIC_NOTE} {}", language.text("control-audio-label")),
                        egui::FontId::proportional(8.0),
                        theme.accent,
                    );

                    let (f_rect, _) =
                        ui.allocate_exact_size(Vec2::new(28.0, 14.0), egui::Sense::hover());
                    ui.painter().rect_filled(f_rect, 3.0, theme.bg_pill);
                    ui.painter().text(
                        f_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        &ext,
                        egui::FontId::proportional(8.0),
                        theme.text_secondary,
                    );

                    ui.label(
                        RichText::new(format!("• {}", format_time(total_dur)))
                            .size(9.5)
                            .color(theme.text_muted),
                    );
                });
            },
        );

        // Botão Fechar (X)
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let (close_rect, close_resp) =
                ui.allocate_exact_size(Vec2::splat(18.0), egui::Sense::click());
            if close_resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                ui.painter()
                    .circle_filled(close_rect.center(), 9.0, theme.bg_pill);
            }
            ui.painter().text(
                close_rect.center(),
                egui::Align2::CENTER_CENTER,
                X,
                egui::FontId::proportional(10.5),
                theme.text_muted,
            );
            if close_resp
                .on_hover_text(language.text("control-close-media"))
                .clicked()
            {
                state.close_player();
            }
        });
    });
}

fn stop_and_close_video(
    ctx: &egui::Context,
    state: &mut ControlPanelState,
    deck: &mut SlideDeck,
    is_presentation_open: &mut bool,
    media: &ActivePlayerMedia,
) {
    ctx.data_mut(|d| {
        d.remove::<bool>(
            egui::Id::NULL
                .with("player_auto_close_fade")
                .with(&media.path),
        )
    });
    crate::services::video::stop(ctx, &media.path);
    let is_live_video = media.is_projecting
        && *is_presentation_open
        && deck.live_media.as_ref().map(|m| &m.path) == Some(&media.path);
    if is_live_video {
        deck.live_media = None;
        deck.preview_media = None;
        if deck.live_title() == media.title {
            deck.clear_live();
        }
        state.apply_theme_to_live();
        *is_presentation_open = false;
    }
    state.close_player();
}

fn render_compact_video_showcase(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    deck: &mut SlideDeck,
    is_presentation_open: &mut bool,
    media: &ActivePlayerMedia,
    theme: &AppTheme,
) {
    let language = state.settings.language;
    let is_live = media.is_projecting
        && *is_presentation_open
        && deck.live_media.as_ref().map(|m| &m.path) == Some(&media.path);

    ui.horizontal(|ui| {
        let (b_rect, _) = ui.allocate_exact_size(Vec2::new(44.0, 15.0), egui::Sense::hover());
        ui.painter().rect_filled(b_rect, 3.0, theme.accent_soft);
        ui.painter().text(
            b_rect.center(),
            egui::Align2::CENTER_CENTER,
            format!("{VIDEO} {}", language.text("control-video-label")),
            egui::FontId::proportional(8.0),
            theme.accent,
        );

        let right_reserved =
            if media.path.starts_with("http://") || media.path.starts_with("https://") {
                72.0
            } else {
                50.0
            };
        let title_w = (ui.available_width() - right_reserved).max(30.0);
        ui.add_sized(
            [title_w, 18.0],
            egui::Label::new(
                RichText::new(&media.title)
                    .size(11.5)
                    .strong()
                    .color(theme.text_primary),
            )
            .truncate(),
        )
        .on_hover_text(language.format("control-file", &[("path", &media.path)]));

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Botão Fechar (X)
            let (close_rect, close_resp) =
                ui.allocate_exact_size(Vec2::splat(18.0), egui::Sense::click());
            if close_resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                ui.painter()
                    .circle_filled(close_rect.center(), 9.0, theme.bg_pill);
            }
            ui.painter().text(
                close_rect.center(),
                egui::Align2::CENTER_CENTER,
                X,
                egui::FontId::proportional(10.5),
                theme.text_muted,
            );
            if close_resp
                .on_hover_text(language.text("control-close-media"))
                .clicked()
            {
                stop_and_close_video(ui.ctx(), state, deck, is_presentation_open, media);
            }

            if media.path.starts_with("http://") || media.path.starts_with("https://") {
                let (web_rect, web_resp) =
                    ui.allocate_exact_size(Vec2::splat(18.0), egui::Sense::click());
                if web_resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    ui.painter()
                        .circle_filled(web_rect.center(), 9.0, theme.bg_pill);
                }
                ui.painter().text(
                    web_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    ARROW_SQUARE_OUT,
                    egui::FontId::proportional(10.5),
                    theme.text_muted,
                );
                if web_resp
                    .on_hover_text(language.text("library-web-video-open-browser"))
                    .clicked()
                {
                    ui.ctx().open_url(egui::OpenUrl::new_tab(&media.path));
                }
            }

            ui.add_space(4.0);

            // Botão de alternar projeção (Ao Vivo / Projetar)
            let (proj_bg, proj_fg, proj_hint) = if is_live {
                (
                    theme.accent,
                    theme.text_on_accent,
                    "Deixar de projetar vídeo",
                )
            } else {
                (
                    theme.bg_pill,
                    theme.text_secondary,
                    "Projetar vídeo no ao vivo",
                )
            };
            let (proj_rect, proj_resp) =
                ui.allocate_exact_size(Vec2::new(20.0, 18.0), egui::Sense::click());
            if proj_resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            ui.painter().rect_filled(proj_rect, 4.0, proj_bg);
            ui.painter().text(
                proj_rect.center(),
                egui::Align2::CENTER_CENTER,
                MONITOR,
                egui::FontId::proportional(11.0),
                proj_fg,
            );
            if proj_resp.on_hover_text(proj_hint).clicked() {
                if is_live {
                    deck.live_media = None;
                    deck.preview_media = None;
                    if deck.live_title() == media.title {
                        deck.clear_live();
                    }
                    state.apply_theme_to_live();
                    if let Some(m) = &mut state.active_player_media {
                        m.is_projecting = false;
                        m.is_playing = false;
                        m.video_paused_at = Some(m.video_position());
                    }
                    crate::services::video::stop(ui.ctx(), &media.path);
                    state.audio_player.pause();
                } else {
                    deck.load_custom_content(
                        media.title.clone(),
                        state.settings.language.text("workspace-media").to_string(),
                        String::new(),
                    );
                    deck.preview_media = Some(crate::core::slides::MediaOutput {
                        path: media.path.clone(),
                        kind: "video".into(),
                    });
                    deck.project_preview();
                    state.apply_theme_to_live();
                    *is_presentation_open = true;
                    state.show_presentation_dock = state.settings.operator().show_dock;
                    if let Some(m) = &mut state.active_player_media {
                        m.is_projecting = true;
                        m.is_playing = true;
                        let pos = m.video_position();
                        let _ = state.audio_player.play_video_stream(
                            m.title.clone(),
                            m.path.clone(),
                            pos,
                        );
                    }
                }
            }
        });
    });
}

fn render_video_fit_segmented_bar(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    media: &ActivePlayerMedia,
    theme: &AppTheme,
) {
    let avail_w = ui.available_width();
    let modes = [
        (
            "contain",
            ARROWS_IN,
            "Conter",
            "Conter: preserva proporção original sem cortes",
        ),
        (
            "cover",
            ARROWS_OUT,
            "Cobrir",
            "Cobrir: preenche a tela cortando sobras",
        ),
        (
            "stretch",
            ARROWS_LEFT_RIGHT,
            "Estender",
            "Estender: estica para preencher toda a tela",
        ),
    ];

    let height = 26.0;
    let (container_rect, _) =
        ui.allocate_exact_size(Vec2::new(avail_w, height), egui::Sense::hover());

    ui.painter().rect_filled(container_rect, 6.0, theme.bg_pill);
    ui.painter().rect_stroke(
        container_rect,
        6.0,
        egui::Stroke::new(1.0, theme.border_subtle),
        egui::StrokeKind::Inside,
    );

    let pad = 2.0;
    let seg_w = (container_rect.width() - pad * 2.0) / modes.len() as f32;
    let seg_h = container_rect.height() - pad * 2.0;

    let is_neutral_dark = theme.accent.r() == theme.accent.g()
        && theme.accent.g() == theme.accent.b()
        && theme.accent.r() < 120;
    let sel_text = if is_neutral_dark {
        theme.text_primary
    } else {
        theme.accent
    };
    let sel_border = if is_neutral_dark {
        theme.border_active
    } else {
        theme.accent
    };

    for (i, (fit_key, icon, label, tooltip)) in modes.iter().enumerate() {
        let is_selected = media.video_fit == *fit_key;
        let seg_rect = egui::Rect::from_min_size(
            egui::pos2(
                container_rect.left() + pad + i as f32 * seg_w,
                container_rect.top() + pad,
            ),
            Vec2::new(seg_w, seg_h),
        );
        let id = ui.auto_id_with(fit_key);
        let resp = ui.interact(seg_rect, id, egui::Sense::click());

        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        if is_selected {
            ui.painter().rect_filled(seg_rect, 5.0, theme.bg_card_hover);
            ui.painter().rect_stroke(
                seg_rect,
                5.0,
                egui::Stroke::new(1.0, sel_border),
                egui::StrokeKind::Inside,
            );
        } else if resp.hovered() {
            ui.painter()
                .rect_filled(seg_rect, 5.0, theme.bg_card_hover.linear_multiply(0.5));
        } else if i > 0 && media.video_fit != modes[i - 1].0 {
            let div_x = seg_rect.left();
            ui.painter().line_segment(
                [
                    egui::pos2(div_x, seg_rect.top() + 4.0),
                    egui::pos2(div_x, seg_rect.bottom() - 4.0),
                ],
                egui::Stroke::new(1.0, theme.border_subtle),
            );
        }

        let text_color = if is_selected {
            sel_text
        } else if resp.hovered() {
            theme.text_primary
        } else {
            theme.text_secondary
        };

        let font_id = if is_selected {
            egui::FontId::new(11.0, egui::FontFamily::Name("inter_bold".into()))
        } else {
            egui::FontId::proportional(11.0)
        };

        ui.painter().text(
            seg_rect.center(),
            egui::Align2::CENTER_CENTER,
            format!("{icon} {label}"),
            font_id,
            text_color,
        );

        if resp.on_hover_text(*tooltip).clicked() && !is_selected {
            if let Some(m) = &mut state.active_player_media {
                m.video_fit = (*fit_key).into();
            }
        }
    }
}

fn render_compact_controls_and_volume(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    deck: &mut SlideDeck,
    is_presentation_open: &mut bool,
    media: &ActivePlayerMedia,
    cur_pos: f32,
    total_dur: f32,
    theme: &AppTheme,
) {
    let avail_w = ui.available_width();
    let is_wide = avail_w >= 265.0;

    if is_wide {
        // Modo uma única linha: controles à esquerda e volume à direita
        ui.horizontal(|ui| {
            render_playback_buttons(
                ui,
                state,
                deck,
                is_presentation_open,
                media,
                cur_pos,
                total_dur,
                theme,
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                render_volume_controls(ui, state, 50.0, theme);
            });
        });
    } else {
        // Modo duas linhas compactas
        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                let controls_width = 148.0;
                let lead_space = ((ui.available_width() - controls_width) * 0.5).max(0.0);
                ui.add_space(lead_space);
                render_playback_buttons(
                    ui,
                    state,
                    deck,
                    is_presentation_open,
                    media,
                    cur_pos,
                    total_dur,
                    theme,
                );
            });
        });

        ui.add_space(2.0);

        ui.horizontal(|ui| {
            let slider_w = (ui.available_width() - 56.0).clamp(40.0, 90.0);
            render_volume_controls(ui, state, slider_w, theme);
        });
    }
}

fn render_playback_buttons(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    deck: &mut SlideDeck,
    is_presentation_open: &mut bool,
    media: &ActivePlayerMedia,
    cur_pos: f32,
    total_dur: f32,
    theme: &AppTheme,
) {
    let language = state.settings.language;
    // Repetir (Loop)
    let is_looping = state.audio_player.is_looping();
    if render_icon_circle_button(
        ui,
        REPEAT,
        24.0,
        11.0,
        is_looping,
        theme,
        if is_looping {
            language.text("control-repeat-on")
        } else {
            language.text("control-repeat-off")
        },
    )
    .clicked()
    {
        state.audio_player.toggle_loop();
    }

    ui.add_space(3.0);

    // Retroceder 10s
    if render_icon_circle_button(
        ui,
        SKIP_BACK,
        26.0,
        12.0,
        false,
        theme,
        language.text("control-back-10"),
    )
    .clicked()
    {
        let target = (cur_pos - 10.0).max(0.0);
        state.seek_player(target);
        if media.kind == PlayerMediaKind::Video {
            crate::services::video::seek(ui.ctx(), &media.path, target);
        }
    }

    ui.add_space(4.0);

    // Botão HERO de Play/Pause (30x30 destacado)
    if render_hero_play_button(ui, media.is_playing, theme, language).clicked() {
        state.toggle_player_playback();
        if media.kind == PlayerMediaKind::Video {
            let is_now_playing = state
                .active_player_media
                .as_ref()
                .map_or(false, |m| m.is_playing);
            if is_now_playing {
                crate::services::video::resume(ui.ctx(), &media.path);
            } else {
                crate::services::video::pause(ui.ctx(), &media.path);
            }
        }
    }

    ui.add_space(4.0);

    // Avançar 10s
    if render_icon_circle_button(
        ui,
        SKIP_FORWARD,
        26.0,
        12.0,
        false,
        theme,
        language.text("control-forward-10"),
    )
    .clicked()
    {
        let target = (cur_pos + 10.0).min(total_dur);
        state.seek_player(target);
        if media.kind == PlayerMediaKind::Video {
            crate::services::video::seek(ui.ctx(), &media.path, target);
        }
    }

    ui.add_space(3.0);

    // Parar (Stop)
    if render_icon_circle_button(
        ui,
        STOP,
        24.0,
        10.5,
        false,
        theme,
        language.text("control-stop-playback"),
    )
    .clicked()
    {
        stop_and_close_video(ui.ctx(), state, deck, is_presentation_open, media);
    }
}

fn render_volume_controls(
    ui: &mut egui::Ui,
    state: &mut ControlPanelState,
    slider_w: f32,
    theme: &AppTheme,
) {
    let language = state.settings.language;
    let vol = state.audio_player.volume();
    let vol_icon = if vol == 0.0 {
        SPEAKER_SLASH
    } else if vol < 0.4 {
        SPEAKER_LOW
    } else {
        SPEAKER_HIGH
    };

    let (spk_rect, spk_resp) = ui.allocate_exact_size(Vec2::splat(18.0), egui::Sense::click());
    if spk_resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let spk_color = if vol == 0.0 {
        Color32::from_rgb(220, 38, 38)
    } else if spk_resp.hovered() {
        theme.text_primary
    } else {
        theme.text_secondary
    };
    ui.painter().text(
        spk_rect.center(),
        egui::Align2::CENTER_CENTER,
        vol_icon,
        egui::FontId::proportional(12.5),
        spk_color,
    );
    if spk_resp
        .on_hover_text(if vol == 0.0 {
            language.text("control-unmute")
        } else {
            language.text("control-mute")
        })
        .clicked()
    {
        if vol > 0.0 {
            state.audio_player.set_volume(0.0);
        } else {
            state.audio_player.set_volume(1.0);
        }
    }

    ui.add_space(3.0);

    if let Some(new_vol) = render_custom_volume_slider(ui, vol, slider_w, theme, language) {
        state.audio_player.set_volume(new_vol);
    }

    ui.add_space(4.0);

    ui.label(
        RichText::new(format!("{:.0}%", vol * 100.0))
            .size(9.5)
            .color(theme.text_muted),
    );
}

#[cfg(test)]
#[path = "player_tests.rs"]
mod tests;
