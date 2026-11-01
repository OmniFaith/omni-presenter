use super::*;
use crate::core::design::{DesignDocument, DesignLayer, LayerContent};
use crate::core::models::fonts::THEME_FONT;

#[test]
fn web_design_uses_each_scenes_theme_font_and_exact_layer_weights() {
    let mut base_theme = PresentationTheme::default();
    base_theme.font_family = "Serif".into();
    let mut overlay_theme = base_theme.clone();
    overlay_theme.id = "overlay-font".into();
    overlay_theme.font_family = "Monospace".into();
    let themes = vec![base_theme.clone(), overlay_theme.clone()];
    let mut layer = DesignLayer::new(LayerContent::Text("Fonte herdada".into()));
    layer.font_family = THEME_FONT.into();
    layer.set_font_weight(300);
    let mut heavy = layer.clone();
    heavy.set_font_weight(800);
    let scene = DesignDocument {
        theme_id: Some(base_theme.id.clone()),
        layers: vec![layer.clone(), heavy],
        ..Default::default()
    };
    let overlay = DesignDocument {
        theme_id: Some(overlay_theme.id.clone()),
        layers: vec![layer],
        overlay: true,
        ..Default::default()
    };
    let mut deck = SlideDeck::default();
    deck.live_design = Some(scene);
    deck.live_overlay = Some(overlay);
    let mut server = BrowserServer::preview_renderer();
    server.publish(&AppSettings::default(), &deck, &base_theme, true, &themes);
    let data = server.data.read().unwrap();
    let layers = data.design.as_ref().unwrap()["layers"].as_array().unwrap();
    assert_eq!(layers[0]["font_family"], "Serif");
    assert_eq!(layers[0]["font_weight"], 300);
    assert_eq!(layers[1]["font_weight"], 800);
    assert_ne!(layers[0]["fontUrl"], layers[1]["fontUrl"]);
    assert_eq!(layers[2]["font_family"], "Monospace");
    assert_eq!(
        deck.live_design.as_ref().unwrap().layers[0].font_family,
        THEME_FONT
    );
}
