use eframe::egui;
use crate::modules::{stat_mod::Status, comp_mod::ComponentStats, time_mod::TimeModule, medi_mod::MediaLink, netw_mod::NetworkStats};
use crate::osc::OscClient;
use crate::options::{Config};
use std::fs;
use std::process::Command;

pub struct App {
    current_tab: Tab,
    status: Status,
    components: ComponentStats,
    time_module: TimeModule,
    media_link: MediaLink,
    network_stats: NetworkStats,
    osc_preview: String,
    send_to_vrchat: bool,
    config: Config,
    osc_client: OscClient,
    last_osc_send: std::time::Instant,
    personal_status_enabled: bool,
    component_stats_enabled: bool,
    network_stats_enabled: bool,
    current_time_enabled: bool,
    medialink_enabled: bool,
    medialink_video_mode: bool,
    medialink_show_artist: bool,
    medialink_auto_switch: bool,
    detected_gpus: Vec<String>,
    config_changed: bool, 
}

#[derive(PartialEq)]
enum Tab {
    Integrations,
    Status,
    Chatting,
    Options,
}

impl App {
    pub fn new(osc_client: OscClient, config: Config) -> Self {
        let detected_gpus = ComponentStats::get_available_gpus(); // Query GPUs once
        Self {
            current_tab: Tab::Integrations,
            status: Status::new(),
            components: ComponentStats::new(),
            time_module: TimeModule,
            media_link: MediaLink,
            network_stats: NetworkStats,
            osc_preview: String::new(),
            send_to_vrchat: false,
            config,
            osc_client,
            last_osc_send: std::time::Instant::now(),
            personal_status_enabled: true,
            component_stats_enabled: true,
            network_stats_enabled: true,
            current_time_enabled: true,
            medialink_enabled: true,
            medialink_video_mode: false,
            medialink_show_artist: true,
            medialink_auto_switch: false,
            detected_gpus, // Cache the detected GPUs
            config_changed: false, // Initialize the new field
        }
    }

    fn save_config(&self) {
        // Save the configuration to a file
        if let Err(e) = std::fs::write("config.json", serde_json::to_string(&self.config).unwrap()) {
            log::error!("Failed to save config: {}", e);
        }
    }

    fn save_config_if_needed(&mut self) {
        if self.config_changed {
            self.save_config();
            self.config_changed = false;
        }
    }
    
    fn update_osc_preview(&mut self) {
        let mut parts = Vec::new();

        if self.personal_status_enabled {
            if let Some(status) = self.status.get_current_message(&self.config.status_options) {
                let status = if self.config.status_options.add_speech_bubble {
                    format!("🗨️ {}", status)
                } else {
                    status
                };
                parts.push(status);
            }
        }

        if self.component_stats_enabled {
            parts.push(self.components.get_formatted_stats(&self.config.component_stats_options));
        }

        if self.current_time_enabled {
            let time = TimeModule::get_local_time(&self.config.time_options);
            let time = if self.config.time_options.show_my_time_prefix {
                format!("My time: {}", time)
            } else {
                time
            };
            parts.push(time);
        }

        if self.medialink_enabled {
            if let Some(track) = self.media_link.get_formatted_track(&self.config.medialink_options) {
                parts.push(track);
            }
        }

        let separator = if self.config.osc_options.separate_lines {
            "\n"
        } else {
            " | "
        };
        self.osc_preview = parts.join(separator);
        if self.config.app_options.slim_mode {
            self.osc_preview.push_str("\\u0003\\u001f");
        }

        if self.send_to_vrchat
            && !self.osc_preview.is_empty()
            && self.last_osc_send.elapsed().as_secs_f32() >= self.config.osc_options.update_rate
        {
            if let Err(e) = self.osc_client.send_chatbox_message(&self.osc_preview, self.config.chatting_options.play_fx_sound) {
                log::error!("Failed to send OSC message: {}", e);
            }
            self.last_osc_send = std::time::Instant::now();
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_osc_preview();

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

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_tab {
                Tab::Integrations => self.show_integrations_tab(ui),
                Tab::Status => self.show_status_tab(ui),
                Tab::Chatting => self.show_chatting_tab(ui),
                Tab::Options => self.show_options_tab(ui),
            }
        });

        egui::SidePanel::right("right_panel")
            .resizable(false)
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.heading("Preview");
                ui.group(|ui| {
                    ui.set_min_size(egui::vec2(200.0, 200.0));
                    ui.label(&self.osc_preview);
                });

                ui.separator();

                if ui.button("GitHub").clicked() {
                    // Placeholder
                }
                if ui.button("Discord").clicked() {
                    // Placeholder
                }
            });

        ctx.request_repaint();
        self.save_config(); // Save config on every UI update
    }
}

impl App {
    fn show_integrations_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Integrations");
    
        // Personal Status Section
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.personal_status_enabled, "");
                ui.heading("Personal Status");
                if ui.button("⚙").clicked() {
                    self.current_tab = Tab::Options; // Shortcut to Options tab
                }
            });
            ui.label("Manage your personal status messages.");
        });
    
        ui.separator();
    
        // Component Stats Section
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.component_stats_enabled, "");
                ui.heading("Component Stats");
                if ui.button("⚙").clicked() {
                    self.current_tab = Tab::Options; // Shortcut to Options tab
                }
            });
            ui.label("View system performance stats (CPU, GPU, RAM, etc.).");
        });
    
        ui.separator();
    
        // Network Stats Section
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.network_stats_enabled, "");
                ui.heading("Network Stats");
                if ui.button("⚙").clicked() {
                    self.current_tab = Tab::Options; // Shortcut to Options tab
                }
            });
            ui.label("Monitor real-time network statistics (download/upload speeds).");
        });
    
        ui.separator();
    
        // Current Time Section
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.current_time_enabled, "");
                ui.heading("Current Time");
                if ui.button("⚙").clicked() {
                    self.current_tab = Tab::Options; // Shortcut to Options tab
                }
            });
            ui.label("Display the current local time and other time zones.");
        });
    
        ui.separator();

        // MediaLink Section
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.medialink_enabled, ""); // Add checkbox for MediaLink
                ui.heading("MediaLink");
                if ui.button("⚙").clicked() {
                    self.current_tab = Tab::Options; // Shortcut to Options tab
                }
            });
            ui.label("Show the current media track and artist.");
        });
    }

    fn show_status_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Status");
        ui.label("Manage personal status messages.");
    }

    fn show_chatting_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Chatting");
        ui.label("Send and manage chat messages.");
    }

    fn show_options_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Options");
    
        // Status Options
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.config.status_options.enabled, ""); // Add checkbox to toggle section
            ui.label(egui::RichText::new("Status options").text_style(egui::TextStyle::Button)); // Smaller title
        });
        if self.config.status_options.enabled {
            ui.group(|ui| {
                ui.checkbox(&mut self.config.status_options.add_speech_bubble, "Add 🗨️ as a prefix for the personal status");
                ui.checkbox(&mut self.config.status_options.enable_custom_prefix_shuffle, "Enable custom prefix icon shuffle");
                if self.config.status_options.enable_custom_prefix_shuffle {
                    ui.text_edit_singleline(&mut self.config.status_options.custom_prefixes);
                }
                ui.checkbox(&mut self.config.status_options.cycle_status, "Cycle status items that are toggled on");
                ui.checkbox(&mut self.config.status_options.cycle_random, "Cycle status items in a random order");
                ui.horizontal(|ui| {
                    ui.label("Cycle to the next every ");
                    ui.add(egui::DragValue::new(&mut self.config.status_options.cycle_interval).speed(1.0));
                    ui.label(" seconds");
                });
            });
        }
    
        ui.separator();
    
        // Time Options
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.config.time_options.enabled, ""); // Add checkbox to toggle section
            ui.label(egui::RichText::new("Time options").text_style(egui::TextStyle::Button)); // Smaller title
        });
        if self.config.time_options.enabled {
            ui.group(|ui| {
                ui.checkbox(&mut self.config.time_options.show_my_time_prefix, "Show 'My time:' in front of the time integration");
                ui.checkbox(&mut self.config.time_options.use_24_hour, "24-Hour time format");
                ui.checkbox(&mut self.config.time_options.use_system_culture, "Use current system culture for formatting time");
                ui.checkbox(&mut self.config.time_options.auto_dst, "Auto daylight savings time");
                ui.checkbox(&mut self.config.time_options.custom_timezone.is_some(), "Custom time zone");
                if self.config.time_options.custom_timezone.is_some() {
                    let mut tz_str = self.config.time_options.custom_timezone.clone().unwrap_or_default();
                    egui::ComboBox::from_label("")
                        .selected_text(&tz_str)
                        .show_ui(ui, |ui| {
                            for tz in chrono_tz::TZ_VARIANTS.iter() {
                                if ui.selectable_label(tz_str == tz.to_string(), tz.to_string()).clicked() {
                                    tz_str = tz.to_string();
                                }
                            }
                        });
                    self.config.time_options.custom_timezone = Some(tz_str);
                }
            });
        }
    
        ui.separator();
    
        // Component Stats Options
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.config.component_stats_options.enabled, ""); // Add checkbox to toggle section
            ui.label(egui::RichText::new("Component stats options").text_style(egui::TextStyle::Button)); // Smaller title
        });
        if self.config.component_stats_options.enabled {
            ui.group(|ui| {
                ui.checkbox(&mut self.config.component_stats_options.auto_switch_temp_units, "Auto switch temperature units");
                ui.checkbox(&mut self.config.component_stats_options.use_emoji_temp_power, "Use emojis for temperature and power");
                ui.checkbox(&mut self.config.component_stats_options.auto_select_gpu, "Auto select GPU");
                if !self.config.component_stats_options.auto_select_gpu {
                    let mut gpu = self.config.component_stats_options.selected_gpu.clone().unwrap_or_default();
                    egui::ComboBox::from_id_source("gpu_dropdown")
                        .selected_text(if gpu.is_empty() { "Select GPU" } else { &gpu })
                        .show_ui(ui, |ui| {
                            for gpu_name in self.detected_gpus.iter() {
                                if ui.selectable_value(&mut gpu, gpu_name.clone(), gpu_name).clicked() {
                                    self.config.component_stats_options.selected_gpu = Some(gpu.clone());
                                }
                            }
                        });
                }
                // CPU Section
                ui.horizontal(|ui| {
                    ui.label("CPU: ");
                    ui.label(self.components.get_cpu_model());
                });
                // CPU Options
                ui.checkbox(&mut self.config.component_stats_options.cpu_show_wattage, "Show CPU wattage");
                ui.checkbox(&mut self.config.component_stats_options.cpu_show_temp, "Show CPU temperature");
                ui.checkbox(&mut self.config.component_stats_options.cpu_stylized_uppercase, "Stylized uppercase letters");
                ui.checkbox(&mut self.config.component_stats_options.cpu_display_model, "Display model name instead of 'CPU'");
                ui.checkbox(&mut self.config.component_stats_options.cpu_round_usage, "Round off CPU usage percentage");
                ui.checkbox(&mut self.config.component_stats_options.cpu_custom_model.is_some(), "Set custom CPU model name");
                if self.config.component_stats_options.cpu_custom_model.is_some() && self.config.component_stats_options.cpu_display_model {
                    let mut custom = self.config.component_stats_options.cpu_custom_model.clone().unwrap_or_default();
                    ui.text_edit_singleline(&mut custom);
                    self.config.component_stats_options.cpu_custom_model = Some(custom);
                }
                if self.config.component_stats_options.gpu_display_model {
                    let gpu_name = self.config.component_stats_options.selected_gpu.clone().unwrap_or("GPU".to_string());
                    ui.label(format!("GPU: {}", gpu_name));
                }
                // GPU Section
                ui.checkbox(&mut self.config.component_stats_options.gpu_show_wattage, "Show GPU wattage");
                ui.checkbox(&mut self.config.component_stats_options.gpu_show_temp, "Show GPU temperature");
                ui.checkbox(&mut self.config.component_stats_options.gpu_show_hotspot_temp, "Show GPU hotspot temperature");
                ui.checkbox(&mut self.config.component_stats_options.gpu_stylized_uppercase, "Stylized uppercase letters");
                ui.checkbox(&mut self.config.component_stats_options.gpu_3d_usage, "Utilization is based on the 3D usage");
                ui.checkbox(&mut self.config.component_stats_options.gpu_display_model, "Display model name instead of 'GPU'");
                ui.checkbox(&mut self.config.component_stats_options.gpu_round_usage, "Round off GPU usage percentage");
                ui.checkbox(&mut self.config.component_stats_options.gpu_custom_model.is_some(), "Set custom GPU model name");
                // VRAM Options
                ui.label("VRAM:");
                ui.checkbox(&mut self.config.component_stats_options.vram_stylized_uppercase, "Stylized uppercase letters");
                ui.checkbox(&mut self.config.component_stats_options.vram_3d_usage, "VRAM usage is based on the 3D usage");
                ui.checkbox(&mut self.config.component_stats_options.vram_show_max, "Show max VRAM capacity");
                ui.checkbox(&mut self.config.component_stats_options.vram_round_usage, "Round off GPU VRAM usage in GBs");
                // RAM Options
                ui.label("RAM:");
                ui.checkbox(&mut self.config.component_stats_options.ram_stylized_uppercase, "Stylized uppercase letters");
                ui.checkbox(&mut self.config.component_stats_options.ram_show_max, "Show max RAM capacity");
                ui.checkbox(&mut self.config.component_stats_options.ram_show_ddr, "Show RAM DDR version");
                ui.checkbox(&mut self.config.component_stats_options.ram_round_usage, "Round off RAM usage in GBs");
            });
        }
    
        ui.separator();
    
        // Network Statistics Options
        ui.group(|ui| {
            ui.heading("Network statistics options");
            ui.checkbox(&mut self.config.network_options.use_interface_max_speed, "Use network interface as the max speed");
            ui.checkbox(&mut self.config.network_options.show_download_speed, "Show current download speed");
            ui.checkbox(&mut self.config.network_options.show_upload_speed, "Show current upload speed");
            ui.checkbox(&mut self.config.network_options.show_max_download, "Show max download speed");
            ui.checkbox(&mut self.config.network_options.show_max_upload, "Show max upload speed");
            ui.checkbox(&mut self.config.network_options.show_total_download, "Show total download");
            ui.checkbox(&mut self.config.network_options.show_total_upload, "Show total upload");
            ui.checkbox(&mut self.config.network_options.show_utilization, "Show network utilization");
            ui.checkbox(&mut self.config.network_options.stylized_chars, "Stylized characters");
        });
    
        ui.separator();
    
        // Chatting Options
        ui.group(|ui| {
            ui.heading("Chatting options");
            ui.horizontal(|ui| {
                ui.label("Chat timeout ");
                ui.add(egui::DragValue::new(&mut self.config.chatting_options.chat_timeout).speed(1.0));
                ui.label(" seconds");
            });
            ui.checkbox(&mut self.config.chatting_options.add_speech_bubble, "Add 🗨️ as prefix for chat messages");
            ui.checkbox(&mut self.config.chatting_options.use_custom_idle_prefix, "Use your custom idle icon shuffle as a prefix for chat messages");
            ui.checkbox(&mut self.config.chatting_options.play_fx_sound, "Play FX sound for VRChat users when sending messages");
            if self.config.chatting_options.play_fx_sound {
                ui.checkbox(&mut self.config.chatting_options.play_fx_resend, "Play FX when clicking resend");
            }
            ui.checkbox(&mut self.config.chatting_options.small_delay, "Small delay when sending a message");
            if self.config.chatting_options.small_delay {
                ui.add(egui::Slider::new(&mut self.config.chatting_options.delay_seconds, 0.1..=2.0).text("seconds"));
            }
            ui.checkbox(&mut self.config.chatting_options.override_display_time, "Override display time for chat messages");
            if self.config.chatting_options.override_display_time {
                ui.add(egui::Slider::new(&mut self.config.chatting_options.display_time_seconds, 2.0..=10.0).text("seconds"));
            }
            ui.checkbox(&mut self.config.chatting_options.edit_messages, "Edit chat messages");
            if self.config.chatting_options.edit_messages {
                ui.checkbox(&mut self.config.chatting_options.live_editing, "Live editing");
            }
        });
    
        ui.separator();
    
        // MediaLink Options
        ui.group(|ui| {
            ui.heading("MediaLink options");
            ui.label("Basic options");
            ui.label(egui::RichText::new("Customize here how you want your media to look in your chatbox").text_style(egui::TextStyle::Small));
            ui.checkbox(&mut self.config.medialink_options.use_music_note_prefix, "Change 'Listening to:' prefix to 🎵");
            ui.checkbox(&mut self.config.medialink_options.show_pause_emoji, "Show ⏸️ when music is paused");
            ui.label("MediaLink behavior");
            ui.label(egui::RichText::new("Here you can adjust the way MediaLink handles sessions").text_style(egui::TextStyle::Small));
            ui.checkbox(&mut self.config.medialink_options.auto_switch_state, "Auto switch when media state is changed");
            ui.checkbox(&mut self.config.medialink_options.auto_switch_session, "Turn on auto switch when new session is detected");
            ui.horizontal(|ui| {
                ui.label("Forget session ");
                ui.add(egui::DragValue::new(&mut self.config.medialink_options.forget_session_seconds).speed(1.0));
                ui.label(" seconds after disconnect");
            });
            ui.label("Media progress bar");
            ui.label(egui::RichText::new("Customize how your seek bar looks like").text_style(egui::TextStyle::Small));
            ui.label("Seekbar style");
            egui::ComboBox::from_label("")
                .selected_text(&self.config.medialink_options.seekbar_style)
                .show_ui(ui, |ui| {
                    for style in ["Small numbers", "Custom", "None"].iter() {
                        ui.selectable_value(&mut self.config.medialink_options.seekbar_style, style.to_string(), *style);
                    }
                });
            ui.checkbox(&mut self.config.medialink_options.show_progress, "Show media progress");
        });
    
        ui.separator();
    
        // App Options
        ui.group(|ui| {
            ui.heading("App options");
            ui.label("Local IP address");
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.config.osc_options.ip);
                if ui.button("Default").clicked() {
                    self.config.osc_options.ip = "127.0.0.1".to_string();
                }
            });
            ui.label("Port (outgoing)");
            ui.horizontal(|ui| {
                let mut port_str = self.config.osc_options.port.to_string();
                ui.text_edit_singleline(&mut port_str);
                if let Ok(port) = port_str.parse() {
                    self.config.osc_options.port = port;
                }
                if ui.button("Default").clicked() {
                    self.config.osc_options.port = 9000;
                }
            });
            ui.label("OSC update rate");
            ui.add(egui::Slider::new(&mut self.config.osc_options.update_rate, 1.6..=10.0).text("seconds"));
            ui.checkbox(&mut self.config.osc_options.separate_lines, "Separate Integrations on a separate line instead of '|'");
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
        });
    
        ui.separator();
    
        // Extra Options
        ui.group(|ui| {
            ui.heading("Extra options");
            ui.checkbox(&mut self.config.app_options.slim_mode, "Slim Mode");
        });
    }
}