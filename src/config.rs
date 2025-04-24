use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::ui::types::{ChatTab, StatusTab, Tab};
use crate::modules::{
    activity::WindowActivityOptions,
    app::AppOptions,
    chatting::ChatOptions,
    component::ComponentStatsOptions,
    extra::ExtraOptions,
    media::MediaLinkOptions,
    network::{NetworkStatsOptions, NetworkOptions},
    status::StatusOptions,
    time::TimeOptions,
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub app_options: AppOptions,
    pub personal_status_enabled: bool,
    pub component_stats_enabled: bool,
    pub network_stats_enabled: bool,
    pub current_time_enabled: bool,
    pub medialink_enabled: bool,
    pub window_activity_enabled: Option<bool>,
    pub window_activity_options: Option<WindowActivityOptions>,
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

impl Default for Config {
    fn default() -> Self {
        Config {
            app_options: AppOptions::default(),
            personal_status_enabled: true,
            component_stats_enabled: false,
            network_stats_enabled: false,
            current_time_enabled: true,
            medialink_enabled: true,
            window_activity_enabled: Some(true),
            window_activity_options: Some(WindowActivityOptions::default()),
            chat_options: ChatOptions::default(),
            chat_tab: ChatTab {
                message: String::new(),
                is_focused: false,
            },
            component_stats_options: ComponentStatsOptions::default(),
            extra_options: ExtraOptions::default(),
            media_link_options: MediaLinkOptions::default(),
            network_stats_options: NetworkStatsOptions::default(),
            status_options: StatusOptions::default(),
            status_tab: StatusTab {
                new_message: String::new(),
            },
            status_messages: Vec::new(),
            time_options: TimeOptions::default(),
            current_tab: Tab::Chatting,
            send_to_vrchat: true,
            live_edit_enabled: false,
        }
    }
}

impl Config {
    pub fn load_or_create<P: AsRef<Path>>(path: P, default_network_options: NetworkOptions) -> Self {
        let path = path.as_ref();
        if path.exists() {
            if let Ok(contents) = std::fs::read_to_string(path) {
                if let Ok(mut config) = serde_json::from_str::<Config>(&contents) {
                    config.network_stats_options = NetworkStatsOptions::new(default_network_options);
                    return config;
                }
            }
        }
        let mut config = Config::default();
        config.network_stats_options = NetworkStatsOptions::new(default_network_options);
        config
    }
}