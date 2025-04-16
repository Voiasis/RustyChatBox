// time_mod.rs
use chrono::Local;
use chrono_tz::Tz;
use log::error;
use serde::{Deserialize, Serialize};

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
    pub fn new() -> Self {
        Self {
            config: TimeConfig {
                enabled: true,
                show_my_time_prefix: true,
                use_24_hour: false,
                use_system_culture: true,
                auto_dst: true,
                custom_timezone: None,
            },
        }
    }
}

pub struct TimeModule;

impl TimeModule {
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