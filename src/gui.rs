use eframe::egui;
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
    scroll_to: Option<egui::Id>,
    pending_scroll_to: Option<egui::Id>,
    clipboard: Clipboard,
    live_edit_enabled: bool,
    previous_osc_preview: String,
}
impl App {
    pub fn new(osc_client: OscClient, config: Config, clipboard: Clipboard) -> Self {
        Self {
            current_tab: Tab::Integrations,
            app_options: AppOptionsOptions {
                app_options: config.app_options,
                enabled: true,
            },
            chat_tab: ChatTab {
                message: String::new(),
                is_focused: false,
            },
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
            status_tab: StatusTab {
                new_message: String::new(),
            },
            status_options: config.status_options,
            time_options: config.time_options,
            osc_client,
            components_module: ComponentStatsModule::new(),
            media_module: MediaLinkModule::new(),
            status_module: StatusModule::new(),
            osc_preview: String::new(),
            send_to_vrchat: false,
            last_osc_send: std::time::Instant::now(),
            config_changed: false,
            scroll_to: None,
            pending_scroll_to: None,
            clipboard,
            live_edit_enabled: false,
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
                    println!("DEBUG: Personal Status cogwheel clicked");
                    self.current_tab = Tab::Options;
                    self.pending_scroll_to = Some(egui::Id::new("status_options"));
                    self.status_options.enabled = true;
                    self.config_changed = true;
                    ui.ctx().request_repaint();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.checkbox(&mut self.integrations_tab.personal_status_enabled, "").changed() {
                        self.config_changed = true;
                    }
                });
            });
            ui.label("Manage your personal status messages.");
        });
        ui.separator();
        // Component Stats
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.heading("Component Stats");
                if ui.button("⚙").clicked() {
                    println!("DEBUG: Component Stats cogwheel clicked");
                    self.current_tab = Tab::Options;
                    self.pending_scroll_to = Some(egui::Id::new("component_stats_options"));
                    self.component_stats.enabled = true;
                    self.config_changed = true;
                    ui.ctx().request_repaint();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.checkbox(&mut self.integrations_tab.component_stats_enabled, "").changed() {
                        self.config_changed = true;
                    }
                });
            });
            ui.label("View system performance stats (CPU, GPU, RAM, etc.).");
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
                    println!("DEBUG: Network Stats cogwheel clicked");
                    self.current_tab = Tab::Options;
                    self.pending_scroll_to = Some(egui::Id::new("network_stats_options"));
                    self.network_stats.enabled = true;
                    self.config_changed = true;
                    ui.ctx().request_repaint();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.checkbox(&mut self.integrations_tab.network_stats_enabled, "").changed() {
                        self.config_changed = true;
                    }
                });
            });
            ui.label("Monitor real-time network statistics (download/upload speeds).");
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
                    println!("DEBUG: Current Time cogwheel clicked");
                    self.current_tab = Tab::Options;
                    self.pending_scroll_to = Some(egui::Id::new("time_options"));
                    self.time_options.config.enabled = true;
                    self.config_changed = true;
                    ui.ctx().request_repaint();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.checkbox(&mut self.integrations_tab.current_time_enabled, "").changed() {
                        self.config_changed = true;
                    }
                });
            });
            ui.label("Display the current local time and other time zones.");
        });
        ui.separator();
        // MediaLink
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.heading("MediaLink");
                if ui.button("⚙").clicked() {
                    println!("DEBUG: MediaLink cogwheel clicked");
                    self.current_tab = Tab::Options;
                    self.pending_scroll_to = Some(egui::Id::new("medialink_options"));
                    self.media_link.enabled = true;
                    self.config_changed = true;
                    ui.ctx().request_repaint();
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.checkbox(&mut self.integrations_tab.medialink_enabled, "").changed() {
                        self.config_changed = true;
                    }
                });
            });
            ui.label("Show the current media track and artist.");
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
            ui.text_edit_singleline(&mut self.status_tab.new_message);
            if ui.button("Add").clicked() && !self.status_tab.new_message.is_empty() {
                self.status_module.add_message(self.status_tab.new_message.clone());
                self.status_tab.new_message.clear();
                self.config_changed = true;
            }
        });
        if let Some(current) = self.status_module.get_current_message(&self.status_options) {
            ui.label(format!("Current status: {}", current));
            if ui.button("Send to OSC").clicked() {
                if let Err(e) = self.osc_client.send_chatbox_message(current.as_str(), false) {
                    eprintln!("Failed to send status: {}", e);
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
                        if self.chat_options.can_send() {
                            let formatted_message = if self.chat_options.add_speech_bubble {
                                format!("🗨 {}", message)
                            } else {
                                message.clone()
                            };
                            self.osc_preview = formatted_message.clone();
                            if self.send_to_vrchat {
                                if let Err(e) = self.osc_client.send_chatbox_message(&formatted_message, self.chat_options.play_fx_sound) {
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
                        if self.chat_options.can_send() {
                            let formatted_message = if self.chat_options.add_speech_bubble {
                                format!("🗨 {}", message)
                            } else {
                                message.clone()
                            };
                            self.osc_preview = formatted_message.clone();
                            if self.send_to_vrchat {
                                if let Err(e) = self.osc_client.send_chatbox_message(&formatted_message, self.chat_options.play_fx_sound) {
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
                                        if self.send_to_vrchat {
                                            if let Err(e) = self.osc_client.send_chatbox_message(&formatted_message, self.chat_options.play_fx_sound) {
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
                                            if self.send_to_vrchat && !self.osc_preview.is_empty() {
                                                let mut message_text = self.osc_preview.clone();
                                                if self.extra_options.slim_mode {
                                                    message_text.push_str("\u{0003}\u{001f}");
                                                }
                                                if let Err(e) = self.osc_client.send_chatbox_message(&message_text, false) {
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
                                                if self.send_to_vrchat && !self.osc_preview.is_empty() {
                                                    let mut message_text = self.osc_preview.clone();
                                                    if self.extra_options.slim_mode {
                                                        message_text.push_str("\u{0003}\u{001f}");
                                                    }
                                                    if let Err(e) = self.osc_client.send_chatbox_message(&message_text, false) {
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
                                            let formatted_message = if self.chat_options.add_speech_bubble {
                                                format!("🗨 {}", message.text)
                                            } else {
                                                message.text.clone()
                                            };
                                            message.sent_at_ms = now_ms;
                                            self.osc_preview = formatted_message.clone();
                                            if self.send_to_vrchat {
                                                if let Err(e) = self.osc_client.send_chatbox_message(&formatted_message, self.chat_options.play_fx_resend && self.chat_options.play_fx_sound) {
                                                    eprintln!("Failed to send OSC message: {}", e);
                                                }
                                                self.last_osc_send = std::time::Instant::now();
                                            }
                                            self.config_changed = true;
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
                                            let formatted_message = if self.chat_options.add_speech_bubble {
                                                format!("🗨 {}", message.text)
                                            } else {
                                                message.text.clone()
                                            };
                                            message.sent_at_ms = now_ms;
                                            self.osc_preview = formatted_message.clone();
                                            if self.send_to_vrchat {
                                                if let Err(e) = self.osc_client.send_chatbox_message(&formatted_message, self.chat_options.play_fx_resend && self.chat_options.play_fx_sound) {
                                                    eprintln!("Failed to send OSC message: {}", e);
                                                }
                                                self.last_osc_send = std::time::Instant::now();
                                            }
                                            self.config_changed = true;
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
                                    if self.send_to_vrchat && !self.osc_preview.is_empty() {
                                        let mut message = self.osc_preview.clone();
                                        if self.extra_options.slim_mode {
                                            message.push_str("\u{0003}\u{001f}");
                                        }
                                        if let Err(e) = self.osc_client.send_chatbox_message(&message, false) {
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
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("Options");
            // Status Options
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut self.status_options.enabled, "").changed() {
                        self.config_changed = true;
                    }
                    ui.push_id(egui::Id::new("status_options"), |ui| {
                        ui.heading("Status Options");
                    });
                });
                if self.status_options.enabled {
                    let response = self.status_options.show_status_options(ui);
                    if response.changed() {
                        self.config_changed = true;
                    }
                }
            });
            ui.separator();
            // Time Options
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut self.time_options.config.enabled, "").changed() {
                        self.config_changed = true;
                    }
                    ui.push_id(egui::Id::new("time_options"), |ui| {
                        ui.heading("Time Options");
                    });
                });
                if self.time_options.config.enabled {
                    let response = self.time_options.show_time_options(ui);
                    if response.changed() {
                        self.config_changed = true;
                    }
                }
            });
            ui.separator();
            // Component Stats Options
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut self.component_stats.enabled, "").changed() {
                        self.config_changed = true;
                    }
                    ui.push_id(egui::Id::new("component_stats_options"), |ui| {
                        ui.heading("Component Stats Options");
                    });
                });
                if self.component_stats.enabled {
                    let response = self.component_stats.show_component_stats_options(ui);
                    if response.changed() {
                        self.config_changed = true;
                    }
                }
            });
            ui.separator();
            // Network Stats Options
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut self.network_stats.enabled, "").changed() {
                        self.config_changed = true;
                    }
                    ui.push_id(egui::Id::new("network_stats_options"), |ui| {
                        ui.heading("Network Stats Options");
                    });
                });
                if self.network_stats.enabled {
                    let response = self.network_stats.show_network_stats_options(ui);
                    if response.changed() {
                        self.config_changed = true;
                    }
                }
            });
            ui.separator();
            // Chatting Options
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut self.chat_options.enabled, "").changed() {
                        self.config_changed = true;
                    }
                    ui.push_id(egui::Id::new("chatting_options"), |ui| {
                        ui.heading("Chatting Options");
                    });
                });
                if self.chat_options.enabled {
                    let response = self.chat_options.show_chatting_options(ui);
                    if response.changed() {
                        self.config_changed = true;
                    }
                }
            });
            ui.separator();
            // MediaLink Options
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut self.media_link.enabled, "").changed() {
                        self.config_changed = true;
                    }
                    ui.push_id(egui::Id::new("medialink_options"), |ui| {
                        ui.heading("MediaLink Options");
                    });
                });
                if self.media_link.enabled {
                    let response = self.media_link.show_medialink_options(ui);
                    if response.changed() {
                        self.config_changed = true;
                    }
                }
            });
            ui.separator();
            // App Options
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut self.app_options.enabled, "").changed() {
                        self.config_changed = true;
                    }
                    ui.push_id(egui::Id::new("app_options"), |ui| {
                        ui.heading("App Options");
                    });
                });
                if self.app_options.enabled {
                    let response = self.app_options.show_app_options(ui);
                    if response.changed() {
                        self.config_changed = true;
                    }
                }
            });
            ui.separator();
            // Extra Options
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut self.extra_options.enabled, "").changed() {
                        self.config_changed = true;
                    }
                    ui.push_id(egui::Id::new("extra_options"), |ui| {
                        ui.heading("Extra Options");
                    });
                });
                if self.extra_options.enabled {
                    let response = self.extra_options.show_extra_options(ui);
                    if response.changed() {
                        self.config_changed = true;
                    }
                }
            });
        });
    }
    fn update_osc_preview(&mut self) {
        self.status_module.update_cycle(&self.status_options);
        let mut parts = Vec::new();

        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let mut active_chat_message = None;
        for message in self.chat_options.messages.iter() {
            if ((now_ms - message.sent_at_ms) / 1000) < self.chat_options.chat_timeout as u64 {
                active_chat_message = Some(message);
                break;
            }
        }

        // Collect other module data first
        if self.integrations_tab.personal_status_enabled {
            if let Some(status) = self.status_module.get_current_message(&self.status_options) {
                parts.push(status);
            }
        }

        if self.integrations_tab.component_stats_enabled {
            let stats = self.components_module.get_formatted_stats(&self.component_stats);
            if !stats.is_empty() {
                parts.push(stats);
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
        // Collect MediaLink data last to ensure it appears at the bottom
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
        self.previous_osc_preview = parts.join(separator);
        // Handle chat message priority
        if let Some(message) = active_chat_message {
            let message_text = if message.editing && self.chat_options.live_editing {
                &message.edit_text
            } else {
                &message.text
            };
            let mut formatted_message = if self.chat_options.add_speech_bubble {
                format!("🗨 {}", message_text)
            } else {
                message_text.clone()
            };
            if self.extra_options.slim_mode {
                formatted_message.push_str("\u{0003}\u{001f}");
            }
            // Append module data (with MediaLink at bottom) to chat message if available
            if !self.previous_osc_preview.is_empty() {
                self.osc_preview = format!("{}\n{}", formatted_message, self.previous_osc_preview);
            } else {
                self.osc_preview = formatted_message;
            }
            if self.send_to_vrchat
                && !self.osc_preview.is_empty()
                && message.editing
                && self.chat_options.live_editing
                && self.chat_options.override_display_time
                && self.last_osc_send.elapsed().as_secs_f32() >= self.chat_options.display_time_seconds
            {
                if let Err(e) = self.osc_client.send_chatbox_message(&self.osc_preview, self.chat_options.play_fx_sound) {
                    eprintln!("Failed to send OSC message: {}", e);
                }
                self.last_osc_send = std::time::Instant::now();
            }
        } else {
            // Use previous_osc_preview (with MediaLink at bottom) when no chat message
            self.osc_preview = self.previous_osc_preview.clone();
        }
        // Send OSC data if needed
        if self.send_to_vrchat
            && self.chat_options.can_send()
            && !self.osc_preview.is_empty()
            && self.last_osc_send.elapsed().as_secs_f32() >= self.app_options.app_options.osc_options.update_rate
        {
            if let Some(message) = self.chat_options.take_queued_message() {
                let formatted_message = if self.chat_options.add_speech_bubble {
                    format!("🗨 {}", message)
                } else {
                    message
                };
                // Append module data (with MediaLink at bottom) to queued message if available
                if !self.previous_osc_preview.is_empty() {
                    self.osc_preview = format!("{}\n{}", formatted_message, self.previous_osc_preview);
                } else {
                    self.osc_preview = formatted_message.clone();
                }
                if let Err(e) = self.osc_client.send_chatbox_message(&self.osc_preview, self.chat_options.play_fx_sound) {
                    eprintln!("Failed to send OSC message: {}", e);
                }
                self.chat_options.add_message(formatted_message);
                self.last_osc_send = std::time::Instant::now();
            } else if !self.osc_preview.is_empty() {
                let mut message = self.osc_preview.clone();
                if self.extra_options.slim_mode {
                    message.push_str("\u{0003}\u{001f}");
                }
                if let Err(e) = self.osc_client.send_chatbox_message(&message, self.chat_options.play_fx_sound) {
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
                component_stats_options: self.component_stats.clone(),
                extra_options: self.extra_options.clone(),
                media_link_options: self.media_link.clone(),
                network_stats_options: self.network_stats.clone(),
                status_options: self.status_options.clone(),
                time_options: self.time_options.clone(),
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
        // Top panel with title, tabs, and VRChat toggle
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("VRC OSC");
                    ui.heading("RustyChatBox");
                });
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.add_space(20.0);
                    // Integrations tab
                    let mut button = egui::Button::new("Integrations").min_size(egui::vec2(100.0, 40.0));
                    if self.current_tab == Tab::Integrations {
                        button = button.fill(egui::Color32::from_rgb(50, 50, 100)); // Highlight color
                    }
                    if ui.add(button).clicked() {
                        self.current_tab = Tab::Integrations;
                    }
                    // Status tab
                    let mut button = egui::Button::new("Status").min_size(egui::vec2(100.0, 40.0));
                    if self.current_tab == Tab::Status {
                        button = button.fill(egui::Color32::from_rgb(50, 50, 100));
                    }
                    if ui.add(button).clicked() {
                        self.current_tab = Tab::Status;
                    }
                    // Chatting tab
                    let mut button = egui::Button::new("Chatting").min_size(egui::vec2(100.0, 40.0));
                    if self.current_tab == Tab::Chatting {
                        button = button.fill(egui::Color32::from_rgb(50, 50, 100));
                    }
                    if ui.add(button).clicked() {
                        self.current_tab = Tab::Chatting;
                    }
                    // Options tab
                    let mut button = egui::Button::new("Options").min_size(egui::vec2(100.0, 40.0));
                    if self.current_tab == Tab::Options {
                        button = button.fill(egui::Color32::from_rgb(50, 50, 100));
                    }
                    if ui.add(button).clicked() {
                        self.current_tab = Tab::Options;
                    }
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.checkbox(&mut self.send_to_vrchat, "Send to VRChat");
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
                        if ui.checkbox(&mut self.live_edit_enabled, "Live edit chat messages").changed() {
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