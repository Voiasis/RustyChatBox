// appo_mod.rs
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscOptions {
    pub ip: String,
    pub port: u16,
    pub update_rate: f32,
    pub separate_lines: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppOptions {
    pub osc_options: OscOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppOptionsOptions {
    pub enabled: bool,
    pub app_options: AppOptions,
}

impl AppOptions {
    pub fn new() -> Self {
        Self {
            osc_options: OscOptions {
                ip: "127.0.0.1".to_string(),
                port: 9000,
                update_rate: 1.6,
                separate_lines: false,
            },
        }
    }
}

impl AppOptionsOptions {
    pub fn new() -> Self {
        Self {
            enabled: true,
            app_options: AppOptions::new(),
        }
    }
}