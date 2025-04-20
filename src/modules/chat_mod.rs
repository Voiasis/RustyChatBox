use eframe::egui;
use serde::{Serialize, Deserialize};
use std::collections::VecDeque;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub text: String,
    pub sent_at_ms: u64,
    pub editing: bool,
    pub edit_text: String,
}
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
    pub messages: VecDeque<ChatMessage>,
    pub last_send_ms: Option<u64>,
    pub queued_message: Option<String>,
}
impl ChatOptions {
    pub fn add_message(&mut self, text: String) {
        if self.messages.len() >= 10 {
            self.messages.pop_front();
        }
        self.messages.push_back(ChatMessage {
            text: text.clone(),
            sent_at_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            editing: false,
            edit_text: text,
        });
    }
    pub fn clear_messages(&mut self) {
        self.messages.clear();
    }
    pub fn can_send(&self) -> bool {
        if !self.small_delay {
            return true;
        }
        match self.last_send_ms {
            Some(last_ms) => {
                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;
                (now_ms - last_ms) as f32 / 1000.0 >= self.delay_seconds
            }
            None => true,
        }
    }
    pub fn set_queued_message(&mut self, message: String) {
        self.queued_message = Some(message);
    }
    pub fn take_queued_message(&mut self) -> Option<String> {
        self.last_send_ms = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        );
        self.queued_message.take()
    }
    pub fn get_remaining_time(&self, message: &ChatMessage) -> u32 {
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let elapsed = ((now_ms - message.sent_at_ms) / 1000) as u32;
        if elapsed >= self.chat_timeout {
            0
        } else {
            self.chat_timeout - elapsed
        }
    }
    pub fn show_chatting_options(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let mut response = ui.interact(
            egui::Rect::EVERYTHING,
            ui.id().with("chat_options"),
            egui::Sense::hover(),
        );
        ui.horizontal(|ui| {
            ui.label("Chat timeout ");
            response |= ui.add(egui::DragValue::new(&mut self.chat_timeout).speed(1.0));
            ui.label(" seconds");
        });
        response |= ui.checkbox(&mut self.add_speech_bubble, "Add 🗨 as prefix for chat messages");
        response |= ui.checkbox(&mut self.use_custom_idle_prefix, "Use your custom idle icon shuffle as a prefix for chat messages");
        response |= ui.checkbox(&mut self.play_fx_sound, "Play FX sound for VRChat users when sending messages");
        if self.play_fx_sound {
            response |= ui.checkbox(&mut self.play_fx_resend, "Play FX when clicking resend");
        }
        response |= ui.checkbox(&mut self.small_delay, "Small delay when sending a message");
        if self.small_delay {
            response |= ui.add(egui::Slider::new(&mut self.delay_seconds, 0.1..=2.0).text("seconds"));
        }
        response |= ui.checkbox(&mut self.override_display_time, "Override display time for chat messages (keeps updating)");
        if self.override_display_time {
            response |= ui.add(egui::Slider::new(&mut self.display_time_seconds, 2.0..=10.0).text("seconds"));
        }
        if self.override_display_time {
            response |= ui.checkbox(&mut self.edit_messages, "Edit chat messages");
            if self.edit_messages {
                response |= ui.checkbox(&mut self.live_editing, "Live editing");
            }
        }
        response
    }
}