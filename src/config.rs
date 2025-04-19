use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::modules::{
    appo_mod::AppOptions,
    chat_mod::ChatOptions,
    comp_mod::ComponentStatsOptions,
    extr_mod::ExtraOptions,
    medi_mod::MediaLinkOptions,
    netw_mod::{NetworkOptions, NetworkStatsOptions},
    stat_mod::StatusOptions,
    time_mod::TimeOptions,
};
use crate::gui::{ChatTab, StatusTab, Tab};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub app_options: AppOptions,
    pub personal_status_enabled: bool,
    pub component_stats_enabled: bool,
    pub network_stats_enabled: bool,
    pub current_time_enabled: bool,
    pub medialink_enabled: bool,
    pub chat_options: ChatOptions,
    pub chat_tab: ChatTab,
    pub component_stats_options: ComponentStatsOptions,
    pub extra_options: ExtraOptions,
    pub media_link_options: MediaLinkOptions,
    pub network_stats_options: NetworkStatsOptions,
    pub status_options: StatusOptions,
    pub status_tab: StatusTab,
    pub status_messages: Vec<String>,
    pub time_options: TimeOptions,
    pub current_tab: Tab,
    pub send_to_vrchat: bool,
    pub live_edit_enabled: bool,
}

impl Config {
    pub fn load_or_create(path: &Path, default_network_options: NetworkOptions) -> Self {
        if path.exists() {
            match fs::read_to_string(path) {
                Ok(config_str) => match serde_json::from_str::<Config>(&config_str) {
                    Ok(mut config) => {
                        config.app_options.osc_options.update_rate = config
                            .app_options
                            .osc_options
                            .update_rate
                            .clamp(1.6, 10.0);
                        return config;
                    }
                    Err(e) => eprintln!("Failed to parse config: {}", e),
                },
                Err(e) => eprintln!("Failed to read config: {}", e),
            }
        }
        let config = Config {
            app_options: AppOptions::new(),
            personal_status_enabled: true,
            component_stats_enabled: true,
            network_stats_enabled: true,
            current_time_enabled: true,
            medialink_enabled: true,
            chat_options: ChatOptions::new(),
            chat_tab: ChatTab {
                message: String::new(),
                is_focused: false,
            },
            component_stats_options: ComponentStatsOptions::new(),
            extra_options: ExtraOptions::new(),
            media_link_options: MediaLinkOptions::new(),
            network_stats_options: NetworkStatsOptions::new(default_network_options),
            status_options: StatusOptions::new(),
            status_tab: StatusTab {
                new_message: String::new(),
            },
            status_messages: Vec::new(),
            time_options: TimeOptions::new(),
            current_tab: Tab::Integrations,
            send_to_vrchat: false,
            live_edit_enabled: false,
        };
        if let Ok(json) = serde_json::to_string_pretty(&config) {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Err(e) = fs::write(path, json) {
                eprintln!("Failed to write default config: {}", e);
            }
        }
        config
    }
}