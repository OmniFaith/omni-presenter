use super::PresentationApp;
use crate::core::{
    design::{LayerContent, now_ms},
};

impl PresentationApp {
    pub(super) fn remote_player_control(
        &mut self,
        operation: &str,
        value: Option<f32>,
    ) -> Result<(), String> {
        match operation {
            "toggle" => self.control_state.toggle_player_playback(),
            "stop" => self.control_state.stop_player(),
            "volume" => self.control_state.audio_player.set_volume(
                value
                    .filter(|v| v.is_finite())
                    .ok_or("Volume inválido.")?
                    .clamp(0.0, 1.0),
            ),
            "seek" => {
                let value = value
                    .filter(|value| value.is_finite() && *value >= 0.0)
                    .ok_or("Posição inválida.")?;
                if self.control_state.active_player_media.is_none() {
                    return Err("O player está vazio.".into());
                }
                self.control_state.seek_player(value);
            }
            _ => return Err("Controle de mídia inválido.".into()),
        }
        Ok(())
    }

    pub(super) fn remote_design_control(&mut self, operation: &str) -> Result<(), String> {
        if ![
            "start",
            "pause",
            "reset",
            "question_next",
            "question_previous",
            "question_reveal",
        ]
        .contains(&operation)
        {
            return Err("Controle de design inválido.".into());
        }
        let id = self
            .slide_deck
            .live_overlay
            .as_ref()
            .or(self.slide_deck.live_design.as_ref())
            .or(self.slide_deck.preview_design.as_ref())
            .map(|scene| scene.id.clone())
            .ok_or("Não há design selecionado.")?;
        for scene in [
            &mut self.slide_deck.live_overlay,
            &mut self.slide_deck.live_design,
            &mut self.slide_deck.preview_design,
        ]
        .into_iter()
        .flatten()
        .filter(|scene| scene.id == id)
        {
            for layer in &mut scene.layers {
                match &mut layer.content {
                    LayerContent::Countdown(timer) => match operation {
                        "start" => timer.start(now_ms()),
                        "pause" => timer.pause(now_ms()),
                        "reset" => timer.reset(),
                        _ => {}
                    },
                    LayerContent::Questions {
                        items,
                        current,
                        revealed,
                    } => match operation {
                        "question_next" => {
                            *current = (*current + 1).min(items.len().saturating_sub(1));
                            *revealed = false;
                        }
                        "question_previous" => {
                            *current = current.saturating_sub(1);
                            *revealed = false;
                        }
                        "question_reveal" => *revealed = !*revealed,
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
        Ok(())
    }
}
