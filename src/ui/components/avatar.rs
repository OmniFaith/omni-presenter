use eframe::egui;
use std::hash::{Hash, Hasher};

pub fn paint(
    ui: &egui::Ui,
    rect: egui::Rect,
    photo: Option<&[u8]>,
    fallback: &str,
    tokens: &crate::ui::theme::AppTheme,
    active: bool,
) {
    if let Some(bytes) = photo {
        let mut hash = std::collections::hash_map::DefaultHasher::new();
        bytes.hash(&mut hash);
        let id = egui::Id::unique("profile_photo_texture").with(hash.finish());
        let cached = ui
            .ctx()
            .data_mut(|data| data.get_temp::<Option<egui::TextureHandle>>(id));
        let texture = cached.unwrap_or_else(|| {
            let texture = image::load_from_memory(bytes).ok().map(|image| {
                let image = image.thumbnail(192, 192).to_rgba8();
                ui.ctx().load_texture(
                    format!("avatar_{:x}", hash.finish()),
                    egui::ColorImage::from_rgba_unmultiplied(
                        [image.width() as usize, image.height() as usize],
                        image.as_raw(),
                    ),
                    egui::TextureOptions::LINEAR,
                )
            });
            ui.ctx()
                .data_mut(|data| data.insert_temp(id, texture.clone()));
            texture
        });
        if let Some(texture) = texture {
            egui::Image::from_texture((texture.id(), texture.size_vec2()))
                .corner_radius(egui::CornerRadius::same(
                    (rect.width() / 2.0).min(255.0) as u8
                ))
                .paint_at(ui, rect);
            return;
        }
    }
    ui.painter().circle_filled(
        rect.center(),
        rect.width() / 2.0,
        if active {
            tokens.accent_soft
        } else {
            tokens.bg_pill
        },
    );
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        fallback,
        egui::FontId::proportional((rect.width() * 0.48).clamp(12.0, 26.0)),
        tokens.accent,
    );
}
