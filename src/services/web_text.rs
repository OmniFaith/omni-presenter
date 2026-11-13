use super::Published;
use crate::core::models::PresentationTheme;

pub(super) fn layer(
    data: &mut Published,
    theme: &PresentationTheme,
    text: &str,
    font_scale: u32,
    aspect: f32,
) -> Option<String> {
    if text.trim().is_empty() {
        return None;
    }
    let mut theme = theme.clone();
    theme.font_size *= font_scale as f32 / 100.0;
    let (key, bytes) = crate::render::web_text::request(&theme, text, aspect);
    let bytes = bytes?;
    if data
        .design_assets
        .keys()
        .filter(|key| key.starts_with("native-text-"))
        .count()
        >= 32
    {
        data.design_assets
            .retain(|key, _| !key.starts_with("native-text-"));
    }
    data.design_assets
        .entry(key.clone())
        .or_insert_with(|| ((*bytes).clone(), "image/png"));
    Some(format!("/design-assets/{key}"))
}
