use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use eframe::egui;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusOptions {
    pub enabled: bool,
    pub cycle_status: bool,
    pub cycle_interval: u32,
    pub cycle_random: bool,
    pub enable_custom_prefix_shuffle: bool,
    pub custom_prefixes: String,
    pub add_speech_bubble: bool,
}
impl StatusOptions {
    pub fn new() -> Self {
        Self {
            enabled: true,
            cycle_status: false,
            cycle_interval: 60,
            cycle_random: false,
            enable_custom_prefix_shuffle: false,
            custom_prefixes: String::new(),
            add_speech_bubble: false,
        }
    }
    pub fn show_status_options(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let mut response = ui.interact(
            egui::Rect::EVERYTHING,
            ui.id().with("status_options"),
            egui::Sense::hover(),
        );
        response |= ui.checkbox(&mut self.cycle_status, "Cycle status messages");
        if self.cycle_status {
            ui.horizontal(|ui| {
                ui.label("Cycle interval: ");
                response |= ui.add(egui::DragValue::new(&mut self.cycle_interval).speed(1.0));
                ui.label(" seconds");
            });
            response |= ui.checkbox(&mut self.cycle_random, "Random cycle order");
        }
        response |= ui.checkbox(&mut self.enable_custom_prefix_shuffle, "Enable custom prefix shuffle");
        if self.enable_custom_prefix_shuffle {
            ui.label("Custom prefixes (comma-separated):");
            response |= ui.text_edit_singleline(&mut self.custom_prefixes);
        }
        response |= ui.checkbox(&mut self.add_speech_bubble, "Add 🗨 as prefix");
        response
    }
}
pub struct StatusModule {
    pub messages: Vec<String>,
    current_index: usize,
    last_cycle: std::time::Instant,
}
impl StatusModule {
    pub fn new() -> Self {
        Self {
            messages: vec![],
            current_index: 0,
            last_cycle: std::time::Instant::now(),
        }
    }
    pub fn add_message(&mut self, message: String) {
        self.messages.push(message);
    }
    pub fn remove_message(&mut self, index: usize) {
        if index < self.messages.len() {
            self.messages.remove(index);
            if self.current_index >= self.messages.len() && !self.messages.is_empty() {
                self.current_index = self.messages.len() - 1;
            }
        }
    }
    /*pub fn edit_message(&mut self, index: usize, new_message: String) {
        if index < self.messages.len() {
            self.messages[index] = new_message;
        }
    }*/
    pub fn get_current_message(&self, options: &StatusOptions) -> Option<String> {
        if self.messages.is_empty() {
            return None;
        }
        let mut message = self.messages[self.current_index].clone();
        if options.enable_custom_prefix_shuffle {
            let prefixes: Vec<&str> = options.custom_prefixes.split(',').collect();
            if !prefixes.is_empty() {
                if let Some(prefix) = prefixes.choose(&mut rand::thread_rng()) {
                    message = format!("{} {}", prefix.trim(), message);
                }
            }
        }
        if options.add_speech_bubble {
            message = format!("🗨 {}", message);
        }
        Some(message)
    }
    pub fn update_cycle(&mut self, options: &StatusOptions) {
        if !options.cycle_status || self.messages.is_empty() {
            return;
        }
        if self.last_cycle.elapsed().as_secs() >= options.cycle_interval as u64 {
            if options.cycle_random {
                self.current_index = rand::random::<usize>() % self.messages.len();
            } else {
                self.current_index = (self.current_index + 1) % self.messages.len();
            }
            self.last_cycle = std::time::Instant::now();
        }
    }
}