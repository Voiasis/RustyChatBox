// gui.rs
use eframe::egui;
use serde::{Serialize, Deserialize};
use std::fs;
use std::process::Command;
use crate::modules::{
    appo_mod::{AppOptions, AppOptionsOptions},
    chat_mod::ChatOptions,
    comp_mod::{ComponentStatsModule, ComponentStatsOptions},
    extr_mod::ExtraOptions,
    medi_mod::{MediaLinkModule, MediaLinkOptions},
    netw_mod::{NetworkStats, NetworkStatsOptions},
    stat_mod::{StatusModule, StatusOptions},
    time_mod::{TimeModule, TimeOptions},
};
use crate::osc::OscClient;

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
}

impl App {
    pub fn new(osc_client: OscClient, config: Config) -> Self {
        Self {
            current_tab: Tab::Integrations,
            app_options: AppOptionsOptions {
                app_options: config.app_options,
                enabled: true,
            },
            chat_tab: ChatTab {
                message: String::new(),
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
        }
    }

    fn update_osc_preview(&mut self) {
        self.status_module.update_cycle(&self.status_options);
        let mut parts = Vec::new();
    
        // Add Personal Status
        if self.integrations_tab.personal_status_enabled {
            if let Some(status) = self.status_module.get_current_message(&self.status_options) {
                parts.push(status);
            }
        }
    
        // Add Component Stats (moved above Time)
        if self.integrations_tab.component_stats_enabled {
            let stats = self.components_module.get_formatted_stats(&self.component_stats);
            if !stats.is_empty() {
                parts.push(stats);
            }
        }
    
        // Add Network Stats
        if self.integrations_tab.network_stats_enabled {
            let interfaces = NetworkStats::get_interfaces();
            if let Some(iface) = interfaces.first() {
                let stats = NetworkStats::get_formatted_stats(&self.network_stats.config, &iface.name);
                if !stats.is_empty() {
                    parts.push(stats);
                }
            }
        }
    
        // Add Current Time
        if self.integrations_tab.current_time_enabled {
            let time = TimeModule::get_local_time(&self.time_options);
            parts.push(time);
        }
    
        // Add MediaLink
        if self.integrations_tab.medialink_enabled {
            if let Some(track) = self.media_module.get_formatted_track(&self.media_link) {
                parts.push(track);
            }
        }
    
        // Join all parts with the appropriate separator
        let separator = if self.app_options.app_options.osc_options.separate_lines {
            "\n"
        } else {
            " | "
        };
        self.osc_preview = parts.join(separator);
    
        // Send to VRChat if enabled
        if self.send_to_vrchat
            && !self.osc_preview.is_empty()
            && self.last_osc_send.elapsed().as_secs_f32() >= self.app_options.app_options.osc_options.update_rate
        {
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
                    if ui.add(egui::Button::new("Integrations").min_size(egui::vec2(100.0, 40.0))).clicked() {
                        self.current_tab = Tab::Integrations;
                    }
                    if ui.add(egui::Button::new("Status").min_size(egui::vec2(100.0, 40.0))).clicked() {
                        self.current_tab = Tab::Status;
                    }
                    if ui.add(egui::Button::new("Chatting").min_size(egui::vec2(100.0, 40.0))).clicked() {
                        self.current_tab = Tab::Chatting;
                    }
                    if ui.add(egui::Button::new("Options").min_size(egui::vec2(100.0, 40.0))).clicked() {
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
    .resizable(false) // Prevent resizing
    .default_width(300.0) // Set the width of the side panel
    .show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.button("Discord").clicked() {
                if let Err(e) = open::that("https://discord.gg/kzYjRnppFn") {
                    eprintln!("Failed to open GitHub URL: {}", e);
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
        let size = 300.0; // Set the size of the square
        ui.allocate_ui(egui::vec2(size, size), |ui| {
            ui.group(|ui| {
                ui.set_min_size(egui::vec2(size, size)); // Ensure the group is a square
                ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::TopDown), |ui| {
                    ui.label(&self.osc_preview);
                });
            });
        });

        ui.separator();
    });

        // Central panel with tab content
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_tab {
                Tab::Integrations => self.show_integrations_tab(ui),
                Tab::Status => self.show_status_tab(ui),
                Tab::Chatting => self.show_chatting_tab(ui),
                Tab::Options => self.show_options_tab(ui),
            }
        });

        ctx.request_repaint();
    }
}
impl App {
    fn show_integrations_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Integrations");
        // Personal Status
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.heading("Personal Status");
                if ui.button("⚙").clicked() {
                    self.current_tab = Tab::Options;
                    self.scroll_to = Some(egui::Id::new("status_options"));
                    self.status_options.enabled = true;
                    self.config_changed = true;
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
                    self.current_tab = Tab::Options;
                    self.scroll_to = Some(egui::Id::new("component_stats_options"));
                    self.component_stats.enabled = true;
                    self.config_changed = true;
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
                    self.current_tab = Tab::Options;
                    self.scroll_to = Some(egui::Id::new("network_stats_options"));
                    self.network_stats.enabled = true;
                    self.config_changed = true;
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
                    self.current_tab = Tab::Options;
                    self.scroll_to = Some(egui::Id::new("time_options"));
                    self.time_options.config.enabled = true;
                    self.config_changed = true;
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
                    self.current_tab = Tab::Options;
                    self.scroll_to = Some(egui::Id::new("medialink_options"));
                    self.media_link.enabled = true;
                    self.config_changed = true;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.checkbox(&mut self.integrations_tab.medialink_enabled, "").changed() {
                        self.config_changed = true;
                    }
                });
            });
            ui.label("Show the current media track and artist.");
            if self.integrations_tab.medialink_enabled {
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
                if let Err(e) = self.osc_client.send_chatbox_message(&current, false) {
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
        ui.heading("Chatting");
        ui.label("Send and manage chat messages.");
        ui.horizontal(|ui| {
            ui.label("Message: ");
            ui.text_edit_singleline(&mut self.chat_tab.message);
            if ui.button("Send").clicked() && !self.chat_tab.message.is_empty() {
                let formatted_message = if self.chat_options.add_speech_bubble {
                    format!("🗨 {}", self.chat_tab.message)
                } else {
                    self.chat_tab.message.clone()
                };
                if let Err(e) = self.osc_client.send_chatbox_message(&formatted_message, self.chat_options.play_fx_sound) {
                    eprintln!("Failed to send message: {}", e);
                }
                self.chat_tab.message.clear();
            }
        });
    }
    fn show_options_tab(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            // Handle scrolling to a specific section
            if let Some(scroll_id) = self.scroll_to.take() {
                ui.ctx().request_repaint(); // Ensure a repaint to render widgets
                ui.ctx().memory_mut(|mem| {
                    mem.request_focus(scroll_id); // Request focus to scroll to the widget
                });
            }
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
}

// Implementations for individual option structs
impl AppOptionsOptions {
    pub fn show_app_options(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let mut response = ui.interact(
            egui::Rect::EVERYTHING,
            ui.id().with("app_options"),
            egui::Sense::hover(),
        );
        ui.label("Local IP address");
        ui.horizontal(|ui| {
            response |= ui.text_edit_singleline(&mut self.app_options.osc_options.ip);
            if ui.button("Default").clicked() {
                self.app_options.osc_options.ip = "127.0.0.1".to_string();
                response.mark_changed();
            }
        });
        ui.label("Port (outgoing)");
        ui.horizontal(|ui| {
            let mut port_str = self.app_options.osc_options.port.to_string();
            response |= ui.text_edit_singleline(&mut port_str);
            if let Ok(port) = port_str.parse() {
                self.app_options.osc_options.port = port;
            }
            if ui.button("Default").clicked() {
                self.app_options.osc_options.port = 9000;
                response.mark_changed();
            }
        });
        ui.label("OSC update rate");
        response |= ui.add(egui::Slider::new(&mut self.app_options.osc_options.update_rate, 1.6..=10.0).text("seconds"));
        response |= ui.checkbox(
            &mut self.app_options.osc_options.separate_lines,
            "Separate integrations on a separate line instead of '|'",
        );
        ui.horizontal(|ui| {
            if ui.button("Open config folder").clicked() {
                let path = dirs::config_dir().unwrap().join("RustyChatBox");
                Command::new("xdg-open").arg(path).spawn().ok();
            }
            if ui.button("Open logs").clicked() {
                let path = dirs::config_dir().unwrap().join("RustyChatBox/app.log");
                Command::new("xdg-open").arg(path).spawn().ok();
            }
        });
        response
    }
}

impl ChatOptions {
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
        response |= ui.checkbox(&mut self.override_display_time, "Override display time for chat messages");
        if self.override_display_time {
            response |= ui.add(egui::Slider::new(&mut self.display_time_seconds, 2.0..=10.0).text("seconds"));
        }
        response |= ui.checkbox(&mut self.edit_messages, "Edit chat messages");
        if self.edit_messages {
            response |= ui.checkbox(&mut self.live_editing, "Live editing");
        }
        response
    }
}

impl ComponentStatsOptions {
    pub fn show_component_stats_options(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let mut response = ui.interact(
            egui::Rect::EVERYTHING,
            ui.id().with("component_stats_options"),
            egui::Sense::hover(),
        );
        response |= ui.checkbox(&mut self.show_cpu, "Show CPU stats");
        response |= ui.checkbox(&mut self.cpu_display_model, "Display CPU model name");
        if self.cpu_display_model {
            ui.horizontal(|ui| {
                ui.label("Custom CPU model:");
                response |= ui.text_edit_singleline(self.cpu_custom_model.get_or_insert(String::new()));
            });
        }
        response |= ui.checkbox(&mut self.cpu_round_usage, "Round CPU usage percentage");
        response |= ui.checkbox(&mut self.cpu_stylized_uppercase, "Stylized uppercase letters for CPU");

        response |= ui.checkbox(&mut self.show_gpu, "Show GPU stats");
        response |= ui.checkbox(&mut self.gpu_display_model, "Display GPU model name");
        if self.gpu_display_model {
            ui.horizontal(|ui| {
                ui.label("Custom GPU model:");
                response |= ui.text_edit_singleline(self.gpu_custom_model.get_or_insert(String::new()));
            });
        }
        response |= ui.checkbox(&mut self.gpu_round_usage, "Round GPU usage percentage");
        response |= ui.checkbox(&mut self.gpu_stylized_uppercase, "Stylized uppercase letters for GPU");

        response |= ui.checkbox(&mut self.show_vram, "Show VRAM stats");
        response |= ui.checkbox(&mut self.vram_round_usage, "Round VRAM usage in GB");
        response |= ui.checkbox(&mut self.vram_show_max, "Show max VRAM capacity");
        response |= ui.checkbox(&mut self.vram_stylized_uppercase, "Stylized uppercase letters for VRAM");

        response |= ui.checkbox(&mut self.show_ram, "Show RAM stats");
        response |= ui.checkbox(&mut self.ram_round_usage, "Round RAM usage in GB");
        response |= ui.checkbox(&mut self.ram_show_max, "Show max RAM capacity");
        response |= ui.checkbox(&mut self.ram_stylized_uppercase, "Stylized uppercase letters for RAM");
        response
    }
}

impl ExtraOptions {
    pub fn show_extra_options(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let mut response = ui.interact(
            egui::Rect::EVERYTHING,
            ui.id().with("extra_options"),
            egui::Sense::hover(),
        );
        response |= ui.checkbox(&mut self.slim_mode, "Slim Mode");
        response
    }
}

impl MediaLinkOptions {
    pub fn show_medialink_options(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let mut response = ui.interact(
            egui::Rect::EVERYTHING,
            ui.id().with("medialink_options"),
            egui::Sense::hover(),
        );
        ui.label("Basic options");
        ui.label(egui::RichText::new("Customize how your media looks in your chatbox").text_style(egui::TextStyle::Small));
        response |= ui.checkbox(&mut self.use_music_note_prefix, "Change 'Listening to:' prefix to 🎵");
        response |= ui.checkbox(&mut self.show_pause_emoji, "Show ⏸ when music is paused");
        response |= ui.checkbox(&mut self.auto_switch_state, "Auto switch when media state changes");
        response |= ui.checkbox(&mut self.auto_switch_session, "Auto switch when a new session is detected");
        ui.horizontal(|ui| {
            ui.label("Forget session after");
            response |= ui.add(egui::DragValue::new(&mut self.forget_session_seconds).speed(1.0));
            ui.label("seconds");
        });
        ui.label("Media progress bar");
        ui.label(egui::RichText::new("Customize how your seek bar looks").text_style(egui::TextStyle::Small));
        ui.label("Seekbar style");
        let combo_response = egui::ComboBox::from_label("")
            .selected_text(&self.seekbar_style)
            .show_ui(ui, |ui| {
                for style in ["Small numbers", "Custom", "None"].iter() {
                    if ui.selectable_value(&mut self.seekbar_style, style.to_string(), *style).changed() {
                        response.mark_changed();
                    }
                }
            });
        response |= combo_response.response;
        response |= ui.checkbox(&mut self.show_progress, "Show media progress");
        response
    }
}

impl StatusOptions {
    pub fn show_status_options(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let mut response = ui.interact(
            egui::Rect::EVERYTHING,
            ui.id().with("status_options"),
            egui::Sense::hover(),
        );
        response |= ui.checkbox(&mut self.cycle_status, "Cycle status messages");
        response |= ui.checkbox(&mut self.add_speech_bubble, "Add 🗨 as a prefix for the personal status");
        response |= ui.checkbox(&mut self.enable_custom_prefix_shuffle, "Enable custom prefix icon shuffle");
        if self.enable_custom_prefix_shuffle {
            response |= ui.text_edit_singleline(&mut self.custom_prefixes);
        }
        response |= ui.checkbox(&mut self.cycle_random, "Cycle status items in a random order");
        ui.horizontal(|ui| {
            ui.label("Cycle to the next every ");
            response |= ui.add(egui::DragValue::new(&mut self.cycle_interval).speed(1.0));
            ui.label(" seconds");
        });
        response
    }
}

impl TimeOptions {
    pub fn show_time_options(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let mut response = ui.interact(
            egui::Rect::EVERYTHING,
            ui.id().with("time_options"),
            egui::Sense::hover(),
        );
        response |= ui.checkbox(&mut self.config.show_my_time_prefix, "Show 'My time:' in front of the time integration");
        response |= ui.checkbox(&mut self.config.use_24_hour, "24-Hour time format");
        response |= ui.checkbox(&mut self.config.use_system_culture, "Use current system culture for formatting time");
        response |= ui.checkbox(&mut self.config.auto_dst, "Auto daylight savings time");
        let mut use_custom_tz = self.config.custom_timezone.is_some();
        response |= ui.checkbox(&mut use_custom_tz, "Custom time zone");
        if use_custom_tz {
            let mut tz_str = self.config.custom_timezone.clone().unwrap_or_default();
            let combo_response = egui::ComboBox::from_label("")
                .selected_text(&tz_str)
                .show_ui(ui, |ui| {
                    for tz in chrono_tz::TZ_VARIANTS.iter() {
                        if ui.selectable_value(&mut tz_str, tz.to_string(), tz.to_string()).changed() {
                            response.mark_changed();
                        }
                    }
                });
            response |= combo_response.response;
            self.config.custom_timezone = Some(tz_str);
        } else {
            self.config.custom_timezone = None;
        }
        response
    }
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub app_options: AppOptions,
    pub personal_status_enabled: bool,
    pub component_stats_enabled: bool,
    pub network_stats_enabled: bool,
    pub current_time_enabled: bool,
    pub medialink_enabled: bool,
    pub chat_options: ChatOptions,
    pub component_stats_options: ComponentStatsOptions,
    pub extra_options: ExtraOptions,
    pub media_link_options: MediaLinkOptions,
    pub network_stats_options: NetworkStatsOptions,
    pub status_options: StatusOptions,
    pub time_options: TimeOptions,
}