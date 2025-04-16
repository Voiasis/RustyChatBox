// netw_mod.rs
use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkOptions {
    pub use_interface_max_speed: bool,
    pub show_download_speed: bool,
    pub show_upload_speed: bool,
    pub show_max_download: bool,
    pub show_max_upload: bool,
    pub show_total_download: bool,
    pub show_total_upload: bool,
    pub show_utilization: bool,
    pub stylized_chars: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatsOptions {
    pub enabled: bool,
    pub config: NetworkOptions,
}

impl NetworkStatsOptions {
    pub fn new(config: NetworkOptions) -> Self {
        Self {
            enabled: true,
            config,
        }
    }

    pub fn show_network_stats_options(&mut self, ui: &mut eframe::egui::Ui) -> eframe::egui::Response {
        let mut response = ui.interact(
            eframe::egui::Rect::EVERYTHING,
            ui.id().with("network_stats_options"),
            eframe::egui::Sense::hover(),
        );
        response |= ui.checkbox(&mut self.config.use_interface_max_speed, "Use network interface as the max speed");
        response |= ui.checkbox(&mut self.config.show_download_speed, "Show current download speed");
        response |= ui.checkbox(&mut self.config.show_upload_speed, "Show current upload speed");
        response |= ui.checkbox(&mut self.config.show_max_download, "Show max download speed");
        response |= ui.checkbox(&mut self.config.show_max_upload, "Show max upload speed");
        response |= ui.checkbox(&mut self.config.show_total_download, "Show total download");
        response |= ui.checkbox(&mut self.config.show_total_upload, "Show total upload");
        response |= ui.checkbox(&mut self.config.show_utilization, "Show network utilization");
        response |= ui.checkbox(&mut self.config.stylized_chars, "Stylized characters");
        response
    }
}

pub struct NetworkStats;

impl NetworkStats {
    pub fn get_interfaces() -> Vec<NetworkInterface> {
        NetworkInterface::show()
            .map_err(|e| eprintln!("Failed to get network interfaces: {}", e))
            .unwrap_or_default()
    }

    pub fn get_interface_stats(interface_name: &str) -> Option<NetworkInterface> {
        Self::get_interfaces()
            .into_iter()
            .find(|iface| iface.name == interface_name)
    }

    pub fn get_download_speed(_interface_name: &str) -> Option<f64> {
        None
    }

    pub fn get_upload_speed(_interface_name: &str) -> Option<f64> {
        None
    }

    pub fn get_max_download_speed(_interface_name: &str) -> Option<f64> {
        None
    }

    pub fn get_max_upload_speed(_interface_name: &str) -> Option<f64> {
        None
    }

    pub fn get_total_download(_interface_name: &str) -> Option<u64> {
        None
    }

    pub fn get_total_upload(_interface_name: &str) -> Option<u64> {
        None
    }

    pub fn get_utilization(_interface_name: &str) -> Option<f32> {
        None
    }

    pub fn get_formatted_stats(options: &NetworkOptions, interface_name: &str) -> String {
        let mut parts = Vec::new();

        if options.show_download_speed {
            if let Some(speed) = Self::get_download_speed(interface_name) {
                let text = format!("Download: {:.2} MB/s", speed);
                parts.push(if options.stylized_chars { text.to_uppercase() } else { text });
            }
        }
        if options.show_upload_speed {
            if let Some(speed) = Self::get_upload_speed(interface_name) {
                let text = format!("Upload: {:.2} MB/s", speed);
                parts.push(if options.stylized_chars { text.to_uppercase() } else { text });
            }
        }
        if options.show_max_download {
            if let Some(speed) = Self::get_max_download_speed(interface_name) {
                let text = format!("Max Download: {:.2} MB/s", speed);
                parts.push(if options.stylized_chars { text.to_uppercase() } else { text });
            }
        }
        if options.show_max_upload {
            if let Some(speed) = Self::get_max_upload_speed(interface_name) {
                let text = format!("Max Upload: {:.2} MB/s", speed);
                parts.push(if options.stylized_chars { text.to_uppercase() } else { text });
            }
        }
        if options.show_total_download {
            if let Some(total) = Self::get_total_download(interface_name) {
                let text = format!("Total Download: {} MB", total / 1_048_576);
                parts.push(if options.stylized_chars { text.to_uppercase() } else { text });
            }
        }
        if options.show_total_upload {
            if let Some(total) = Self::get_total_upload(interface_name) {
                let text = format!("Total Upload: {} MB", total / 1_048_576);
                parts.push(if options.stylized_chars { text.to_uppercase() } else { text });
            }
        }
        if options.show_utilization {
            if let Some(util) = Self::get_utilization(interface_name) {
                let text = format!("Utilization: {}%", util);
                parts.push(if options.stylized_chars { text.to_uppercase() } else { text });
            }
        }

        if parts.is_empty() {
            "".to_string()
        } else {
            parts.join("\n")
        }
    }
}