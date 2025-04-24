use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum Tab {
    Integrations,
    Status,
    Chatting,
    Options,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ChatTab {
    pub message: String,
    pub is_focused: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct IntegrationsTab {
    pub personal_status_enabled: bool,
    pub component_stats_enabled: bool,
    pub network_stats_enabled: bool,
    pub current_time_enabled: bool,
    pub medialink_enabled: bool,
    pub window_activity_enabled: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StatusTab {
    pub new_message: String,
}