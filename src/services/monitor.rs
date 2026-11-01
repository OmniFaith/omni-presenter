#[derive(Clone, Debug, PartialEq)]
pub struct MonitorInfo {
    pub winit_index: usize,
    pub name: String,
    pub display_name: String,
    pub resolution: (u32, u32),
    pub position: (i32, i32),
    pub scale_factor: f64,
}

impl MonitorInfo {
    /// ViewportBuilder is in egui points; winit applies native DPI AND egui
    /// zoom again. Compensate both so the final physical rectangle is exact.
    pub fn viewport_rect(&self, zoom: f32) -> eframe::egui::Rect {
        let scale = (self.scale_factor as f32 * zoom).max(0.1);
        eframe::egui::Rect::from_min_size(
            eframe::egui::pos2(
                self.position.0 as f32 / scale,
                self.position.1 as f32 / scale,
            ),
            eframe::egui::vec2(
                self.resolution.0 as f32 / scale,
                self.resolution.1 as f32 / scale,
            ),
        )
    }
}

pub fn detect_primary_monitor_from_window(window: &winit::window::Window) -> Option<String> {
    if let Some(primary) = window.primary_monitor()
        && let Some(name) = primary.name() {
            return Some(name);
        }
    detect_primary_monitor_name()
}

pub fn detect_primary_monitor_name() -> Option<String> {
    let output = std::process::Command::new("xrandr")
        .arg("--listmonitors")
        .output()
        .ok()?;

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        if line.contains('*')
            && let Some(name) = line.split_whitespace().last() {
                return Some(name.to_owned());
            }
    }
    None
}

pub fn get_unique_monitors(window: &winit::window::Window) -> Vec<MonitorInfo> {
    let available: Vec<_> = window.available_monitors().collect();
    let mut unique_monitors = Vec::new();

    for (index, monitor) in available.iter().enumerate() {
        let name = monitor.name().unwrap_or_else(|| format!("Monitor {index}"));
        if !unique_monitors.iter().any(|m: &MonitorInfo| m.name == name) {
            let size = monitor.size();
            let pos = monitor.position();
            let scale = monitor.scale_factor();
            unique_monitors.push(MonitorInfo {
                winit_index: index,
                display_name: display_name(&name, index),
                name,
                resolution: (size.width, size.height),
                position: (pos.x, pos.y),
                scale_factor: scale,
            });
        }
    }

    // Keep the card numbering tied to the desktop arrangement, not the order
    // of the native backend's monitor map (which may change between queries).
    unique_monitors.sort_by(|a, b| (a.position, &a.name).cmp(&(b.position, &b.name)));
    unique_monitors
}

fn display_name(_connector: &str, index: usize) -> String {
    #[cfg(target_os = "linux")]
    if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            if !entry
                .file_name()
                .to_string_lossy()
                .ends_with(&format!("-{_connector}"))
            {
                continue;
            }
            if let Ok(edid) = std::fs::read(entry.path().join("edid")) {
                if edid.len() < 128 {
                    continue;
                }
                for descriptor in edid[54..126].as_chunks::<18>().0 {
                    if descriptor[..5] == [0, 0, 0, 0xfc, 0] {
                        let model = String::from_utf8_lossy(&descriptor[5..18])
                            .trim()
                            .to_owned();
                        if !model.is_empty() {
                            let id = u16::from_be_bytes([edid[8], edid[9]]);
                            let vendor: String = [10, 5, 0]
                                .map(|shift| (((id >> shift) & 31) as u8 + b'@') as char)
                                .into_iter()
                                .collect();
                            let vendor = match vendor.as_str() {
                                "SAM" => "Samsung",
                                "DEL" => "Dell",
                                "GSM" => "LG",
                                "ACR" => "Acer",
                                "AUS" => "ASUS",
                                _ => &vendor,
                            };
                            return format!("{vendor} {model}");
                        }
                    }
                }
            }
        }
    }
    format!("Monitor {}", index + 1)
}

pub fn find_recommended_presentation_monitor(
    monitors: &[MonitorInfo],
    current_name: &str,
) -> usize {
    if monitors.len() <= 1 {
        return monitors.first().map(|m| m.winit_index).unwrap_or(0);
    }

    // Procura monitor diferente do monitor atual onde a janela de controle está
    if !current_name.is_empty()
        && let Some(opposite) = monitors.iter().find(|m| m.name != current_name) {
            return opposite.winit_index;
        }

    // Se não encontrou por nome, seleciona o segundo monitor da lista
    if monitors.len() > 1 {
        monitors[1].winit_index
    } else {
        monitors[0].winit_index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_geometry_preserves_the_target_pixels_with_dpi_zoom_and_negative_coordinates() {
        let monitor = MonitorInfo {
            winit_index: 7,
            name: "left".into(),
            display_name: "Left monitor".into(),
            resolution: (3840, 2160),
            position: (-3840, -240),
            scale_factor: 1.5,
        };
        for zoom in [0.8, 1.0, 1.25, 1.5] {
            let rect = monitor.viewport_rect(zoom);
            let scale = monitor.scale_factor as f32 * zoom;
            assert!((rect.min.x * scale + 3840.0).abs() < 0.01);
            assert!((rect.min.y * scale + 240.0).abs() < 0.01);
            assert!((rect.width() * scale - 3840.0).abs() < 0.01);
            assert!((rect.height() * scale - 2160.0).abs() < 0.01);
        }
    }
}
