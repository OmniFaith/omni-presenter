use crate::{
    core::i18n::Language,
    services::search::{SearchResult, SearchSnapshot, SearchTarget},
    ui::AppTheme,
};
use eframe::egui::{self, RichText};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};

#[derive(Default)]
pub struct GlobalSearch {
    pub open: bool,
    query: String,
    results: Vec<SearchResult>,
    selected: usize,
    focus: bool,
    requested_at: Option<Instant>,
    revision: u64,
    receiver: Option<mpsc::Receiver<(u64, Vec<SearchResult>)>>,
    pending: bool,
    cancellation: Option<Arc<AtomicBool>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opening_search_only_draws_the_input_and_waits_for_typing() {
        let ctx = egui::Context::default();
        let mut search = GlobalSearch::default();
        search.toggle();
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1260.0, 760.0),
            )),
            ..Default::default()
        };
        for _ in 0..2 {
            let mut output = ctx.run_ui(input.clone(), |_| {
                search.show(&ctx, Language::English, &AppTheme::dark());
            });
            output.textures_delta.clear();
        }
        let mut output = ctx.run_ui(input.clone(), |_| {
            search.show(&ctx, Language::English, &AppTheme::dark());
        });
        output.textures_delta.clear();
        let texts: Vec<_> = output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::Shape::Text(text) if !text.galley.job.text.is_empty() => {
                    Some(text.galley.job.text.as_str())
                }
                _ => None,
            })
            .collect();
        assert_eq!(texts, vec!["Search"]);
        assert!(!search.needs_search());
        let mut typed = input;
        typed.events.push(egui::Event::Text("settings".into()));
        let mut output = ctx.run_ui(typed, |_| {
            search.show(&ctx, Language::English, &AppTheme::dark());
        });
        output.textures_delta.clear();
        assert_eq!(search.query, "settings");
        assert!(search.pending);
    }
}

impl GlobalSearch {
    pub fn toggle(&mut self) {
        if let Some(flag) = &self.cancellation {
            flag.store(true, Ordering::Relaxed);
        }
        self.open = !self.open;
        if self.open {
            self.query.clear();
            self.results.clear();
            self.selected = 0;
            self.pending = false;
            self.receiver = None;
            self.revision += 1;
            self.focus = true;
            self.requested_at = None;
        }
    }

    pub fn needs_search(&self) -> bool {
        self.open
            && !self.query.trim().is_empty()
            && self
                .requested_at
                .is_some_and(|time| time.elapsed() >= Duration::from_millis(60))
    }

    pub fn search(&mut self, snapshot: SearchSnapshot, ctx: &egui::Context) {
        let query = self.query.clone();
        let revision = self.revision;
        let (sender, receiver) = mpsc::channel();
        let ctx = ctx.clone();
        if let Some(flag) = &self.cancellation {
            flag.store(true, Ordering::Relaxed);
        }
        let cancelled = Arc::new(AtomicBool::new(false));
        self.cancellation = Some(cancelled.clone());
        std::thread::spawn(move || {
            let local = snapshot.local_results(&query);
            if !local.is_empty() && !cancelled.load(Ordering::Relaxed) {
                let _ = sender.send((revision, local));
                ctx.request_repaint();
            }
            let results = snapshot.search_with_cancellation(&query, &cancelled);
            if !cancelled.load(Ordering::Relaxed) {
                let _ = sender.send((revision, results));
                ctx.request_repaint();
            }
        });
        self.receiver = Some(receiver);
        self.pending = true;
        self.requested_at = None;
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        language: Language,
        theme: &AppTheme,
    ) -> Option<SearchTarget> {
        if !self.open {
            return None;
        }
        if let Some(receiver) = &self.receiver {
            while let Ok((revision, results)) = receiver.try_recv() {
                if revision == self.revision {
                    let selected = self
                        .results
                        .get(self.selected)
                        .map(|result| result.target.clone());
                    self.selected = selected
                        .and_then(|target| {
                            results.iter().position(|result| result.target == target)
                        })
                        .unwrap_or(0);
                    self.results = results;
                    self.pending = false;
                }
            }
        }
        let mut target = None;
        let close =
            ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        let down =
            ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown));
        let up =
            ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp));
        let enter =
            ctx.input_mut(|input| input.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
        if down {
            self.selected = (self.selected + 1).min(self.results.len().min(6).saturating_sub(1));
        }
        if up {
            self.selected = self.selected.saturating_sub(1);
        }
        if enter && !self.query.trim().is_empty() && !self.pending && self.requested_at.is_none() {
            target = self
                .results
                .get(self.selected)
                .map(|result| result.target.clone());
        }
        let id = egui::Id::unique("global_search");
        let screen = ctx.input(|input| input.viewport_rect());
        let modal = egui::Modal::new(id)
            .area(egui::Modal::default_area(id).anchor(
                egui::Align2::CENTER_TOP,
                egui::vec2(0.0, (screen.height() * 0.18).max(32.0)),
            ))
            .backdrop_color(egui::Color32::from_black_alpha(65))
            .frame(
                egui::Frame::new()
                    .fill(theme.bg_panel)
                    .stroke(egui::Stroke::new(1.0, theme.border_default))
                    .corner_radius(12.0)
                    .inner_margin(12),
            )
            .show(ctx, |ui| {
                ui.set_width(560.0_f32.min(screen.width() - 48.0));
                let response = ui.add_sized(
                    [ui.available_width(), 34.0],
                    egui::TextEdit::singleline(&mut self.query)
                        .font(egui::FontId::proportional(15.0))
                        .hint_text(language.text("common-search"))
                        .margin(egui::vec2(10.0, 8.0)),
                );
                if self.focus {
                    response.request_focus();
                    self.focus = false;
                }
                if response.changed() {
                    if let Some(flag) = &self.cancellation {
                        flag.store(true, Ordering::Relaxed);
                    }
                    self.revision += 1;
                    self.results.clear();
                    self.selected = 0;
                    self.pending = !self.query.trim().is_empty();
                    self.requested_at = if self.pending {
                        Some(Instant::now())
                    } else {
                        None
                    };
                    if self.pending {
                        ctx.request_repaint_after(Duration::from_millis(65));
                    }
                }
                if !self.query.trim().is_empty() {
                    ui.add_space(6.0);
                    if self.pending {
                        ui.horizontal(|ui| {
                            ui.spinner();
                        });
                    } else if self.results.is_empty() {
                        ui.label(
                            RichText::new(language.text("common-empty"))
                                .size(12.0)
                                .color(theme.text_muted),
                        );
                    }
                    for (index, result) in self.results.iter().take(6).enumerate() {
                        let (rect, response) = ui.allocate_exact_size(
                            egui::vec2(ui.available_width(), 42.0),
                            egui::Sense::click(),
                        );
                        let selected = index == self.selected;
                        if selected || response.hovered() {
                            ui.painter().rect_filled(
                                rect,
                                6.0,
                                if selected {
                                    theme.accent_soft
                                } else {
                                    theme.bg_card_hover
                                },
                            );
                        }
                        let category = ui.painter().layout_no_wrap(
                            language.text(result.category).into(),
                            egui::FontId::proportional(10.5),
                            theme.text_muted,
                        );
                        let title_rect = egui::Rect::from_min_max(
                            rect.min + egui::vec2(10.0, 0.0),
                            rect.max - egui::vec2(category.size().x + 24.0, 0.0),
                        );
                        let title = ui.painter().layout_no_wrap(
                            result.title.clone(),
                            egui::FontId::proportional(13.0),
                            theme.text_primary,
                        );
                        ui.painter().with_clip_rect(title_rect).galley(
                            egui::pos2(
                                title_rect.left(),
                                rect.center().y - title.size().y.min(18.0) / 2.0,
                            ),
                            title,
                            theme.text_primary,
                        );
                        ui.painter().galley(
                            egui::pos2(
                                rect.right() - category.size().x - 10.0,
                                rect.center().y - category.size().y / 2.0,
                            ),
                            category,
                            theme.text_muted,
                        );
                        if response
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            target = Some(result.target.clone());
                        }
                    }
                }
            });
        if close || modal.should_close() || target.is_some() {
            self.open = false;
            if let Some(flag) = &self.cancellation {
                flag.store(true, Ordering::Relaxed);
            }
        }
        target
    }
}
