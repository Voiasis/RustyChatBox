// stat_mod.rs
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusOptions {
    pub enabled: bool,
    pub cycle_status: bool,
    pub cycle_interval: u32,
    pub cycle_random: bool,
    pub enable_custom_prefix_shuffle: bool,
    pub custom_prefixes: String,
    pub add_speech_bubble: bool,
}

impl StatusOptions {
    pub fn new() -> Self {
        Self {
            enabled: true,
            cycle_status: false,
            cycle_interval: 60,
            cycle_random: false,
            enable_custom_prefix_shuffle: false,
            custom_prefixes: String::new(),
            add_speech_bubble: false,
        }
    }
}

pub struct StatusModule {
    pub messages: Vec<String>,
    current_index: usize,
    last_cycle: std::time::Instant,
}

impl StatusModule {
    pub fn new() -> Self {
        Self {
            messages: vec![],
            current_index: 0,
            last_cycle: std::time::Instant::now(),
        }
    }

    pub fn add_message(&mut self, message: String) {
        self.messages.push(message);
    }

    pub fn remove_message(&mut self, index: usize) {
        if index < self.messages.len() {
            self.messages.remove(index);
            if self.current_index >= self.messages.len() && !self.messages.is_empty() {
                self.current_index = self.messages.len() - 1;
            }
        }
    }

    pub fn edit_message(&mut self, index: usize, new_message: String) {
        if index < self.messages.len() {
            self.messages[index] = new_message;
        }
    }

    pub fn get_current_message(&self, options: &StatusOptions) -> Option<String> {
        if self.messages.is_empty() {
            return None;
        }
        let mut message = self.messages[self.current_index].clone();
        if options.enable_custom_prefix_shuffle {
            let prefixes: Vec<&str> = options.custom_prefixes.split(',').collect();
            if !prefixes.is_empty() {
                if let Some(prefix) = prefixes.choose(&mut rand::thread_rng()) {
                    message = format!("{} {}", prefix.trim(), message);
                }
            }
        }
        if options.add_speech_bubble {
            message = format!("🗨 {}", message);
        }
        Some(message)
    }

    pub fn update_cycle(&mut self, options: &StatusOptions) {
        if !options.cycle_status || self.messages.is_empty() {
            return;
        }
        if self.last_cycle.elapsed().as_secs() >= options.cycle_interval as u64 {
            if options.cycle_random {
                self.current_index = rand::random::<usize>() % self.messages.len();
            } else {
                self.current_index = (self.current_index + 1) % self.messages.len();
            }
            self.last_cycle = std::time::Instant::now();
        }
    }
}