use eframe::egui::{self, Response, Sense, Ui};
use serde::{Serialize, Deserialize};
use std::fs;
use crate::config::Config;
use crate::modules::{
    appo_mod::AppOptionsOptions,
    chat_mod::ChatOptions,
    comp_mod::{ComponentStatsModule, ComponentStatsOptions},
    extr_mod::ExtraOptions,
    medi_mod::{MediaLinkModule, MediaLinkOptions},
    netw_mod::{NetworkStats, NetworkStatsOptions},
    stat_mod::{StatusModule, StatusOptions},
    time_mod::{TimeModule, TimeOptions},
};
use crate::osc::OscClient;
use arboard::Clipboard;

// Toggle Switch function with improved appearance for egui 0.22.0
pub fn toggle_switch(ui: &mut Ui, on: &mut bool, id_source: &str) -> Response {
    let id = ui.make_persistent_id(id_source);
    let size = ui.spacing().interact_size.y * egui::vec2(1.5, 1.0);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());

    if response.clicked() {
        *on = !*on;
        ui.memory_mut(|m| m.data.insert_temp(id, *on));
    }

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let style_visuals = ui.style().visuals.clone();
        let (small_rect, big_rect) = {
            let small = rect.shrink(2.0);
            let big = small.expand(1.0);
            (small, big)
        };

        let t = ui.ctx().animate_bool_with_time(id, *on, 0.2);
        let color = {
            let start = style_visuals.widgets.inactive.bg_fill.to_array(); // disabled_color #7200ff
            let end = style_visuals.widgets.active.bg_fill.to_array(); // enabled_color #ff3f00
            let r = (start[0] as f32 + t * (end[0] as f32 - start[0] as f32)) as u8;
            let g = (start[1] as f32 + t * (end[1] as f32 - start[1] as f32)) as u8;
            let b = (start[2] as f32 + t * (end[2] as f32 - start[2] as f32)) as u8;
            let a = (start[3] as f32 + t * (end[3] as f32 - start[3] as f32)) as u8;
            egui::Color32::from_rgba_premultiplied(r, g, b, a)
        };

        ui.painter().add(egui::Shape::rect_filled(
            big_rect,
            4.0,
            visuals.bg_fill, // scrollbar_color #3f3f3f
        ));
        let base_pos = small_rect.min + egui::vec2(small_rect.height() / 2.0, small_rect.height() / 2.0);
        let offset = egui::vec2(small_rect.width() - small_rect.height(), 0.0);
        let interpolated_offset = egui::lerp(egui::vec2(0.0, 0.0)..=offset, t);
        let circle_pos = base_pos + interpolated_offset;
        ui.painter().add(egui::Shape::circle_filled(
            circle_pos,
            small_rect.height() / 2.5,
            color,
        ));
    }

    response
}

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
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StatusTab {
    pub new_message: String,
}

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
    osc_client: OscClient,
    components_module: ComponentStatsModule,
    media_module: MediaLinkModule,
    status_module: StatusModule,
    osc_preview: String,
    send_to_vrchat: bool,
    last_osc_send: std::time::Instant,
    config_changed: bool,
    pending_scroll_to: Option<egui::Id>,
    clipboard: Clipboard,
    live_edit_enabled: bool,
    previous_osc_preview: String,
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
            },
            media_link: config.media_link_options,
            network_stats: NetworkStatsOptions::new(config.network_stats_options.config),
            status_tab: config.status_tab,
            status_options: config.status_options,
            time_options: config.time_options,
            osc_client,
            components_module: ComponentStatsModule::new(),
            media_module: MediaLinkModule::new(),
            status_module,
            osc_preview: String::new(),
            send_to_vrchat: config.send_to_vrchat,
            last_osc_send: std::time::Instant::now(),
            config_changed: false,
            pending_scroll_to: None,
            clipboard,
            live_edit_enabled: config.live_edit_enabled,
            previous_osc_preview: String::new(),
        }
    }

    fn show_integrations_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Integrations");
    
        // Personal Status
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.heading("Personal Status");
                if ui.button("⚙").clicked() {
                    self.current_tab = Tab::Options;
                    self.pending_scroll_to = Some(egui::Id::new("status_options"));
                    self.status_options.enabled = true;
                    self.config_changed = true;
                    ui.ctx().request_repaint();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let response = toggle_switch(ui, &mut self.integrations_tab.personal_status_enabled, "personal_status_toggle");
                    if response.changed() {
                        self.config_changed = true;
                    }
                });
            });
            ui.label(egui::RichText::new("Manage your personal status messages.").color(egui::Color32::from_rgb(0x3f, 0x3f, 0x3f)));
        });
        ui.separator();
    
        // Component Stats
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.heading("Component Stats");
                if ui.button("⚙").clicked() {
                    self.current_tab = Tab::Options;
                    self.pending_scroll_to = Some(egui::Id::new("component_stats_options"));
                    self.component_stats.enabled = true;
                    self.config_changed = true;
                    ui.ctx().request_repaint();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let response = toggle_switch(ui, &mut self.integrations_tab.component_stats_enabled, "component_stats_toggle");
                    if response.changed() {
                        self.config_changed = true;
                    }
                });
            });
            ui.label(egui::RichText::new("View system performance stats (CPU, GPU, RAM, etc.).").color(egui::Color32::from_rgb(0x3f, 0x3f, 0x3f)));
            if self.integrations_tab.component_stats_enabled {
                ui.horizontal(|ui| {
                    if ui.toggle_value(&mut self.component_stats.show_cpu, "CPU").changed() {
                        self.config_changed = true;
                    }
                    if ui.toggle_value(&mut self.component_stats.show_gpu, "GPU").changed() {
                        self.config_changed = true;
                    }
                    if ui.toggle_value(&mut self.component_stats.show_vram, "VRAM").changed() {
                        self.config_changed = true;
                    }
                    if ui.toggle_value(&mut self.component_stats.show_ram, "RAM").changed() {
                        self.config_changed = true;
                    }
                });
            }
        });
        ui.separator();
    
        // Network Stats
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.heading("Network Stats");
                if ui.button("⚙").clicked() {
                    self.current_tab = Tab::Options;
                    self.pending_scroll_to = Some(egui::Id::new("network_stats_options"));
                    self.network_stats.enabled = true;
                    self.config_changed = true;
                    ui.ctx().request_repaint();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let response = toggle_switch(ui, &mut self.integrations_tab.network_stats_enabled, "network_stats_toggle");
                    if response.changed() {
                        self.config_changed = true;
                    }
                });
            });
            ui.label(egui::RichText::new("Monitor real-time network statistics (download/upload speeds).").color(egui::Color32::from_rgb(0x3f, 0x3f, 0x3f)));
            if self.integrations_tab.network_stats_enabled {
                let interfaces = NetworkStats::get_interfaces();
                if let Some(iface) = interfaces.first() {
                    let stats = NetworkStats::get_formatted_stats(&self.network_stats.config, &iface.name);
                    if !stats.is_empty() {
                        ui.label(stats);
                    }
                }
            }
        });
        ui.separator();
    
        // Current Time
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.heading("Current Time");
                if ui.button("⚙").clicked() {
                    self.current_tab = Tab::Options;
                    self.pending_scroll_to = Some(egui::Id::new("time_options"));
                    self.time_options.config.enabled = true;
                    self.config_changed = true;
                    ui.ctx().request_repaint();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let response = toggle_switch(ui, &mut self.integrations_tab.current_time_enabled, "current_time_toggle");
                    if response.changed() {
                        self.config_changed = true;
                    }
                });
            });
            ui.label(egui::RichText::new("Display the current local time and other time zones.").color(egui::Color32::from_rgb(0x3f, 0x3f, 0x3f)));
        });
        ui.separator();
    
        // MediaLink
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.heading("MediaLink");
                if ui.button("⚙").clicked() {
                    self.current_tab = Tab::Options;
                    self.pending_scroll_to = Some(egui::Id::new("medialink_options"));
                    self.media_link.enabled = true;
                    self.config_changed = true;
                    ui.ctx().request_repaint();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let response = toggle_switch(ui, &mut self.integrations_tab.medialink_enabled, "medialink_toggle");
                    if response.changed() {
                        self.config_changed = true;
                    }
                });
            });
            ui.label(egui::RichText::new("Show the current media track and artist.").color(egui::Color32::from_rgb(0x3f, 0x3f, 0x3f)));
            if self.integrations_tab.medialink_enabled {
                if let Some(track) = self.media_module.get_formatted_track(&self.media_link) {
                    ui.label(format!("Now playing: {}", track));
                } else {
                    ui.label("No media playing.");
                }
                ui.horizontal(|ui| {
                    ui.label("Progress");
                    let duration = self.media_module.get_duration().unwrap_or(1.0);
                    let mut position = self.media_module.get_position().unwrap_or(0.0);
                    let response = ui.add(
                        egui::Slider::new(&mut position, 0.0..=duration)
                            .show_value(false)
                            .text(""),
                    );
                    if response.changed() {
                        self.media_module.seek(position);
                    }
                    if ui.button("⏮").clicked() {
                        self.media_module.previous();
                    }
                    let is_playing = self.media_module.is_playing();
                    let play_pause_icon = if is_playing { "⏸" } else { "▶" };
                    if ui.button(play_pause_icon).clicked() {
                        self.media_module.play_pause();
                    }
                    if ui.button("⏭").clicked() {
                        self.media_module.next();
                    }
                });
            }
        });
    }

    fn show_status_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Status");
        ui.label("Manage personal status messages.");
        ui.horizontal(|ui| {
            ui.label("New status: ");
            if ui.text_edit_singleline(&mut self.status_tab.new_message).changed() {
                self.config_changed = true;
            }
            if ui.button("Add").clicked() && !self.status_tab.new_message.is_empty() {
                self.status_module.add_message(self.status_tab.new_message.clone());
                self.status_tab.new_message.clear();
                self.config_changed = true;
            }
        });
        if let Some(current) = self.status_module.get_current_message(&self.status_options) {
            ui.label(format!("Current status: {}", current));
            if ui.button("Send to OSC").clicked() {
                if self.last_osc_send.elapsed().as_secs_f32() >= self.app_options.app_options.osc_options.update_rate {
                    if let Err(e) = self.osc_client.send_chatbox_message(current.as_str(), false, self.extra_options.slim_mode) {
                        eprintln!("Failed to send status: {}", e);
                    }
                    self.last_osc_send = std::time::Instant::now();
                }
            }
        } else {
            ui.label("No status messages set.");
        }
        ui.label("Stored messages:");
        let mut to_remove = None;
        for (i, msg) in self.status_module.messages.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label(msg);
                if ui.button("Remove").clicked() {
                    to_remove = Some(i);
                    self.config_changed = true;
                }
            });
        }
        if let Some(index) = to_remove {
            self.status_module.remove_message(index);
            self.config_changed = true;
        }
    }

    fn show_chatting_tab(&mut self, ui: &mut egui::Ui) {
        // Input section pinned to the bottom
        egui::TopBottomPanel::bottom("chat_input").show(ui.ctx(), |ui| {
            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    let placeholder = if self.chat_tab.is_focused || !self.chat_tab.message.is_empty() {
                        ""
                    } else {
                        "Send a chat message"
                    };
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.chat_tab.message)
                            .desired_width(200.0)
                            .hint_text(placeholder),
                    );
                    if response.changed() {
                        self.config_changed = true;
                    }
                    if response.has_focus() != self.chat_tab.is_focused {
                        self.chat_tab.is_focused = response.has_focus();
                        self.config_changed = true;
                    }
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && !self.chat_tab.message.is_empty() && self.chat_tab.message.len() <= 140 {
                        let message = self.chat_tab.message.clone();
                        if self.chat_options.can_send() && self.last_osc_send.elapsed().as_secs_f32() >= self.app_options.app_options.osc_options.update_rate {
                            let formatted_message = if self.chat_options.add_speech_bubble {
                                format!("🗨 {}", message)
                            } else {
                                message.clone()
                            };
                            self.osc_preview = formatted_message.clone();
                            if self.send_to_vrchat {
                                if let Err(e) = self.osc_client.send_chatbox_message(&formatted_message, self.chat_options.play_fx_sound, self.extra_options.slim_mode) {
                                    eprintln!("Failed to send OSC message: {}", e);
                                }
                                self.last_osc_send = std::time::Instant::now();
                            }
                            self.chat_options.add_message(message);
                            self.chat_tab.message.clear();
                            self.chat_tab.is_focused = false;
                            self.config_changed = true;
                        } else {
                            self.chat_options.set_queued_message(message);
                            self.chat_tab.message.clear();
                            self.chat_tab.is_focused = false;
                            self.config_changed = true;
                        }
                    }
                    ui.label(format!("{}/140", self.chat_tab.message.len()));
                    if ui.button("Paste").clicked() {
                        if let Ok(text) = self.clipboard.get_text() {
                            self.chat_tab.message = text.chars().take(140).collect();
                            self.chat_tab.is_focused = true;
                            self.config_changed = true;
                        }
                    }
                    if ui.button("Send").clicked() && !self.chat_tab.message.is_empty() && self.chat_tab.message.len() <= 140 {
                        let message = self.chat_tab.message.clone();
                        if self.chat_options.can_send() && self.last_osc_send.elapsed().as_secs_f32() >= self.app_options.app_options.osc_options.update_rate {
                            let formatted_message = if self.chat_options.add_speech_bubble {
                                format!("🗨 {}", message)
                            } else {
                                message.clone()
                            };
                            self.osc_preview = formatted_message.clone();
                            if self.send_to_vrchat {
                                if let Err(e) = self.osc_client.send_chatbox_message(&formatted_message, self.chat_options.play_fx_sound, self.extra_options.slim_mode) {
                                    eprintln!("Failed to send OSC message: {}", e);
                                }
                                self.last_osc_send = std::time::Instant::now();
                            }
                            self.chat_options.add_message(message);
                            self.chat_tab.message.clear();
                            self.chat_tab.is_focused = false;
                            self.config_changed = true;
                        } else {
                            self.chat_options.set_queued_message(message);
                            self.chat_tab.message.clear();
                            self.chat_tab.is_focused = false;
                            self.config_changed = true;
                        }
                    }
                });
            });
        });

        // Message list and control buttons in a scrollable central panel
        egui::CentralPanel::default().show(ui.ctx(), |ui| {
            ui.heading("Chatting");
            ui.label("Send and manage chat messages.");

            egui::ScrollArea::vertical().show(ui, |ui| {
                // Message list
                ui.group(|ui| {
                    ui.set_width(ui.available_width());
                    let now_ms = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64;
                    let mut remaining_times = Vec::new();
                    for message in self.chat_options.messages.iter() {
                        if ((now_ms - message.sent_at_ms) / 1000) < self.chat_options.chat_timeout as u64 {
                            remaining_times.push(self.chat_options.get_remaining_time(message));
                        } else {
                            remaining_times.push(0);
                        }
                    }
                    let mut index = 0;
                    for (_index, message) in self.chat_options.messages.iter_mut().enumerate().rev() {
                        if ((now_ms - message.sent_at_ms) / 1000) < self.chat_options.chat_timeout as u64 {
                            let remaining_time = remaining_times[index];
                            ui.horizontal(|ui| {
                                if message.editing {
                                    let response = ui.add(
                                        egui::TextEdit::singleline(&mut message.edit_text)
                                            .desired_width(200.0)
                                            .hint_text("Edit message"),
                                    );
                                    if response.changed() {
                                        self.config_changed = true;
                                    }
                                    if !self.chat_options.live_editing && response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                        message.text = message.edit_text.clone();
                                        message.sent_at_ms = now_ms;
                                        let formatted_message = if self.chat_options.add_speech_bubble {
                                            format!("🗨 {}", message.text)
                                        } else {
                                            message.text.clone()
                                        };
                                        self.osc_preview = formatted_message.clone();
                                        if self.send_to_vrchat && self.last_osc_send.elapsed().as_secs_f32() >= self.app_options.app_options.osc_options.update_rate {
                                            if let Err(e) = self.osc_client.send_chatbox_message(&formatted_message, self.chat_options.play_fx_sound, self.extra_options.slim_mode) {
                                                eprintln!("Failed to send OSC message: {}", e);
                                            }
                                            self.last_osc_send = std::time::Instant::now();
                                        }
                                        message.editing = false;
                                        self.config_changed = true;
                                    }
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if self.chat_options.live_editing && ui.button("X").clicked() {
                                            message.editing = false;
                                            message.edit_text = message.text.clone();
                                            self.osc_preview = self.previous_osc_preview.clone();
                                            if self.send_to_vrchat && !self.osc_preview.is_empty() && self.last_osc_send.elapsed().as_secs_f32() >= self.app_options.app_options.osc_options.update_rate {
                                                if let Err(e) = self.osc_client.send_chatbox_message(&self.osc_preview, false, self.extra_options.slim_mode) {
                                                    eprintln!("Failed to send OSC message: {}", e);
                                                }
                                                self.last_osc_send = std::time::Instant::now();
                                            }
                                            self.config_changed = true;
                                        }
                                        let edit_label = if self.chat_options.live_editing {
                                            format!("Live Edit ({})", remaining_time)
                                        } else {
                                            format!("Edit ({})", remaining_time)
                                        };
                                        if ui.button(&edit_label).clicked() {
                                            message.editing = !message.editing;
                                            if !message.editing {
                                                message.edit_text = message.text.clone();
                                                self.osc_preview = self.previous_osc_preview.clone();
                                                if self.send_to_vrchat && !self.osc_preview.is_empty() && self.last_osc_send.elapsed().as_secs_f32() >= self.app_options.app_options.osc_options.update_rate {
                                                    if let Err(e) = self.osc_client.send_chatbox_message(&self.osc_preview, false, self.extra_options.slim_mode) {
                                                        eprintln!("Failed to send OSC message: {}", e);
                                                    }
                                                    self.last_osc_send = std::time::Instant::now();
                                                }
                                            }
                                            self.config_changed = true;
                                        }
                                        if ui.button("Copy").clicked() {
                                            if let Err(e) = self.clipboard.set_text(&message.text) {
                                                eprintln!("Failed to copy to clipboard: {}", e);
                                            }
                                        }
                                        if ui.button("Resend").clicked() {
                                            if self.last_osc_send.elapsed().as_secs_f32() >= self.app_options.app_options.osc_options.update_rate {
                                                let formatted_message = if self.chat_options.add_speech_bubble {
                                                    format!("🗨 {}", message.text)
                                                } else {
                                                    message.text.clone()
                                                };
                                                message.sent_at_ms = now_ms;
                                                self.osc_preview = formatted_message.clone();
                                                if self.send_to_vrchat {
                                                    if let Err(e) = self.osc_client.send_chatbox_message(&formatted_message, self.chat_options.play_fx_resend && self.chat_options.play_fx_sound, self.extra_options.slim_mode) {
                                                        eprintln!("Failed to send OSC message: {}", e);
                                                    }
                                                    self.last_osc_send = std::time::Instant::now();
                                                }
                                                self.config_changed = true;
                                            }
                                        }
                                    });
                                } else {
                                    ui.label(&message.text);
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if self.chat_options.edit_messages {
                                            let edit_label = if self.chat_options.live_editing && self.live_edit_enabled {
                                                format!("Live Edit ({})", remaining_time)
                                            } else {
                                                format!("Edit ({})", remaining_time)
                                            };
                                            if ui.button(&edit_label).clicked() {
                                                message.editing = true;
                                                self.config_changed = true;
                                            }
                                        }
                                        if ui.button("Copy").clicked() {
                                            if let Err(e) = self.clipboard.set_text(&message.text) {
                                                eprintln!("Failed to copy to clipboard: {}", e);
                                            }
                                        }
                                        if ui.button("Resend").clicked() {
                                            if self.last_osc_send.elapsed().as_secs_f32() >= self.app_options.app_options.osc_options.update_rate {
                                                let formatted_message = if self.chat_options.add_speech_bubble {
                                                    format!("🗨 {}", message.text)
                                                } else {
                                                    message.text.clone()
                                                };
                                                message.sent_at_ms = now_ms;
                                                self.osc_preview = formatted_message.clone();
                                                if self.send_to_vrchat {
                                                    if let Err(e) = self.osc_client.send_chatbox_message(&formatted_message, self.chat_options.play_fx_resend && self.chat_options.play_fx_sound, self.extra_options.slim_mode) {
                                                        eprintln!("Failed to send OSC message: {}", e);
                                                    }
                                                    self.last_osc_send = std::time::Instant::now();
                                                }
                                                self.config_changed = true;
                                            }
                                        }
                                    });
                                }
                            });
                            ui.separator();
                            index += 1;
                        }
                    }
                });
                // Control buttons (Stop, Clear history) only when messages exist
                if !self.chat_options.messages.is_empty() {
                    ui.group(|ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("Clear history").clicked() {
                                    self.chat_options.clear_messages();
                                    self.config_changed = true;
                                }
                                if ui.button("Stop").clicked() {
                                    self.osc_preview = self.previous_osc_preview.clone();
                                    if self.send_to_vrchat && !self.osc_preview.is_empty() && self.last_osc_send.elapsed().as_secs_f32() >= self.app_options.app_options.osc_options.update_rate {
                                        if let Err(e) = self.osc_client.send_chatbox_message(&self.osc_preview, false, self.extra_options.slim_mode) {
                                            eprintln!("Failed to send OSC message: {}", e);
                                        }
                                        self.last_osc_send = std::time::Instant::now();
                                    }
                                }
                            });
                        });
                    });
                }
            });
        });
    }

    fn show_options_tab(&mut self, ui: &mut egui::Ui) {
        let mut scroll_to_rect = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Options");
            // Status Options
            let _status_id = egui::Id::new("status_options_group");
            let status_response = ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), 0.0),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.group(|ui| {
                        ui.push_id(egui::Id::new("status_options"), |ui| {
                            ui.horizontal(|ui| {
                                let response = ui.checkbox(&mut self.status_options.enabled, "");
                                if response.changed() {
                                    self.config_changed = true;
                                }
                                ui.heading("Status Options");
                            });
                            if self.status_options.enabled {
                                let response = self.status_options.show_status_options(ui);
                                if response.changed() {
                                    self.config_changed = true;
                                }
                            }
                        });
                    });
                },
            );
            if self.pending_scroll_to == Some(egui::Id::new("status_options")) {
                scroll_to_rect = Some(status_response.response.rect);
            }
            ui.separator();
            // Time Options
            let _time_id = egui::Id::new("time_options_group");
            let time_response = ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), 0.0),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.group(|ui| {
                        ui.push_id(egui::Id::new("time_options"), |ui| {
                            ui.horizontal(|ui| {
                                let response = ui.checkbox(&mut self.time_options.config.enabled, "");
                                if response.changed() {
                                    self.config_changed = true;
                                }
                                ui.heading("Time Options");
                            });
                            if self.time_options.config.enabled {
                                let response = self.time_options.show_time_options(ui);
                                if response.changed() {
                                    self.config_changed = true;
                                }
                            }
                        });
                    });
                },
            );
            if self.pending_scroll_to == Some(egui::Id::new("time_options")) {
                scroll_to_rect = Some(time_response.response.rect);
            }
            ui.separator();
            // Component Stats Options
            let _component_id = egui::Id::new("component_stats_options_group");
            let component_response = ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), 0.0),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.group(|ui| {
                        ui.push_id(egui::Id::new("component_stats_options"), |ui| {
                            ui.horizontal(|ui| {
                                let response = ui.checkbox(&mut self.component_stats.enabled, "");
                                if response.changed() {
                                    self.config_changed = true;
                                }
                                ui.heading("Component Stats Options");
                            });
                            if self.component_stats.enabled {
                                let response = self.component_stats.show_component_stats_options(ui);
                                if response.changed() {
                                    self.config_changed = true;
                                }
                            }
                        });
                    });
                },
            );
            if self.pending_scroll_to == Some(egui::Id::new("component_stats_options")) {
                scroll_to_rect = Some(component_response.response.rect);
            }
            ui.separator();
            // Network Stats Options
            let _network_id = egui::Id::new("network_stats_options_group");
            let network_response = ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), 0.0),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.group(|ui| {
                        ui.push_id(egui::Id::new("network_stats_options"), |ui| {
                            ui.horizontal(|ui| {
                                let response = ui.checkbox(&mut self.network_stats.enabled, "");
                                if response.changed() {
                                    self.config_changed = true;
                                }
                                ui.heading("Network Stats Options");
                            });
                            if self.network_stats.enabled {
                                let response = self.network_stats.show_network_stats_options(ui);
                                if response.changed() {
                                    self.config_changed = true;
                                }
                            }
                        });
                    });
                },
            );
            if self.pending_scroll_to == Some(egui::Id::new("network_stats_options")) {
                scroll_to_rect = Some(network_response.response.rect);
            }
            ui.separator();
            // Chatting Options
            let _chatting_id = egui::Id::new("chatting_options_group");
            let chatting_response = ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), 0.0),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.group(|ui| {
                        ui.push_id(egui::Id::new("chatting_options"), |ui| {
                            ui.horizontal(|ui| {
                                let response = ui.checkbox(&mut self.chat_options.enabled, "");
                                if response.changed() {
                                    self.config_changed = true;
                                }
                                ui.heading("Chatting Options");
                            });
                            if self.chat_options.enabled {
                                let response = self.chat_options.show_chatting_options(ui);
                                if response.changed() {
                                    self.config_changed = true;
                                }
                            }
                        });
                    });
                },
            );
            if self.pending_scroll_to == Some(egui::Id::new("chatting_options")) {
                scroll_to_rect = Some(chatting_response.response.rect);
            }
            ui.separator();
            // MediaLink Options
            let _medialink_id = egui::Id::new("medialink_options_group");
            let medialink_response = ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), 0.0),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.group(|ui| {
                        ui.push_id(egui::Id::new("medialink_options"), |ui| {
                            ui.horizontal(|ui| {
                                let response = ui.checkbox(&mut self.media_link.enabled, "");
                                if response.changed() {
                                    self.config_changed = true;
                                }
                                ui.heading("MediaLink Options");
                            });
                            if self.media_link.enabled {
                                let response = self.media_link.show_medialink_options(ui);
                                if response.changed() {
                                    self.config_changed = true;
                                }
                            }
                        });
                    });
                },
            );
            if self.pending_scroll_to == Some(egui::Id::new("medialink_options")) {
                scroll_to_rect = Some(medialink_response.response.rect);
            }
            ui.separator();
            // App Options
            let _app_id = egui::Id::new("app_options_group");
            let app_response = ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), 0.0),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.group(|ui| {
                        ui.push_id(egui::Id::new("app_options"), |ui| {
                            ui.horizontal(|ui| {
                                let response = ui.checkbox(&mut self.app_options.enabled, "");
                                if response.changed() {
                                    self.config_changed = true;
                                }
                                ui.heading("App Options");
                            });
                            if self.app_options.enabled {
                                let response = self.app_options.show_app_options(ui);
                                if response.changed() {
                                    self.config_changed = true;
                                }
                            }
                        });
                    });
                },
            );
            if self.pending_scroll_to == Some(egui::Id::new("app_options")) {
                scroll_to_rect = Some(app_response.response.rect);
            }
            ui.separator();
            // Extra Options
            let _extra_id = egui::Id::new("extra_options_group");
            let extra_response = ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), 0.0),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.group(|ui| {
                        ui.push_id(egui::Id::new("extra_options"), |ui| {
                            ui.horizontal(|ui| {
                                let response = ui.checkbox(&mut self.extra_options.enabled, "");
                                if response.changed() {
                                    self.config_changed = true;
                                }
                                ui.heading("Extra Options");
                            });
                            if self.extra_options.enabled {
                                let response = self.extra_options.show_extra_options(ui);
                                if response.changed() {
                                    self.config_changed = true;
                                }
                            }
                        });
                    });
                },
            );
            if self.pending_scroll_to == Some(egui::Id::new("extra_options")) {
                scroll_to_rect = Some(extra_response.response.rect);
            }
            // Perform scroll after rendering all sections
            if let Some(rect) = scroll_to_rect {
                ui.scroll_to_rect(rect, Some(egui::Align::TOP));
                self.pending_scroll_to = None;
            }
        });
    }

    fn update_osc_preview(&mut self) {
        self.status_module.update_cycle(&self.status_options);
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
    
        const MAX_LINE_WIDTH: usize = 27; // VRChat chatbox line width
    
        let should_update = self.send_to_vrchat
            && self.chat_options.can_send()
            && self.last_osc_send.elapsed().as_secs_f32() >= self.app_options.app_options.osc_options.update_rate;
    
        let should_update_live = self.send_to_vrchat
            && self.chat_options.live_editing
            && self.chat_options.override_display_time
            && self.last_osc_send.elapsed().as_secs_f32() >= self.app_options.app_options.osc_options.update_rate.max(self.chat_options.display_time_seconds);
    
        // Find the currently editing message, if any
        let active_chat_message = self.chat_options.messages.iter_mut().find(|m| m.editing);
    
        if should_update || should_update_live {
            let mut parts = Vec::new();
    
            if self.integrations_tab.personal_status_enabled {
                if let Some(status) = self.status_module.get_current_message(&self.status_options) {
                    parts.push(status);
                }
            }
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
            if self.integrations_tab.network_stats_enabled {
                let interfaces = NetworkStats::get_interfaces();
                if let Some(iface) = interfaces.first() {
                    let stats = NetworkStats::get_formatted_stats(&self.network_stats.config, &iface.name);
                    if !stats.is_empty() {
                        parts.push(stats);
                    }
                }
            }
            if self.integrations_tab.current_time_enabled {
                let time = TimeModule::get_local_time(&self.time_options);
                parts.push(time);
            }
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
                let part_text = part.trim();
    
                if part_text.len() > MAX_LINE_WIDTH {
                    let mut chars = part_text.chars().collect::<Vec<_>>();
                    while !chars.is_empty() {
                        let take_count = MAX_LINE_WIDTH.min(chars.len());
                        let chunk: String = chars.drain(..take_count).collect();
                        lines.push(chunk);
                    }
                } else {
                    lines.push(part_text.to_string());
                }
    
                if !is_last_part && !self.app_options.app_options.osc_options.separate_lines {
                    if lines.last_mut().map_or(true, |last| last.len() + separator.len() <= MAX_LINE_WIDTH) {
                        lines.last_mut().map(|last| *last += separator);
                    } else {
                        lines.push(separator.to_string());
                    }
                }
            }
            self.previous_osc_preview = lines.join("\n");
    
            // Handle active chat message (e.g., being edited)
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
                if formatted_message.len() > MAX_LINE_WIDTH {
                    let mut chars = formatted_message.chars().collect::<Vec<_>>();
                    while !chars.is_empty() {
                        let take_count = MAX_LINE_WIDTH.min(chars.len());
                        let chunk: String = chars.drain(..take_count).collect();
                        message_lines.push(chunk);
                    }
                } else {
                    message_lines.push(formatted_message);
                }
                if !self.previous_osc_preview.is_empty() {
                    message_lines.push(self.previous_osc_preview.clone());
                }
                self.osc_preview = message_lines.join("\n");
            } else {
                self.osc_preview = self.previous_osc_preview.clone();
            }
        }
    
        // Send live updates for editing messages
        if should_update_live {
            if let Some(ref message) = active_chat_message {
                if message.editing && self.send_to_vrchat && !self.osc_preview.is_empty() {
                    // Only send if the message is still valid (within timeout)
                    if (now_ms - message.sent_at_ms) / 1000 < self.chat_options.chat_timeout as u64 {
                        if let Err(e) = self.osc_client.send_chatbox_message(
                            &self.osc_preview,
                            self.chat_options.play_fx_sound,
                            self.extra_options.slim_mode,
                        ) {
                            eprintln!("Failed to send OSC message: {}", e);
                        }
                        self.last_osc_send = std::time::Instant::now();
                    }
                }
            }
        }
    
        // Handle queued messages or regular updates
        if should_update && !self.osc_preview.is_empty() {
            if let Some(message) = self.chat_options.take_queued_message() {
                let formatted_message = if self.chat_options.add_speech_bubble {
                    format!("🗨 {}", message)
                } else {
                    message
                };
                let mut lines = Vec::new();
                if formatted_message.len() > MAX_LINE_WIDTH {
                    let mut chars = formatted_message.chars().collect::<Vec<_>>();
                    while !chars.is_empty() {
                        let take_count = MAX_LINE_WIDTH.min(chars.len());
                        let chunk: String = chars.drain(..take_count).collect();
                        lines.push(chunk);
                    }
                } else {
                    lines.push(formatted_message.clone());
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
                        eprintln!("Failed to send OSC message: {}", e);
                    }
                    self.last_osc_send = std::time::Instant::now();
                }
                self.chat_options.add_message(formatted_message);
            } else if self.send_to_vrchat {
                if let Err(e) = self.osc_client.send_chatbox_message(
                    &self.osc_preview,
                    self.chat_options.play_fx_sound,
                    self.extra_options.slim_mode,
                ) {
                    eprintln!("Failed to send OSC message: {}", e);
                }
                self.last_osc_send = std::time::Instant::now();
            }
        }
    }

    pub fn save_config_if_needed(&mut self, config_path: &std::path::Path) {
        if self.config_changed {
            let config = Config {
                app_options: self.app_options.app_options.clone(),
                personal_status_enabled: self.integrations_tab.personal_status_enabled,
                component_stats_enabled: self.integrations_tab.component_stats_enabled,
                network_stats_enabled: self.integrations_tab.network_stats_enabled,
                current_time_enabled: self.integrations_tab.current_time_enabled,
                medialink_enabled: self.integrations_tab.medialink_enabled,
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
                    eprintln!("Failed to save config: {}", e);
                }
            }
            self.config_changed = false;
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_osc_preview();

// Set custom visual style
let mut visuals = egui::Visuals::default();
visuals.dark_mode = true; // Ensure dark mode for consistency

// Helper function to convert hex to Color32
let hex_to_color = |hex: &str| {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    egui::Color32::from_rgb(r, g, b)
};

// Define colors
let border_color = hex_to_color("ff3f00"); // Borders and padding
let background_color = hex_to_color("141414"); // Background
let font_color = hex_to_color("ffffff"); // Font (except VRC OSC and descriptions)
let disabled_color = hex_to_color("7200ff"); // Disabled toggles
let enabled_color = hex_to_color("ff3f00"); // Selected tabs, buttons on
let button_color = hex_to_color("3f3f3f"); // Normal buttons, CPU/GPU/VRAM/RAM off
let slider_color = hex_to_color("ff3f00"); // Sliders
let dropdown_color = hex_to_color("3f3f3f"); // Dropdown backgrounds
let dropdown_outline_color = hex_to_color("b7410e"); // Dropdown outlines
let dropdown_hover_color = hex_to_color("b7410e"); // Dropdown hover
let input_field_color = hex_to_color("000000"); // Input fields
let scrollbar_color = hex_to_color("3f3f3f"); // Scrollbars
let inactive_tab_color = hex_to_color("3f3f3f"); // Unselected tabs
let fallback_color = hex_to_color("7200ff"); // Fallback

// Background
visuals.panel_fill = background_color;
visuals.window_fill = dropdown_color; // Dropdown background
visuals.widgets.noninteractive.bg_fill = button_color; // Normal buttons, CPU/GPU/VRAM/RAM off
visuals.widgets.inactive.bg_fill = button_color; // CPU/GPU/VRAM/RAM off
visuals.widgets.active.bg_fill = enabled_color; // CPU/GPU/VRAM/RAM on
visuals.widgets.hovered.bg_fill = enabled_color;

// Borders and padding
visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, border_color); // Button outlines (non-hovered)
visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, border_color); // Button outlines
visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, border_color); // Borders/padding
visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, border_color); // Hovered outlines
visuals.window_stroke = egui::Stroke::new(1.0, dropdown_outline_color); // Dropdown outlines
visuals.widgets.noninteractive.rounding = egui::Rounding::same(4.0);

// Font color
visuals.override_text_color = Some(font_color); // General font
visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, font_color);
visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, font_color);
visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, font_color);
visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, font_color);

// Disabled toggles (including toggle switches)
visuals.widgets.inactive.bg_fill = disabled_color; // Toggle off
visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, border_color);
visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, font_color);

// Enabled toggles (including toggle switches)
visuals.widgets.active.bg_fill = enabled_color; // Toggle on
visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, border_color);
visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, font_color);
visuals.widgets.hovered.bg_fill = enabled_color;

// Normal buttons
visuals.widgets.noninteractive.bg_fill = button_color;
visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, border_color);
visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, font_color);

// Sliders and progress bars
visuals.widgets.active.bg_fill = slider_color;
visuals.widgets.hovered.bg_fill = slider_color;
visuals.widgets.inactive.bg_fill = slider_color.gamma_multiply(0.5);
visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, border_color);

// Dropdowns
visuals.widgets.open.bg_fill = dropdown_color; // Dropdown background
visuals.widgets.open.bg_stroke = egui::Stroke::new(1.0, dropdown_outline_color);
visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, font_color);
visuals.selection.bg_fill = dropdown_hover_color; // Dropdown hover

// Input fields
visuals.extreme_bg_color = input_field_color;
visuals.selection.stroke = egui::Stroke::new(1.0, font_color);

// Scrollbar
visuals.widgets.noninteractive.bg_fill = scrollbar_color;
visuals.widgets.inactive.bg_fill = scrollbar_color.gamma_multiply(0.5);

// Fallback
visuals.widgets.noninteractive.bg_fill = button_color;
visuals.hyperlink_color = fallback_color;
visuals.faint_bg_color = fallback_color.gamma_multiply(0.5);
visuals.code_bg_color = fallback_color.gamma_multiply(0.2);

// Apply visuals to context
ctx.set_visuals(visuals);

        // Top panel with title, tabs, and VRChat toggle
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("VRC OSC").color(egui::Color32::from_rgb(0x3f, 0x3f, 0x3f)));
                    ui.heading("RustyChatBox");
                });
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.add_space(20.0);
                    // Integrations tab
                    let mut button = egui::Button::new("Integrations").min_size(egui::vec2(100.0, 40.0));
                    if self.current_tab == Tab::Integrations {
                        button = button.fill(enabled_color);
                    } else {
                        button = button.fill(inactive_tab_color);
                    }
                    if ui.add(button).clicked() {
                        self.current_tab = Tab::Integrations;
                        self.config_changed = true;
                    }
                    // Status tab
                    let mut button = egui::Button::new("Status").min_size(egui::vec2(100.0, 40.0));
                    if self.current_tab == Tab::Status {
                        button = button.fill(enabled_color);
                    } else {
                        button = button.fill(inactive_tab_color);
                    }
                    if ui.add(button).clicked() {
                        self.current_tab = Tab::Status;
                        self.config_changed = true;
                    }
                    // Chatting tab
                    let mut button = egui::Button::new("Chatting").min_size(egui::vec2(100.0, 40.0));
                    if self.current_tab == Tab::Chatting {
                        button = button.fill(enabled_color);
                    } else {
                        button = button.fill(inactive_tab_color);
                    }
                    if ui.add(button).clicked() {
                        self.current_tab = Tab::Chatting;
                        self.config_changed = true;
                    }
                    // Options tab
                    let mut button = egui::Button::new("Options").min_size(egui::vec2(100.0, 40.0));
                    if self.current_tab == Tab::Options {
                        button = button.fill(enabled_color);
                    } else {
                        button = button.fill(inactive_tab_color);
                    }
                    if ui.add(button).clicked() {
                        self.current_tab = Tab::Options;
                        self.config_changed = true;
                    }
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("Send to VRChat");
                    let response = toggle_switch(ui, &mut self.send_to_vrchat, "send_to_vrchat_toggle");
                    if response.changed() {
                        self.config_changed = true;
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
                        if let Err(e) = open::that("https://discord.gg/kzYjRnppFn") {
                            eprintln!("Failed to open Discord URL: {}", e);
                        }
                    }
                    if ui.button("GitHub").clicked() {
                        if let Err(e) = open::that("https://github.com/Voiasis/RustyChatBox") {
                            eprintln!("Failed to open GitHub URL: {}", e);
                        }
                    }
                    ui.heading("Preview");
                });
                // Create a fixed-size group for the preview
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
                        }
                    });
                }
            });

        // Central panel with tab content
        match self.current_tab {
            Tab::Integrations => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    self.show_integrations_tab(ui);
                });
            }
            Tab::Status => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    self.show_status_tab(ui);
                });
            }
            Tab::Chatting => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    self.show_chatting_tab(ui);
                });
            }
            Tab::Options => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    self.show_options_tab(ui);
                });
            }
        }
        ctx.request_repaint();
    }
}