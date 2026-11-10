use crate::core::models::PresentationTheme;
use eframe::egui::{self, Color32};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// ==========================================
// Cache Global de Texturas em Memória
// ==========================================
type TextureCache = Arc<Mutex<HashMap<String, Option<egui::TextureHandle>>>>;

fn get_texture_cache(ctx: &egui::Context) -> TextureCache {
    ctx.data_mut(|d| {
        d.get_temp_mut_or_insert_with(egui::Id::NULL.with("app_texture_cache_static"), || {
            Arc::new(Mutex::new(HashMap::new()))
        })
        .clone()
    })
}

/// Carrega ou obtém do cache a textura da imagem especificada (resolução original)
pub fn get_or_load_texture(ctx: &egui::Context, path: &str) -> Option<egui::TextureHandle> {
    if path.trim().is_empty() {
        return None;
    }

    let cache = get_texture_cache(ctx);
    if let Ok(map) = cache.lock()
        && let Some(cached_opt) = map.get(path) {
            return cached_opt.clone();
        }

    let loaded = match crate::services::images::request(ctx, path, 4096) {
        crate::services::images::ImageLoad::Ready(image) => {
            Some(ctx.load_texture(path, image.color, egui::TextureOptions::LINEAR))
        }
        crate::services::images::ImageLoad::Pending => return None,
        crate::services::images::ImageLoad::Failed(_) => None,
    };

    if let Ok(mut map) = cache.lock() {
        map.insert(path.to_string(), loaded.clone());
    }

    loaded
}

/// Carrega ou obtém do cache uma miniatura leve (320x180) para economizar até 99% de VRAM da GPU
pub fn get_or_load_thumbnail(ctx: &egui::Context, path: &str) -> Option<egui::TextureHandle> {
    if path.trim().is_empty() {
        return None;
    }

    let thumb_key = format!("{path}##thumb_320x180");
    let cache = get_texture_cache(ctx);
    if let Ok(map) = cache.lock()
        && let Some(cached_opt) = map.get(&thumb_key) {
            return cached_opt.clone();
        }

    let loaded = match crate::services::images::request(ctx, path, 320) {
        crate::services::images::ImageLoad::Ready(image) => {
            Some(ctx.load_texture(&thumb_key, image.color, egui::TextureOptions::LINEAR))
        }
        crate::services::images::ImageLoad::Pending => return None,
        crate::services::images::ImageLoad::Failed(_) => None,
    };

    if let Ok(mut map) = cache.lock() {
        map.insert(thumb_key, loaded.clone());
    }

    loaded
}

fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    Color32::from_rgb(
        (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t) as u8,
        (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t) as u8,
        (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t) as u8,
    )
}

/// Helper para pintar o plano de fundo do tema (Imagem, Gradiente, Cor Sólida, ou Animado)
pub fn paint_theme_bg(
    painter: &egui::Painter,
    rect: egui::Rect,
    theme: &PresentationTheme,
    corner_radius: f32,
) {
    match theme.bg_type.as_str() {
        "video" => crate::services::video::paint(painter, rect, theme, corner_radius),
        "shader" => {
            if !crate::services::shaders::paint(painter, rect, theme, corner_radius) {
                paint_theme_bg_base(painter, rect, theme, corner_radius);
            }
        }
        _ => paint_theme_bg_base(painter, rect, theme, corner_radius),
    }
    let brightness = theme.options.background_brightness;
    if brightness < 1.0 {
        painter.rect_filled(
            rect,
            corner_radius,
            Color32::from_black_alpha(((1.0 - brightness).clamp(0.0, 1.0) * 255.0) as u8),
        );
    } else if brightness > 1.0 {
        painter.rect_filled(
            rect,
            corner_radius,
            Color32::from_white_alpha(
                ((brightness - 1.0) * 0.35)
                    .clamp(0.0, 1.0)
                    .mul_add(255.0, 0.0) as u8,
            ),
        );
    }
    if theme.options.background_overlay > 0.0 {
        painter.rect_filled(
            rect,
            corner_radius,
            Color32::from_black_alpha(
                (theme.options.background_overlay.clamp(0.0, 1.0) * 255.0) as u8,
            ),
        );
    }
}

fn paint_theme_bg_base(
    painter: &egui::Painter,
    rect: egui::Rect,
    theme: &PresentationTheme,
    corner_radius: f32,
) {
    if theme.bg_top == Color32::TRANSPARENT && theme.bg_bottom == Color32::TRANSPARENT {
        return;
    }

    // 1. Se o tema for do tipo Imagem
    if theme.bg_type == "image" && !theme.bg_image_path.is_empty() {
        let tex_opt = if rect.width() <= 360.0 && rect.height() <= 240.0 {
            get_or_load_thumbnail(painter.ctx(), &theme.bg_image_path)
        } else {
            get_or_load_texture(painter.ctx(), &theme.bg_image_path)
        };

        if let Some(tex) = tex_opt {
            painter.rect_filled(rect, corner_radius, Color32::BLACK);
            let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
            paint_textured_rounded_rect(
                painter,
                rect,
                uv,
                tex.id(),
                corner_radius,
                Color32::WHITE,
            );
            return;
        }
        let size = if rect.width() <= 360.0 && rect.height() <= 240.0 { 320 } else { 4096 };
        if let crate::services::images::ImageLoad::Failed(error) = crate::services::images::request(painter.ctx(), &theme.bg_image_path, size) {
            painter.rect_filled(rect, corner_radius, theme.bg_bottom);
            painter.with_clip_rect(rect).text(rect.center(), egui::Align2::CENTER_CENTER,
                crate::ui::i18n::language(painter.ctx()).text("services-could-not-load-the-image"), egui::FontId::proportional(11.0), Color32::LIGHT_GRAY);
            let _ = error;
            return;
        }
    }

    // 2. Fundo dinâmico / animado ou gradiente / sólido
    let is_anim = theme.is_animated;
    let (c_top, c_bottom) = if is_anim {
        let time = painter.ctx().input(|i| i.time) as f32;
        painter.ctx().request_repaint();
        let phase = ((time * 0.7).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
        (
            lerp_color(theme.bg_top, theme.bg_bottom, phase),
            lerp_color(theme.bg_bottom, theme.bg_top, phase),
        )
    } else {
        (theme.bg_top, theme.bg_bottom)
    };

    // Pinta base
    painter.rect_filled(rect, corner_radius, c_bottom);

    let is_gradient_mode = theme.is_gradient || theme.bg_type == "gradient";
    if is_gradient_mode && c_top != c_bottom {
        paint_gradient_rect(
            painter,
            rect,
            c_top,
            c_bottom,
            theme.options.gradient_angle,
            corner_radius,
        );
    }
}

/// Pinta um gradiente suave de duas cores em um retângulo com suporte a rotação em qualquer ângulo (0° a 360°)
/// e cantos arredondados sem serrilhado.
pub fn paint_gradient_rect(
    painter: &egui::Painter,
    rect: egui::Rect,
    c_start: Color32,
    c_end: Color32,
    angle_deg: f32,
    corner_radius: f32,
) {
    if rect.width() <= 0.0 || rect.height() <= 0.0 {
        return;
    }
    let center = rect.center();
    let norm_angle = angle_deg.rem_euclid(360.0);
    let rad = norm_angle.to_radians();
    // 0° = topo para base (vetor aponta para baixo: sen(0)=0, cos(0)=1).
    let dir = egui::vec2(rad.sin(), rad.cos());
    let half_w = rect.width() * 0.5;
    let half_h = rect.height() * 0.5;
    let max_p = (half_w * rad.sin().abs() + half_h * rad.cos().abs()).max(0.001);

    let color_at = |pos: egui::Pos2| -> Color32 {
        let p = (pos - center).dot(dir);
        let t = ((p + max_p) / (2.0 * max_p)).clamp(0.0, 1.0);
        lerp_color(c_start, c_end, t)
    };

    let r = corner_radius.clamp(0.0, rect.width().min(rect.height()) * 0.5);

    if r <= 0.5 {
        // Retângulo com cantos vivos: malha em grade 12x12 garantindo gradiente liso em qualquer ângulo
        let steps_x = 12;
        let steps_y = 12;
        let mut mesh = egui::Mesh::default();
        for j in 0..=steps_y {
            let ty = j as f32 / steps_y as f32;
            let y = rect.min.y + ty * rect.height();
            for i in 0..=steps_x {
                let tx = i as f32 / steps_x as f32;
                let x = rect.min.x + tx * rect.width();
                let pos = egui::pos2(x, y);
                mesh.colored_vertex(pos, color_at(pos));
            }
        }
        for j in 0..steps_y {
            for i in 0..steps_x {
                let tl = (j * (steps_x + 1) + i) as u32;
                let tr = tl + 1;
                let bl = ((j + 1) * (steps_x + 1) + i) as u32;
                let br = bl + 1;
                mesh.add_triangle(tl, tr, br);
                mesh.add_triangle(tl, br, bl);
            }
        }
        painter.add(mesh);
    } else {
        // Retângulo com cantos arredondados: leque de vértices no perímetro com interpolação precisa
        let mut perimeter: Vec<egui::Pos2> = Vec::with_capacity(32);
        let arc_steps = 6;

        // Topo-Direito (-PI/2 até 0)
        let cx = rect.max.x - r;
        let cy = rect.min.y + r;
        for s in 0..=arc_steps {
            let phi = -std::f32::consts::FRAC_PI_2
                + (std::f32::consts::FRAC_PI_2 * (s as f32 / arc_steps as f32));
            perimeter.push(egui::pos2(cx + r * phi.cos(), cy + r * phi.sin()));
        }

        // Base-Direito (0 até PI/2)
        let cx = rect.max.x - r;
        let cy = rect.max.y - r;
        for s in 1..=arc_steps {
            let phi = std::f32::consts::FRAC_PI_2 * (s as f32 / arc_steps as f32);
            perimeter.push(egui::pos2(cx + r * phi.cos(), cy + r * phi.sin()));
        }

        // Base-Esquerdo (PI/2 até PI)
        let cx = rect.min.x + r;
        let cy = rect.max.y - r;
        for s in 1..=arc_steps {
            let phi = std::f32::consts::FRAC_PI_2
                + (std::f32::consts::FRAC_PI_2 * (s as f32 / arc_steps as f32));
            perimeter.push(egui::pos2(cx + r * phi.cos(), cy + r * phi.sin()));
        }

        // Topo-Esquerdo (PI até 3*PI/2)
        let cx = rect.min.x + r;
        let cy = rect.min.y + r;
        for s in 1..=arc_steps {
            let phi = std::f32::consts::PI
                + (std::f32::consts::FRAC_PI_2 * (s as f32 / arc_steps as f32));
            perimeter.push(egui::pos2(cx + r * phi.cos(), cy + r * phi.sin()));
        }

        let n = perimeter.len();
        let mut mesh = egui::Mesh::default();
        mesh.colored_vertex(center, color_at(center));

        for p in &perimeter {
            let mid_pos = center + (*p - center) * 0.5;
            mesh.colored_vertex(mid_pos, color_at(mid_pos));
        }

        for p in &perimeter {
            mesh.colored_vertex(*p, color_at(*p));
        }

        for i in 0..n {
            let next = (i + 1) % n;
            let m1 = (1 + i) as u32;
            let m2 = (1 + next) as u32;
            mesh.add_triangle(0, m1, m2);
        }

        for i in 0..n {
            let next = (i + 1) % n;
            let m1 = (1 + i) as u32;
            let m2 = (1 + next) as u32;
            let o1 = (1 + n + i) as u32;
            let o2 = (1 + n + next) as u32;
            mesh.add_triangle(m1, o1, o2);
            mesh.add_triangle(m1, o2, m2);
        }

        painter.add(mesh);
    }
}

/// Pinta uma textura com cantos arredondados sem serrilhado e mapeamento exato de UV
pub fn paint_textured_rounded_rect(
    painter: &egui::Painter,
    rect: egui::Rect,
    uv: egui::Rect,
    texture_id: egui::TextureId,
    corner_radius: f32,
    tint: Color32,
) {
    let r = corner_radius.clamp(0.0, rect.width().min(rect.height()) * 0.5);
    if r <= 0.5 {
        painter.image(texture_id, rect, uv, tint);
        return;
    }

    let mut perimeter: Vec<egui::Pos2> = Vec::with_capacity(32);
    let arc_steps = 6;

    // Topo-Direito (-PI/2 até 0)
    let cx = rect.max.x - r;
    let cy = rect.min.y + r;
    for s in 0..=arc_steps {
        let phi = -std::f32::consts::FRAC_PI_2
            + (std::f32::consts::FRAC_PI_2 * (s as f32 / arc_steps as f32));
        perimeter.push(egui::pos2(cx + r * phi.cos(), cy + r * phi.sin()));
    }

    // Base-Direito (0 até PI/2)
    let cx = rect.max.x - r;
    let cy = rect.max.y - r;
    for s in 1..=arc_steps {
        let phi = std::f32::consts::FRAC_PI_2 * (s as f32 / arc_steps as f32);
        perimeter.push(egui::pos2(cx + r * phi.cos(), cy + r * phi.sin()));
    }

    // Base-Esquerdo (PI/2 até PI)
    let cx = rect.min.x + r;
    let cy = rect.max.y - r;
    for s in 1..=arc_steps {
        let phi = std::f32::consts::FRAC_PI_2
            + (std::f32::consts::FRAC_PI_2 * (s as f32 / arc_steps as f32));
        perimeter.push(egui::pos2(cx + r * phi.cos(), cy + r * phi.sin()));
    }

    // Topo-Esquerdo (PI até 3*PI/2)
    let cx = rect.min.x + r;
    let cy = rect.min.y + r;
    for s in 1..=arc_steps {
        let phi = std::f32::consts::PI
            + (std::f32::consts::FRAC_PI_2 * (s as f32 / arc_steps as f32));
        perimeter.push(egui::pos2(cx + r * phi.cos(), cy + r * phi.sin()));
    }

    let to_uv = |p: egui::Pos2| -> egui::Pos2 {
        let tx = ((p.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
        let ty = ((p.y - rect.min.y) / rect.height()).clamp(0.0, 1.0);
        egui::pos2(
            uv.min.x + tx * uv.width(),
            uv.min.y + ty * uv.height(),
        )
    };

    let center = rect.center();
    let mut mesh = egui::Mesh::with_texture(texture_id);
    mesh.reserve_vertices(perimeter.len() + 1);
    mesh.reserve_triangles(perimeter.len());

    mesh.vertices.push(egui::epaint::Vertex {
        pos: center,
        uv: to_uv(center),
        color: tint,
    });

    for &p in &perimeter {
        mesh.vertices.push(egui::epaint::Vertex {
            pos: p,
            uv: to_uv(p),
            color: tint,
        });
    }

    let n = perimeter.len() as u32;
    for i in 0..n {
        let next = (i + 1) % n;
        mesh.add_triangle(0, 1 + i, 1 + next);
    }

    painter.add(mesh);
}

/// Pinta máscara nos 4 cantos para impedir que qualquer conteúdo (vídeo, shader, imagem, design)
/// vaze para fora dos cantos arredondados de um card de exibição
pub fn paint_corner_mask(
    painter: &egui::Painter,
    rect: egui::Rect,
    corner_radius: f32,
    bg_color: Color32,
) {
    let r = corner_radius.clamp(0.0, rect.width().min(rect.height()) * 0.5);
    if r <= 0.5 || bg_color == Color32::TRANSPARENT {
        return;
    }

    let arc_steps = 6;
    let mut mesh = egui::Mesh::default();

    let mut add_corner = |corner_p: egui::Pos2, arc_c: egui::Pos2, start_angle: f32| {
        let base_idx = mesh.vertices.len() as u32;
        mesh.colored_vertex(corner_p, bg_color);
        for s in 0..=arc_steps {
            let phi = start_angle + (std::f32::consts::FRAC_PI_2 * (s as f32 / arc_steps as f32));
            let pos = egui::pos2(arc_c.x + r * phi.cos(), arc_c.y + r * phi.sin());
            mesh.colored_vertex(pos, bg_color);
        }
        for s in 0..arc_steps {
            mesh.add_triangle(base_idx, base_idx + 1 + s as u32, base_idx + 2 + s as u32);
        }
    };

    // 1. Topo-Esquerdo
    add_corner(
        rect.left_top(),
        egui::pos2(rect.min.x + r, rect.min.y + r),
        std::f32::consts::PI,
    );

    // 2. Topo-Direito
    add_corner(
        rect.right_top(),
        egui::pos2(rect.max.x - r, rect.min.y + r),
        -std::f32::consts::FRAC_PI_2,
    );

    // 3. Base-Direito
    add_corner(
        rect.right_bottom(),
        egui::pos2(rect.max.x - r, rect.max.y - r),
        0.0,
    );

    // 4. Base-Esquerdo
    add_corner(
        rect.left_bottom(),
        egui::pos2(rect.min.x + r, rect.max.y - r),
        std::f32::consts::FRAC_PI_2,
    );

    painter.add(mesh);
}

/// Renderiza as letras do slide formatadas com bounding box, alinhamentos e tipografia do tema
/// com cálculo de auto-fit proporcional garantindo que a prévia seja uma réplica idêntica do telão.
pub fn render_theme_lyrics(
    painter: &egui::Painter,
    slide_rect: egui::Rect,
    theme: &PresentationTheme,
    raw_text: &str,
) {
    super::text::render(painter, slide_rect, theme, raw_text);
}

/// Renderiza as letras com desfoque óptico gaussiano durante transições
pub fn render_theme_lyrics_blurred(
    painter: &egui::Painter,
    slide_rect: egui::Rect,
    theme: &PresentationTheme,
    raw_text: &str,
    blur_radius: f32,
) {
    super::text::render_blurred(painter, slide_rect, theme, raw_text, blur_radius);
}

/// Versão de [`render_theme_lyrics`] que permite sobrepor a cor do texto (ex: para cartões sem background na UI).
pub fn render_theme_lyrics_with_color(
    painter: &egui::Painter,
    slide_rect: egui::Rect,
    theme: &PresentationTheme,
    raw_text: &str,
    override_color: Option<egui::Color32>,
) {
    super::text::render_with_override_color(painter, slide_rect, theme, raw_text, override_color);
}

/// Pré-aquece todos os glifos das estrofes da música no atlas do egui,
/// eliminando completamente qualquer travamento ou atraso na troca de estrofes durante o ao vivo.
pub fn prewarm_slides_in_egui(
    ctx: &egui::Context,
    theme: &PresentationTheme,
    slides: &[String],
    _target_font_size: f32,
) {
    if slides.is_empty() {
        return;
    }
    let font_family = crate::core::models::ensure_font_loaded_with_style(
        ctx,
        &theme.font_family,
        theme.font_weight,
        theme.font_italic,
    );
    let ref_font_id = egui::FontId::new(24.0, font_family.clone());
    let pres_font_id = egui::FontId::new(96.0, font_family);

    ctx.fonts_mut(|f| {
        for slide in slides {
            for line in slide.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    let _ =
                        f.layout_no_wrap(trimmed.to_string(), ref_font_id.clone(), Color32::WHITE);
                    let _ =
                        f.layout_no_wrap(trimmed.to_string(), pres_font_id.clone(), Color32::WHITE);
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_loading_performance() {
        let ctx = egui::Context::default();
        let path = std::env::temp_dir().join(format!("{}.png", crate::core::preferences::new_id("texture-test")));
        image::RgbaImage::new(2, 2).save(&path).unwrap();
        let path_str = path.to_str().unwrap();
        for _frame in 0..5 {
            let mut output = ctx.run_ui(egui::RawInput::default(), |_ui| {
                let _tex = get_or_load_texture(&ctx, path_str);
            });
            output.textures_delta.clear();
        }
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_prewarm_slides_performance() {
        let ctx = egui::Context::default();
        let mut out = ctx.run_ui(egui::RawInput::default(), |_| {});
        out.textures_delta.clear();
        let theme = PresentationTheme::default();
        let slides = vec![
            "Estrofe 1\nLinha A\nLinha B".to_string(),
            "Estrofe 2\nMais rejeitado dentre os homens".to_string(),
            "Estrofe 3\nO Seu castigo nos traz paz".to_string(),
        ];
        prewarm_slides_in_egui(&ctx, &theme, &slides, 44.0);
    }

    #[test]
    fn test_paint_gradient_rect_all_angles() {
        let ctx = egui::Context::default();
        let angles = [
            0.0, 30.0, 45.0, 90.0, 135.0, 180.0, 270.0, 360.0, -45.0, 720.0,
        ];
        let radii = [0.0, 4.0, 10.0, 50.0];
        let rect = egui::Rect::from_min_size(egui::pos2(10.0, 10.0), egui::vec2(200.0, 100.0));

        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
            for &angle in &angles {
                for &radius in &radii {
                    paint_gradient_rect(
                        ui.painter(),
                        rect,
                        Color32::from_rgb(59, 130, 246),
                        Color32::from_rgb(139, 92, 246),
                        angle,
                        radius,
                    );
                }
            }
        });
        out.textures_delta.clear();
        assert!(
            !out.shapes.is_empty(),
            "Meshes should have been painted for gradients"
        );
    }

    #[test]
    fn test_paint_textured_rounded_rect_and_corner_mask() {
        let ctx = egui::Context::default();
        let rect = egui::Rect::from_min_size(egui::pos2(10.0, 10.0), egui::vec2(200.0, 100.0));
        let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        let tex = ctx.load_texture(
            "test_tex",
            egui::ColorImage::example(),
            egui::TextureOptions::LINEAR,
        );

        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
            // Sharp rect
            paint_textured_rounded_rect(ui.painter(), rect, uv, tex.id(), 0.0, Color32::WHITE);
            // Rounded rect
            paint_textured_rounded_rect(ui.painter(), rect, uv, tex.id(), 10.0, Color32::WHITE);
            // Corner mask
            paint_corner_mask(ui.painter(), rect, 10.0, Color32::from_rgb(20, 20, 20));
        });
        out.textures_delta.clear();
        assert!(out.shapes.len() >= 3);
    }
}
