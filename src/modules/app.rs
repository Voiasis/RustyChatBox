use serde::{Serialize, Deserialize};
use eframe::egui;

#[derive(Clone, Serialize, Deserialize)]
pub struct OscOptions {
    pub ip: String,
    pub port: u16,
    pub update_rate: f32,
    pub separate_lines: bool,
}

impl Default for OscOptions {
    fn default() -> Self {
        OscOptions {
            ip: "127.0.0.1".to_string(),
            port: 9000,
            update_rate: 1.6,
            separate_lines: true,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AppOptions {
    pub osc_options: OscOptions,
}

impl Default for AppOptions {
    fn default() -> Self {
        AppOptions {
            osc_options: OscOptions::default(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AppOptionsOptions {
    pub app_options: AppOptions,
    pub enabled: bool,
}

impl Default for AppOptionsOptions {
    fn default() -> Self {
        AppOptionsOptions {
            app_options: AppOptions::default(),
            enabled: true,
        }
    }
}

impl AppOptionsOptions {
    pub fn show_app_options(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let mut response = ui.interact(
            egui::Rect::EVERYTHING,
            ui.id().with("app_options"),
            egui::Sense::hover(),
        );
        ui.horizontal(|ui| {
            ui.label("OSC IP: ");
            response |= ui.text_edit_singleline(&mut self.app_options.osc_options.ip);
        });
        ui.horizontal(|ui| {
            ui.label("OSC Port: ");
            response |= ui.add(egui::DragValue::new(&mut self.app_options.osc_options.port).speed(1.0));
        });
        ui.horizontal(|ui| {
            ui.label("Update Rate: ");
            response |= ui.add(
                egui::Slider::new(&mut self.app_options.osc_options.update_rate, 1.6..=10.0)
                    .step_by(0.1)
                    .text("seconds"),
            );
        });
        response |= ui.checkbox(
            &mut self.app_options.osc_options.separate_lines,
            "Separate lines in OSC output",
        );
        response
    }
}