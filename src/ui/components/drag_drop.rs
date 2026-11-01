use crate::{
    core::{
        design::DesignDocument,
        layout::ContainerKind,
        models::{QuickItem, QuickItemKind},
    },
    ui::AppTheme,
};
use eframe::egui::{self, RichText};

#[derive(Clone)]
pub enum DragSource {
    Panel(ContainerKind),
    Channels(Vec<String>),
    Layers { document: String, ids: Vec<String> },
    Playlist(Vec<String>),
    QuickBar(Vec<String>),
    Library,
    ChatMessage(u64),
}

#[derive(Clone)]
pub struct DragPayload {
    pub source: DragSource,
    pub title: String,
    pub subtitle: String,
    pub icon: &'static str,
    pub count: usize,
    pub items: Vec<QuickItem>,
    pub library_ids: Vec<i64>,
    pub design: Option<DesignDocument>,
}

impl DragPayload {
    pub fn for_design(scene: &DesignDocument) -> Self {
        let mut payload = Self::new(
            DragSource::Library,
            &scene.name,
            "Design",
            egui_phosphor::regular::STACK,
        );
        payload.design = Some(scene.clone());
        payload.items.push(QuickItem {
            id: scene.id.clone(),
            title: scene.name.clone(),
            subtitle: "Design".into(),
            kind: QuickItemKind::Design,
            content: scene.id.clone(),
        });
        payload
    }
    pub fn new(
        source: DragSource,
        title: impl Into<String>,
        subtitle: impl Into<String>,
        icon: &'static str,
    ) -> Self {
        Self {
            source,
            title: title.into(),
            subtitle: subtitle.into(),
            icon,
            count: 1,
            items: Vec::new(),
            library_ids: Vec::new(),
            design: None,
        }
    }
}

pub fn item_icon(kind: &QuickItemKind) -> &'static str {
    use egui_phosphor::regular::*;
    match kind {
        QuickItemKind::Song => MUSIC_NOTES,
        QuickItemKind::Media => VIDEO,
        QuickItemKind::Verse => BOOK_OPEN,
        QuickItemKind::Text => FILE_TEXT,
        QuickItemKind::Design => STACK,
    }
}

pub fn source(ui: &egui::Ui, response: &egui::Response, payload: impl FnOnce() -> DragPayload) {
    if response.drag_started() {
        egui::DragAndDrop::set_payload(ui.ctx(), payload());
    }
    if response.dragged() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
    }
}

pub fn reorder_target(
    ui: &egui::Ui,
    rect: egui::Rect,
    theme: &AppTheme,
    horizontal: bool,
    accepts: impl FnOnce(&DragSource) -> bool,
) -> Option<(std::sync::Arc<DragPayload>, bool)> {
    let payload = egui::DragAndDrop::payload::<DragPayload>(ui.ctx())?;
    if !accepts(&payload.source) {
        return None;
    }
    let pointer = ui.ctx().pointer_interact_pos()?;
    if !rect.intersect(ui.clip_rect()).contains(pointer) {
        return None;
    }
    let after = if horizontal {
        pointer.x > rect.center().x
    } else {
        pointer.y > rect.center().y
    };
    let (start, end) = if horizontal {
        let x = if after { rect.right() } else { rect.left() };
        (egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom()))
    } else {
        let y = if after { rect.bottom() } else { rect.top() };
        (
            egui::pos2(rect.left() + 3.0, y),
            egui::pos2(rect.right() - 3.0, y),
        )
    };
    ui.painter().line_segment(
        [start, end],
        egui::Stroke::new(6.0, theme.accent.linear_multiply(0.16)),
    );
    ui.painter()
        .line_segment([start, end], egui::Stroke::new(2.0, theme.accent));
    ui.painter().circle_filled(start, 3.0, theme.accent);
    if ui.input(|input| input.pointer.any_released()) {
        egui::DragAndDrop::clear_payload(ui.ctx());
        return Some((payload, after));
    }
    None
}

pub fn content_target(
    ui: &egui::Ui,
    rect: egui::Rect,
    theme: &AppTheme,
    label: &str,
) -> Option<std::sync::Arc<DragPayload>> {
    let payload = egui::DragAndDrop::payload::<DragPayload>(ui.ctx())?;
    if payload.items.is_empty() && !matches!(payload.source, DragSource::ChatMessage(_)) {
        return None;
    }
    if !ui
        .ctx()
        .pointer_interact_pos()
        .is_some_and(|point| rect.intersect(ui.clip_rect()).contains(point))
    {
        return None;
    }
    ui.painter()
        .rect_filled(rect, 7.0, theme.accent.linear_multiply(0.12));
    ui.painter().rect_stroke(
        rect,
        7.0,
        egui::Stroke::new(1.5, theme.accent),
        egui::StrokeKind::Inside,
    );
    let text = crate::ui::i18n::language(ui.ctx()).text(label).to_owned();
    let galley =
        ui.painter()
            .layout_no_wrap(text, egui::FontId::proportional(11.0), theme.text_on_accent);
    let badge = egui::Rect::from_center_size(rect.center(), galley.size() + egui::vec2(20.0, 10.0));
    ui.painter().rect_filled(badge, 6.0, theme.accent);
    ui.painter().galley(
        badge.min + egui::vec2(10.0, 5.0),
        galley,
        theme.text_on_accent,
    );
    if ui.input(|input| input.pointer.any_released()) {
        egui::DragAndDrop::clear_payload(ui.ctx());
        return Some(payload);
    }
    None
}

pub fn paint_preview(ctx: &egui::Context, theme: &AppTheme) {
    let Some(payload) = egui::DragAndDrop::payload::<DragPayload>(ctx) else {
        return;
    };
    let Some(pointer) = ctx.pointer_interact_pos() else {
        return;
    };
    if ctx.input(|input| input.pointer.any_released()) {
        return;
    }
    let screen = ctx.content_rect();
    let position = egui::pos2(
        (pointer.x + 18.0).min(screen.right() - 280.0),
        (pointer.y + 18.0).min(screen.bottom() - 116.0),
    );
    let language = crate::ui::i18n::language(ctx);
    egui::Area::new(egui::Id::unique("drag_preview_card"))
        .order(egui::Order::Tooltip)
        .fixed_pos(position)
        .interactable(false)
        .show(ctx, |ui| {
            egui::Frame::new()
                .fill(theme.bg_panel)
                .stroke(egui::Stroke::new(1.0, theme.accent.linear_multiply(0.65)))
                .corner_radius(12.0)
                .inner_margin(12)
                .shadow(egui::epaint::Shadow {
                    offset: [0, 8],
                    blur: 24,
                    spread: 0,
                    color: egui::Color32::from_black_alpha(90),
                })
                .show(ui, |ui| {
                    ui.set_width(242.0);
                    ui.horizontal(|ui| {
                        let (tile, _) =
                            ui.allocate_exact_size(egui::vec2(58.0, 48.0), egui::Sense::hover());
                        ui.painter().rect_filled(tile, 7.0, theme.accent_soft);
                        if let Some(design) = &payload.design {
                            crate::render::design::paint(
                                &ui.painter().with_clip_rect(tile),
                                tile.shrink(2.0),
                                design,
                            );
                        } else if matches!(payload.source, DragSource::Panel(_)) {
                            let inner = tile.shrink(7.0);
                            ui.painter().rect_stroke(
                                inner,
                                3.0,
                                egui::Stroke::new(1.0, theme.accent),
                                egui::StrokeKind::Inside,
                            );
                            for offset in [0.3, 0.55, 0.8] {
                                let y = inner.top() + inner.height() * offset;
                                ui.painter().line_segment(
                                    [
                                        egui::pos2(inner.left() + 5.0, y),
                                        egui::pos2(inner.right() - 5.0, y),
                                    ],
                                    egui::Stroke::new(2.0, theme.accent.linear_multiply(0.65)),
                                );
                            }
                        } else {
                            ui.painter().text(
                                tile.center(),
                                egui::Align2::CENTER_CENTER,
                                payload.icon,
                                egui::FontId::proportional(26.0),
                                theme.accent,
                            );
                        }
                        ui.vertical(|ui| {
                            ui.add(
                                egui::Label::new(
                                    RichText::new(&payload.title)
                                        .strong()
                                        .size(12.5)
                                        .color(theme.text_primary),
                                )
                                .truncate(),
                            );
                            ui.add(
                                egui::Label::new(
                                    RichText::new(&payload.subtitle)
                                        .size(10.5)
                                        .color(theme.text_secondary),
                                )
                                .truncate(),
                            );
                            if payload.count > 1 {
                                ui.label(
                                    RichText::new(language.format(
                                        "drag-count",
                                        &[("count", &payload.count.to_string())],
                                    ))
                                    .size(10.5)
                                    .color(theme.accent),
                                );
                            }
                        });
                    });
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(language.text(
                            if payload.items.is_empty()
                                && !matches!(payload.source, DragSource::ChatMessage(_))
                            {
                                "drag-move"
                            } else {
                                "drag-pin"
                            },
                        ))
                        .size(10.0)
                        .color(theme.text_muted),
                    );
                });
        });
}
