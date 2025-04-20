use serde::{Serialize, Deserialize};
use eframe::egui;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtraOptions {
    pub enabled: bool,
    pub slim_mode: bool,
}
impl ExtraOptions {
    pub fn show_extra_options(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let mut response = ui.interact(
            egui::Rect::EVERYTHING,
            ui.id().with("extra_options"),
            egui::Sense::hover(),
        );
        response |= ui.checkbox(&mut self.slim_mode, "Enable slim mode");
        response
    }
}