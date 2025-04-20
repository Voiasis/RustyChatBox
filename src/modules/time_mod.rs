use chrono::Local;
use chrono_tz::Tz;
use log::error;
use serde::{Deserialize, Serialize};
use eframe::egui;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeConfig {
    pub enabled: bool,
    pub show_my_time_prefix: bool,
    pub use_24_hour: bool,
    pub use_system_culture: bool,
    pub auto_dst: bool,
    pub custom_timezone: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeOptions {
    pub config: TimeConfig,
}
impl TimeOptions {
    pub fn show_time_options(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let mut response = ui.interact(
            egui::Rect::EVERYTHING,
            ui.id().with("time_options"),
            egui::Sense::hover(),
        );
        response |= ui.checkbox(&mut self.config.show_my_time_prefix, "Show 'My time:' in front of the time integration");
        response |= ui.checkbox(&mut self.config.use_24_hour, "24-Hour time format");
        response |= ui.checkbox(&mut self.config.use_system_culture, "Use current system culture for formatting time");
        response |= ui.checkbox(&mut self.config.auto_dst, "Auto daylight savings time");
        let mut use_custom_tz = self.config.custom_timezone.is_some();
        response |= ui.checkbox(&mut use_custom_tz, "Custom time zone");
        if use_custom_tz {
            let mut tz_str = self.config.custom_timezone.clone().unwrap_or_default();
            let combo_response = egui::ComboBox::from_label("")
                .selected_text(&tz_str)
                .show_ui(ui, |ui| {
                    for tz in chrono_tz::TZ_VARIANTS.iter() {
                        if ui.selectable_value(&mut tz_str, tz.to_string(), tz.to_string()).changed() {
                            response.mark_changed();
                        }
                    }
                });
            response |= combo_response.response;
            self.config.custom_timezone = Some(tz_str);
        } else {
            self.config.custom_timezone = None;
        }
        response
    }
}
pub struct TimeModule;
impl TimeModule {
    /*pub fn new() -> Self {
        Self
    }*/
    pub fn get_local_time(options: &TimeOptions) -> String {
        let now = Local::now();
        let time_str = match &options.config.custom_timezone {
            Some(tz_str) => {
                match tz_str.parse::<Tz>() {
                    Ok(tz) => {
                        let time_in_tz = now.with_timezone(&tz);
                        let format_str = if options.config.use_24_hour {
                            "%H:%M"
                        } else {
                            "%I:%M %p"
                        };
                        time_in_tz.format(format_str).to_string()
                    }
                    Err(e) => {
                        error!("Invalid timezone {}: {}", tz_str, e);
                        now.format("%H:%M").to_string()
                    }
                }
            }
            None => {
                let format_str = if options.config.use_24_hour {
                    "%H:%M"
                } else {
                    "%I:%M %p"
                };
                now.format(format_str).to_string()
            }
        };
        if options.config.show_my_time_prefix {
            format!("My time: {}", time_str)
        } else {
            time_str
        }
    }
}