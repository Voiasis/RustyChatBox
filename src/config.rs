use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::collections::VecDeque;
use crate::modules::{
    appo_mod::{AppOptions, OscOptions},
    comp_mod::ComponentStatsOptions,
    extr_mod::ExtraOptions,
    chat_mod::ChatOptions,
    medi_mod::MediaLinkOptions,
    netw_mod::{NetworkOptions, NetworkStatsOptions},
    stat_mod::StatusOptions,
    time_mod::{TimeConfig, TimeOptions},
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
            app_options: AppOptions {
                osc_options: OscOptions {
                    ip: "127.0.0.1".to_string(),
                    port: 9000,
                    update_rate: 1.6,
                    separate_lines: true,
                },
            },
            personal_status_enabled: true,
            component_stats_enabled: true,
            network_stats_enabled: false,
            current_time_enabled: true,
            medialink_enabled: true,
            chat_options: ChatOptions {
                enabled: true,
                chat_timeout: 30,
                add_speech_bubble: true,
                use_custom_idle_prefix: false,
                play_fx_sound: false,
                play_fx_resend: false,
                small_delay: true,
                delay_seconds: 0.5,
                override_display_time: false,
                display_time_seconds: 5.0,
                edit_messages: false,
                live_editing: false,
                messages: VecDeque::new(),
                last_send_ms: Some(1745101212543),
                queued_message: None,
            },
            chat_tab: ChatTab {
                message: String::new(),
                is_focused: false,
            },
            component_stats_options: ComponentStatsOptions {
                enabled: true,
                show_cpu: true,
                show_gpu: true,
                show_vram: false,
                show_ram: false,
                cpu_display_model: false,
                cpu_custom_model: None,
                cpu_round_usage: true,
                cpu_stylized_uppercase: false,
                gpu_display_model: false,
                gpu_custom_model: None,
                gpu_round_usage: true,
                gpu_stylized_uppercase: false,
                vram_round_usage: false,
                vram_show_max: false,
                vram_stylized_uppercase: false,
                ram_round_usage: false,
                ram_show_max: false,
                ram_stylized_uppercase: false,
            },
            extra_options: ExtraOptions {
                enabled: true,
                slim_mode: true,
            },
            media_link_options: MediaLinkOptions {
                enabled: true,
                use_music_note_prefix: true,
                show_pause_emoji: true,
                auto_switch_state: true,
                auto_switch_session: true,
                forget_session_seconds: 300,
                show_progress: true,
                seekbar_style: "Small numbers".to_string(),
            },
            network_stats_options: NetworkStatsOptions {
                enabled: true,
                config: default_network_options,
            },
            status_options: StatusOptions {
                enabled: true,
                cycle_status: false,
                cycle_interval: 60,
                cycle_random: false,
                enable_custom_prefix_shuffle: false,
                custom_prefixes: String::new(),
                add_speech_bubble: true,
            },
            status_tab: StatusTab {
                new_message: String::new(),
            },
            status_messages: vec!["Powered by RustyChatBox!".to_string()],
            time_options: TimeOptions {
                config: TimeConfig {
                    enabled: true,
                    show_my_time_prefix: true,
                    use_24_hour: false,
                    use_system_culture: true,
                    auto_dst: true,
                    custom_timezone: None,
                },
            },
            current_tab: Tab::Integrations,
            send_to_vrchat: true,
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