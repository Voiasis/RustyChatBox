use serde::{Deserialize, Serialize};
use std::process::Command;
use eframe::egui;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaLinkOptions {
    pub enabled: bool,
    pub use_music_note_prefix: bool,
    pub show_pause_emoji: bool,
    pub auto_switch_state: bool,
    pub auto_switch_session: bool,
    pub forget_session_seconds: u32,
    pub show_progress: bool,
    pub seekbar_style: String,
}
impl MediaLinkOptions {
    pub fn new() -> Self {
        Self {
            enabled: true,
            use_music_note_prefix: false,
            show_pause_emoji: true,
            auto_switch_state: true,
            auto_switch_session: true,
            forget_session_seconds: 300,
            show_progress: true,
            seekbar_style: "Small numbers".to_string(),
        }
    }
    pub fn show_medialink_options(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let mut response = ui.interact(
            egui::Rect::EVERYTHING,
            ui.id().with("medialink_options"),
            egui::Sense::hover(),
        );
        ui.label("Basic options");
        ui.label(egui::RichText::new("Customize how your media looks in your chatbox").text_style(egui::TextStyle::Small));
        response |= ui.checkbox(&mut self.use_music_note_prefix, "Change 'Listening to:' prefix to 🎵");
        response |= ui.checkbox(&mut self.show_pause_emoji, "Show ⏸ when music is paused");
        response |= ui.checkbox(&mut self.auto_switch_state, "Auto switch when media state changes");
        response |= ui.checkbox(&mut self.auto_switch_session, "Auto switch when a new session is detected");
        ui.horizontal(|ui| {
            ui.label("Forget session after");
            response |= ui.add(egui::DragValue::new(&mut self.forget_session_seconds).speed(1.0));
            ui.label("seconds");
        });
        ui.label("Media progress bar");
        ui.label(egui::RichText::new("Customize how your seek bar looks").text_style(egui::TextStyle::Small));
        ui.label("Seekbar style");
        let combo_response = egui::ComboBox::from_label("")
            .selected_text(&self.seekbar_style)
            .show_ui(ui, |ui| {
                let seekbar_styles = ["Small numbers", "Custom", "None"];
                let mut combo_response = ui.selectable_value(&mut self.seekbar_style, seekbar_styles[0].to_string(), seekbar_styles[0]);
                for style in seekbar_styles.iter().skip(1) {
                    combo_response |= ui.selectable_value(&mut self.seekbar_style, style.to_string(), *style);
                }
                combo_response
            });
        response |= combo_response.response;
        response |= ui.checkbox(&mut self.show_progress, "Show media progress");
        response
    }
}
pub struct MediaLinkModule;
impl MediaLinkModule {
    pub fn new() -> Self {
        Self
    }
    pub fn get_formatted_track(&self, options: &MediaLinkOptions) -> Option<String> {
        let status = Command::new("playerctl")
            .arg("status")
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())?;
        if status == "Paused" {
            return Some(if options.show_pause_emoji {
                "⏸".to_string()
            } else {
                "Paused".to_string()
            });
        }
        let output = Command::new("playerctl")
            .arg("metadata")
            .arg("--format")
            .arg("{{artist}} - {{title}}")
            .output()
            .ok()?;
        let track = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if track.is_empty() {
            return None;
        }
        let prefix = if options.use_music_note_prefix {
            "🎵 "
        } else {
            "Listening to: "
        };
        Some(format!("{}{}", prefix, track))
    }
    pub fn is_playing(&self) -> bool {
        Command::new("playerctl")
            .arg("status")
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "Playing")
            .unwrap_or(false)
    }
    pub fn play_pause(&self) {
        Command::new("playerctl")
            .arg("play-pause")
            .output()
            .unwrap_or_else(|e| {
                eprintln!("Failed to toggle play/pause: {}", e);
                std::process::Output {
                    status: std::process::ExitStatus::default(),
                    stdout: vec![],
                    stderr: vec![],
                }
            });
    }
    pub fn next(&self) {
        Command::new("playerctl")
            .arg("next")
            .output()
            .unwrap_or_else(|e| {
                eprintln!("Failed to skip to next: {}", e);
                std::process::Output {
                    status: std::process::ExitStatus::default(),
                    stdout: vec![],
                    stderr: vec![],
                }
            });
    }
    pub fn previous(&self) {
        Command::new("playerctl")
            .arg("previous")
            .output()
            .unwrap_or_else(|e| {
                eprintln!("Failed to go to previous: {}", e);
                std::process::Output {
                    status: std::process::ExitStatus::default(),
                    stdout: vec![],
                    stderr: vec![],
                }
            });
    }
    pub fn seek(&self, position: f32) {
        Command::new("playerctl")
            .arg("position")
            .arg(position.to_string())
            .output()
            .unwrap_or_else(|e| {
                eprintln!("Failed to seek: {}", e);
                std::process::Output {
                    status: std::process::ExitStatus::default(),
                    stdout: vec![],
                    stderr: vec![],
                }
            });
    }
    pub fn get_position(&self) -> Option<f32> {
        let output = Command::new("playerctl")
            .arg("metadata")
            .arg("--format")
            .arg("{{position}}")
            .output()
            .ok()?;
        let binding = String::from_utf8_lossy(&output.stdout);
        let pos_str = binding.trim();
        pos_str.parse::<f32>().ok().map(|p| p / 1_000_000.0)
    }
    pub fn get_duration(&self) -> Option<f32> {
        let output = Command::new("playerctl")
            .arg("metadata")
            .arg("--format")
            .arg("{{mpris:length}}")
            .output()
            .ok()?;
        let binding = String::from_utf8_lossy(&output.stdout);
        let dur_str = binding.trim();
        dur_str.parse::<f32>().ok().map(|d| d / 1_000_000.0)
    }
}