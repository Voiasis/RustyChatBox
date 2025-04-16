// chat_mod.rs
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatOptions {
    pub enabled: bool,
    pub chat_timeout: u32,
    pub add_speech_bubble: bool,
    pub use_custom_idle_prefix: bool,
    pub play_fx_sound: bool,
    pub play_fx_resend: bool,
    pub small_delay: bool,
    pub delay_seconds: f32,
    pub override_display_time: bool,
    pub display_time_seconds: f32,
    pub edit_messages: bool,
    pub live_editing: bool,
}

impl ChatOptions {
    pub fn new() -> Self {
        Self {
            enabled: true,
            chat_timeout: 30,
            add_speech_bubble: false,
            use_custom_idle_prefix: false,
            play_fx_sound: false,
            play_fx_resend: false,
            small_delay: false,
            delay_seconds: 0.5,
            override_display_time: false,
            display_time_seconds: 5.0,
            edit_messages: false,
            live_editing: false,
        }
    }
}