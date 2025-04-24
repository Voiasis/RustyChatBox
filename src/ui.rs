pub(crate) mod types;
mod integrations;
mod status;
mod chatting;
mod options;
mod toggle;

use chatting::show_chatting_tab;
use eframe::egui::{self, Color32, Context, Rounding, Stroke};
use integrations::show_integrations_tab;
use options::show_options_tab;
use status::show_status_tab;
use toggle::toggle_switch;
use types::{ChatTab, IntegrationsTab, StatusTab, Tab};
use std::fs;
use std::time::{Instant, Duration};
use log::{error, info, debug};
use arboard::Clipboard;
use crate::config::Config;
use crate::osc::OscClient;
use crate::modules::{
    activity::{WindowActivityModule, WindowActivityOptions},
    app::AppOptionsOptions,
    chatting::ChatOptions,
    component::{ComponentStatsModule, ComponentStatsOptions},
    extra::ExtraOptions,
    media::{MediaLinkModule, MediaLinkOptions},
    network::{NetworkStats, NetworkStatsOptions},
    status::{StatusModule, StatusOptions},
    time::{TimeModule, TimeOptions},
};
pub struct App {
    current_tab: Tab,
    app_options: AppOptionsOptions,
    chat_tab: ChatTab,
    chat_options: ChatOptions,
    component_stats: ComponentStatsOptions,
    extra_options: ExtraOptions,
    integrations_tab: IntegrationsTab,
    media_link: MediaLinkOptions,
    network_stats: NetworkStatsOptions,
    status_tab: StatusTab,
    status_options: StatusOptions,
    time_options: TimeOptions,
    window_activity: WindowActivityOptions,
    osc_client: OscClient,
    components_module: ComponentStatsModule,
    media_module: MediaLinkModule,
    status_module: StatusModule,
    window_activity_module: WindowActivityModule,
    osc_preview: String,
    send_to_vrchat: bool,
    last_osc_send: Instant,
    config_changed: bool,
    pending_scroll_to: Option<egui::Id>,
    clipboard: Clipboard,
    live_edit_enabled: bool,
    previous_osc_preview: String,
    last_activity_update: Instant,
    cached_activity: Option<String>,
    first_update: bool,
}

impl App {
    pub fn new(osc_client: OscClient, config: Config, clipboard: Clipboard) -> Self {
        let mut app_options = AppOptionsOptions {
            app_options: config.app_options,
            enabled: true,
        };
        app_options.app_options.osc_options.update_rate = app_options
            .app_options
            .osc_options
            .update_rate
            .clamp(1.6, 10.0);
    
        let mut status_module = StatusModule::new();
        for message in config.status_messages {
            status_module.add_message(message);
        }
    
        let window_activity_options = config.window_activity_options.unwrap_or_default();
        let window_activity_enabled = config.window_activity_enabled.unwrap_or(true);
    
        info!("Initializing App with OSC client and config");
        Self {
            current_tab: config.current_tab,
            app_options,
            chat_tab: config.chat_tab,
            chat_options: config.chat_options,
            component_stats: config.component_stats_options,
            extra_options: config.extra_options,
            integrations_tab: IntegrationsTab {
                personal_status_enabled: config.personal_status_enabled,
                component_stats_enabled: config.component_stats_enabled,
                network_stats_enabled: config.network_stats_enabled,
                current_time_enabled: config.current_time_enabled,
                medialink_enabled: config.medialink_enabled,
                window_activity_enabled,
            },
            media_link: config.media_link_options,
            network_stats: NetworkStatsOptions::new(config.network_stats_options.config),
            status_tab: config.status_tab,
            status_options: config.status_options,
            time_options: config.time_options,
            window_activity: WindowActivityOptions {
                enabled: window_activity_enabled,
                ..window_activity_options.clone()
            },
            osc_client,
            components_module: ComponentStatsModule::new(),
            media_module: MediaLinkModule::new(),
            status_module,
            window_activity_module: WindowActivityModule::new(&window_activity_options),
            osc_preview: String::new(),
            send_to_vrchat: config.send_to_vrchat,
            last_osc_send: Instant::now(),
            config_changed: false,
            pending_scroll_to: None,
            clipboard,
            live_edit_enabled: config.live_edit_enabled,
            previous_osc_preview: String::new(),
            last_activity_update: Instant::now(),
            cached_activity: None,
            first_update: true,
        }
    }

    fn update_osc_preview(&mut self) {
        debug!("Updating OSC preview");
        self.status_module.update_cycle(&self.status_options);
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
    
        const MAX_LINE_WIDTH: usize = 27;
    
        let should_update = self.send_to_vrchat
            && self.chat_options.can_send()
            && self.last_osc_send.elapsed().as_secs_f32() >= self.app_options.app_options.osc_options.update_rate;
    
        let should_update_live = self.send_to_vrchat
            && self.chat_options.live_editing
            && self.chat_options.override_display_time
            && self.last_osc_send.elapsed().as_secs_f32() >= self.app_options.app_options.osc_options.update_rate.max(self.chat_options.display_time_seconds);
    
        let active_chat_message = self.chat_options.messages.iter_mut().find(|m| m.editing);
    
        if should_update || should_update_live {
            let mut parts = Vec::new();
    
            // Status
            if self.integrations_tab.personal_status_enabled {
                if let Some(status) = self.status_module.get_current_message(&self.status_options) {
                    parts.push(status);
                }
            }
    
            // Window Activity
            if self.integrations_tab.window_activity_enabled {
                if let Some(activity) = self.window_activity_module.get_formatted_activity(&self.window_activity) {
                    parts.push(activity);
                }
            }
    
            // Component Stats
            if self.integrations_tab.component_stats_enabled {
                let stats = self.components_module.get_formatted_stats(&self.component_stats);
                if !stats.is_empty() {
                    let stat_parts: Vec<&str> = stats.split('|').collect();
                    let mut stat_pairs = Vec::new();
                    for chunk in stat_parts.chunks(2) {
                        if chunk.len() == 2 {
                            stat_pairs.push(format!("{} | {}", chunk[0].trim(), chunk[1].trim()));
                        } else {
                            stat_pairs.push(chunk[0].trim().to_string());
                        }
                    }
                    parts.extend(stat_pairs);
                }
            }
    
            // Time
            if self.integrations_tab.current_time_enabled {
                let time = TimeModule::get_local_time(&self.time_options);
                parts.push(time);
            }
    
            // Network Stats
            if self.integrations_tab.network_stats_enabled {
                let interfaces = NetworkStats::get_interfaces();
                if let Some(iface) = interfaces.first() {
                    let stats = NetworkStats::get_formatted_stats(&self.network_stats.config, &iface.name);
                    if !stats.is_empty() {
                        parts.push(stats);
                    }
                }
            }
    
            // MediaLink
            if self.integrations_tab.medialink_enabled {
                if let Some(track) = self.media_module.get_formatted_track(&self.media_link) {
                    parts.push(track);
                }
            }
    
            let separator = if self.app_options.app_options.osc_options.separate_lines {
                "\n"
            } else {
                " | "
            };
            let mut lines = Vec::new();
    
            for (i, part) in parts.iter().enumerate() {
                let is_last_part = i == parts.len() - 1;
    
                let part_lines = part.split('\n').collect::<Vec<_>>();
    
                for part_line in part_lines {
                    let trimmed_line = part_line.trim();
                    if trimmed_line.is_empty() {
                        continue;
                    }
    
                    let mut current_line = String::new();
                    let words = trimmed_line.split_whitespace();
    
                    for word in words {
                        let word_len = word.len();
                        let space_len = if current_line.is_empty() { 0 } else { 1 };
                        let potential_len = current_line.len() + space_len + word_len;
    
                        if potential_len > MAX_LINE_WIDTH && !current_line.is_empty() {
                            lines.push(current_line);
                            current_line = word.to_string();
                        } else if word_len > MAX_LINE_WIDTH {
                            if !current_line.is_empty() {
                                lines.push(current_line);
                            }
                            lines.push(word[..MAX_LINE_WIDTH].to_string());
                            current_line = String::new();
                        } else {
                            if !current_line.is_empty() {
                                current_line.push(' ');
                            }
                            current_line.push_str(word);
                        }
                    }
    
                    if !current_line.is_empty() {
                        lines.push(current_line);
                    }
                }
    
                if !is_last_part && !self.app_options.app_options.osc_options.separate_lines {
                    if lines.last().map_or(true, |last| last.len() + separator.len() <= MAX_LINE_WIDTH) {
                        lines.last_mut().map(|last| *last += separator);
                    } else {
                        lines.push(separator.to_string());
                    }
                }
            }
    
            self.previous_osc_preview = lines.join("\n");
    
            if let Some(ref message) = active_chat_message {
                let message_text = if self.chat_options.live_editing && message.editing {
                    &message.edit_text
                } else {
                    &message.text
                };
                let formatted_message = if self.chat_options.add_speech_bubble {
                    format!("🗨 {}", message_text)
                } else {
                    message_text.clone()
                };
                let mut message_lines = Vec::new();
    
                let mut current_line = String::new();
                let words = formatted_message.split_whitespace();
                for word in words {
                    let word_len = word.len();
                    let space_len = if current_line.is_empty() { 0 } else { 1 };
                    let potential_len = current_line.len() + space_len + word_len;
    
                    if potential_len > MAX_LINE_WIDTH && !current_line.is_empty() {
                        message_lines.push(current_line);
                        current_line = word.to_string();
                    } else if word_len > MAX_LINE_WIDTH {
                        if !current_line.is_empty() {
                            message_lines.push(current_line);
                        }
                        message_lines.push(word[..MAX_LINE_WIDTH].to_string());
                        current_line = String::new();
                    } else {
                        if !current_line.is_empty() {
                            current_line.push(' ');
                        }
                        current_line.push_str(word);
                    }
                }
                if !current_line.is_empty() {
                    message_lines.push(current_line);
                }
    
                if !self.previous_osc_preview.is_empty() {
                    message_lines.push(self.previous_osc_preview.clone());
                }
                self.osc_preview = message_lines.join("\n");
            } else {
                self.osc_preview = self.previous_osc_preview.clone();
            }
        }
    
        if should_update_live {
            if let Some(ref message) = active_chat_message {
                if message.editing && self.send_to_vrchat && !self.osc_preview.is_empty() {
                    if (now_ms - message.sent_at_ms) / 1000 < self.chat_options.chat_timeout as u64 {
                        if let Err(e) = self.osc_client.send_chatbox_message(
                            &self.osc_preview,
                            self.chat_options.play_fx_sound,
                            self.extra_options.slim_mode,
                        ) {
                            error!("Failed to send live edit OSC message: {}", e);
                        } else {
                            info!("Sent live edit OSC message: {}", self.osc_preview);
                        }
                        self.last_osc_send = Instant::now();
                    }
                }
            }
        }
    
        if should_update && !self.osc_preview.is_empty() {
            if let Some(message) = self.chat_options.take_queued_message() {
                let formatted_message = if self.chat_options.add_speech_bubble {
                    format!("🗨 {}", message)
                } else {
                    message
                };
                let mut lines = Vec::new();
    
                let mut current_line = String::new();
                let words = formatted_message.split_whitespace();
                for word in words {
                    let word_len = word.len();
                    let space_len = if current_line.is_empty() { 0 } else { 1 };
                    let potential_len = current_line.len() + space_len + word_len;
    
                    if potential_len > MAX_LINE_WIDTH && !current_line.is_empty() {
                        lines.push(current_line);
                        current_line = word.to_string();
                    } else if word_len > MAX_LINE_WIDTH {
                        if !current_line.is_empty() {
                            lines.push(current_line);
                        }
                        lines.push(word[..MAX_LINE_WIDTH].to_string());
                        current_line = String::new();
                    } else {
                        if !current_line.is_empty() {
                            current_line.push(' ');
                        }
                        current_line.push_str(word);
                    }
                }
                if !current_line.is_empty() {
                    lines.push(current_line);
                }
    
                if !self.previous_osc_preview.is_empty() {
                    lines.push(self.previous_osc_preview.clone());
                }
                self.osc_preview = lines.join("\n");
    
                if self.send_to_vrchat {
                    if let Err(e) = self.osc_client.send_chatbox_message(
                        &self.osc_preview,
                        self.chat_options.play_fx_sound,
                        self.extra_options.slim_mode,
                    ) {
                        error!("Failed to send queued OSC message: {}", e);
                    } else {
                        info!("Sent queued OSC message: {}", self.osc_preview);
                    }
                    self.last_osc_send = Instant::now();
                }
                self.chat_options.add_message(formatted_message);
            } else if self.send_to_vrchat {
                if let Err(e) = self.osc_client.send_chatbox_message(
                    &self.osc_preview,
                    self.chat_options.play_fx_sound,
                    self.extra_options.slim_mode,
                ) {
                    error!("Failed to send OSC message: {}", e);
                }
                self.last_osc_send = Instant::now();
            }
        }
    }

    pub fn save_config_if_needed(&mut self, config_path: &std::path::Path) {
        if self.config_changed {
            debug!("Saving configuration to {}", config_path.display());
            let config = Config {
                app_options: self.app_options.app_options.clone(),
                personal_status_enabled: self.integrations_tab.personal_status_enabled,
                component_stats_enabled: self.integrations_tab.component_stats_enabled,
                network_stats_enabled: self.integrations_tab.network_stats_enabled,
                current_time_enabled: self.integrations_tab.current_time_enabled,
                medialink_enabled: self.integrations_tab.medialink_enabled,
                window_activity_enabled: Some(self.integrations_tab.window_activity_enabled),
                window_activity_options: Some(self.window_activity.clone()),
                chat_options: self.chat_options.clone(),
                chat_tab: self.chat_tab.clone(),
                component_stats_options: self.component_stats.clone(),
                extra_options: self.extra_options.clone(),
                media_link_options: self.media_link.clone(),
                network_stats_options: self.network_stats.clone(),
                status_options: self.status_options.clone(),
                status_tab: self.status_tab.clone(),
                status_messages: self.status_module.messages.clone(),
                time_options: self.time_options.clone(),
                current_tab: self.current_tab.clone(),
                send_to_vrchat: self.send_to_vrchat,
                live_edit_enabled: self.live_edit_enabled,
            };
            if let Ok(json) = serde_json::to_string_pretty(&config) {
                if let Err(e) = fs::write(config_path, json) {
                    error!("Failed to save config: {}", e);
                } else {
                    info!("Configuration saved successfully");
                }
            } else {
                error!("Failed to serialize config to JSON");
            }
            self.config_changed = false;
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {

        if self.first_update {
            self.first_update = false;
            debug!("First update completed");
        }

        if self.last_activity_update.elapsed() >= Duration::from_millis(500) {
            self.cached_activity = self.window_activity_module.get_formatted_activity(&self.window_activity);
            self.last_activity_update = Instant::now();
            debug!("Updated cached activity");
        }

        static mut LAST_UPDATE: Option<Instant> = None;
        let now = Instant::now();
        let should_update = unsafe {
            if let Some(last) = LAST_UPDATE {
                now.duration_since(last).as_secs_f32() >= self.app_options.app_options.osc_options.update_rate
            } else {
                true
            }
        };
    
        if should_update {
            self.update_osc_preview();
            unsafe { LAST_UPDATE = Some(now); }
        }
    
        // Set custom visual style
        let mut visuals = egui::Visuals::default();
        visuals.dark_mode = true;
    
        let hex_to_color = |hex: &str| {
            let hex = hex.trim_start_matches('#');
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
            Color32::from_rgb(r, g, b)
        };
    
        let border_color = hex_to_color("ff3f00");
        let background_color = hex_to_color("141414");
        let font_color = hex_to_color("ffffff");
        let disabled_color = hex_to_color("7200ff");
        let enabled_color = hex_to_color("ff3f00");
        let button_color = hex_to_color("3f3f3f");
        let slider_color = hex_to_color("ff3f00");
        let dropdown_color = hex_to_color("3f3f3f");
        let dropdown_outline_color = hex_to_color("b7410e");
        let dropdown_hover_color = hex_to_color("b7410e");
        let input_field_color = hex_to_color("000000");
        let scrollbar_color = hex_to_color("3f3f3f");
        let inactive_tab_color = hex_to_color("3f3f3f");
        let fallback_color = hex_to_color("7200ff");
    
        visuals.panel_fill = background_color;
        visuals.window_fill = dropdown_color;
        visuals.widgets.noninteractive.bg_fill = button_color;
        visuals.widgets.inactive.bg_fill = button_color;
        visuals.widgets.active.bg_fill = enabled_color;
        visuals.widgets.hovered.bg_fill = enabled_color;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, border_color);
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, border_color);
        visuals.widgets.active.bg_stroke = Stroke::new(1.0, border_color);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, border_color);
        visuals.window_stroke = Stroke::new(1.0, dropdown_outline_color);
        visuals.widgets.noninteractive.rounding = Rounding::same(4.0);
        visuals.override_text_color = Some(font_color);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, font_color);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, font_color);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, font_color);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, font_color);
        visuals.widgets.inactive.bg_fill = disabled_color;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, border_color);
        visuals.widgets.active.bg_fill = enabled_color;
        visuals.widgets.active.bg_stroke = Stroke::new(1.0, border_color);
        visuals.widgets.hovered.bg_fill = enabled_color;
        visuals.widgets.noninteractive.bg_fill = button_color;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, border_color);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, font_color);
        visuals.widgets.active.bg_fill = slider_color;
        visuals.widgets.hovered.bg_fill = slider_color;
        visuals.widgets.inactive.bg_fill = slider_color.gamma_multiply(0.5);
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, border_color);
        visuals.widgets.open.bg_fill = dropdown_color;
        visuals.widgets.open.bg_stroke = Stroke::new(1.0, dropdown_outline_color);
        visuals.widgets.open.fg_stroke = Stroke::new(1.0, font_color);
        visuals.selection.bg_fill = dropdown_hover_color;
        visuals.extreme_bg_color = input_field_color;
        visuals.selection.stroke = Stroke::new(1.0, font_color);
        visuals.widgets.noninteractive.bg_fill = scrollbar_color;
        visuals.widgets.inactive.bg_fill = scrollbar_color.gamma_multiply(0.5);
        visuals.widgets.noninteractive.bg_fill = button_color;
        visuals.hyperlink_color = fallback_color;
        visuals.faint_bg_color = fallback_color.gamma_multiply(0.5);
        visuals.code_bg_color = fallback_color.gamma_multiply(0.2);
    
        ctx.set_visuals(visuals);
    
        // Top panel with title, tabs, and VRChat toggle
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("VRC OSC").color(Color32::from_rgb(0x3f, 0x3f, 0x3f)));
                    ui.heading("RustyChatBox");
                });
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.add_space(20.0);
                    let mut button = egui::Button::new("Integrations").min_size(egui::vec2(100.0, 40.0));
                    if self.current_tab == Tab::Integrations {
                        button = button.fill(enabled_color);
                    } else {
                        button = button.fill(inactive_tab_color);
                    }
                    if ui.add(button).clicked() {
                        self.current_tab = Tab::Integrations;
                        self.config_changed = true;
                        info!("Switched to Integrations tab");
                    }
                    let mut button = egui::Button::new("Status").min_size(egui::vec2(100.0, 40.0));
                    if self.current_tab == Tab::Status {
                        button = button.fill(enabled_color);
                    } else {
                        button = button.fill(inactive_tab_color);
                    }
                    if ui.add(button).clicked() {
                        self.current_tab = Tab::Status;
                        self.config_changed = true;
                        info!("Switched to Status tab");
                    }
                    let mut button = egui::Button::new("Chatting").min_size(egui::vec2(100.0, 40.0));
                    if self.current_tab == Tab::Chatting {
                        button = button.fill(enabled_color);
                    } else {
                        button = button.fill(inactive_tab_color);
                    }
                    if ui.add(button).clicked() {
                        self.current_tab = Tab::Chatting;
                        self.config_changed = true;
                        info!("Switched to Chatting tab");
                    }
                    let mut button = egui::Button::new("Options").min_size(egui::vec2(100.0, 40.0));
                    if self.current_tab == Tab::Options {
                        button = button.fill(enabled_color);
                    } else {
                        button = button.fill(inactive_tab_color);
                    }
                    if ui.add(button).clicked() {
                        self.current_tab = Tab::Options;
                        self.config_changed = true;
                        info!("Switched to Options tab");
                    }
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("Send to VRChat");
                    let response = toggle_switch(ui, &mut self.send_to_vrchat, "send_to_vrchat_toggle");
                    if response.changed() {
                        self.config_changed = true;
                        debug!("Send to VRChat toggle changed to {}", self.send_to_vrchat);
                    }
                });
            });
        });
    
        // Right side panel with OSC preview
        egui::SidePanel::right("right_panel")
            .resizable(false)
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Discord").clicked() {
                        debug!("Discord button clicked");
                        if let Err(e) = open::that("https://discord.gg/kzYjRnppFn") {
                            error!("Failed to open Discord URL: {}", e);
                        } else {
                            info!("Opened Discord URL");
                        }
                    }
                    if ui.button("GitHub").clicked() {
                        debug!("GitHub button clicked");
                        if let Err(e) = open::that("https://github.com/Voiasis/RustyChatBox") {
                            error!("Failed to open GitHub URL: {}", e);
                        } else {
                            info!("Opened GitHub URL");
                        }
                    }
                    ui.heading("Preview");
                });
                let size = 300.0;
                ui.allocate_ui(egui::vec2(size, size), |ui| {
                    ui.group(|ui| {
                        ui.set_min_size(egui::vec2(size, size));
                        ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
                            ui.label(&self.osc_preview);
                        });
                    });
                });
                ui.separator();
                if self.current_tab == Tab::Chatting && self.chat_options.edit_messages && self.chat_options.live_editing {
                    ui.horizontal(|ui| {
                        let response = ui.checkbox(&mut self.live_edit_enabled, "Live edit chat messages");
                        if response.changed() {
                            self.config_changed = true;
                            debug!("Live edit chat messages checkbox changed to {}", self.live_edit_enabled);
                        }
                    });
                }
            });
    
        // Central panel with tab content
        match self.current_tab {
            Tab::Integrations => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    show_integrations_tab(ui, self);
                });
            }
            Tab::Status => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    show_status_tab(ui, self);
                });
            }
            Tab::Chatting => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    show_chatting_tab(ui, self);
                });
            }
            Tab::Options => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    show_options_tab(ui, self);
                });
            }
        }
    
        ctx.request_repaint_after(Duration::from_millis(100)); // 10 FPS
    }
}