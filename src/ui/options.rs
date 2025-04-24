use eframe::egui::{self, Ui, Align};
use log::{debug, info, error};
use crate::ui::App;

pub fn show_options_tab(ui: &mut Ui, app: &mut App) {
    let mut scroll_to_rect = None;
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.heading("Options");

        // Debug: Confirm Window Activity enabled state
        ui.label(format!("Debug: window_activity.enabled = {}", app.window_activity.enabled));

        // Status Options
        let status_response = ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), 0.0),
            egui::Layout::top_down(Align::LEFT),
            |ui| {
                ui.group(|ui| {
                    ui.push_id(egui::Id::new("status_options"), |ui| {
                        ui.horizontal(|ui| {
                            let response = ui.checkbox(&mut app.status_options.enabled, "");
                            if response.changed() {
                                debug!("Status options enabled checkbox changed");
                                app.config_changed = true;
                            }
                            ui.heading("Status Options");
                        });
                        if app.status_options.enabled {
                            let response = app.status_options.show_status_options(ui);
                            if response.changed() {
                                app.config_changed = true;
                            }
                        }
                    });
                });
            },
        );
        if app.pending_scroll_to == Some(egui::Id::new("status_options")) {
            scroll_to_rect = Some(status_response.response.rect);
        }
        ui.separator();

        // Window Activity Options
        let window_activity_response = ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), 0.0),
            egui::Layout::top_down(Align::LEFT),
            |ui| {
                ui.group(|ui| {
                    ui.push_id(egui::Id::new("window_activity_options"), |ui| {
                        ui.horizontal(|ui| {
                            let response = ui.checkbox(&mut app.window_activity.enabled, "");
                            if response.changed() {
                                debug!("Window activity enabled checkbox changed");
                                app.integrations_tab.window_activity_enabled = app.window_activity.enabled;
                                app.config_changed = true;
                            }
                            ui.heading("Window Activity Options");
                        });
                        if app.window_activity.enabled {
                            let response = app.window_activity.show_window_activity_options(ui);
                            if response.changed() {
                                debug!("Window activity options changed");
                                app.config_changed = true;
                                app.update_osc_preview();
                                if app.send_to_vrchat && !app.osc_preview.is_empty() {
                                    if let Err(e) = app.osc_client.send_chatbox_message(
                                        &app.osc_preview,
                                        false,
                                        app.extra_options.slim_mode,
                                    ) {
                                        error!("Failed to send OSC message for window activity: {}", e);
                                    } else {
                                        info!("Sent OSC message for window activity: {}", app.osc_preview);
                                    }
                                    app.last_osc_send = std::time::Instant::now();
                                }
                            }
                        } else {
                            ui.label("Window Activity Options (Disabled)");
                        }
                    });
                });
            },
        );
        if app.pending_scroll_to == Some(egui::Id::new("window_activity_options")) {
            scroll_to_rect = Some(window_activity_response.response.rect);
        }
        ui.separator();

        // Time Options
        let time_response = ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), 0.0),
            egui::Layout::top_down(Align::LEFT),
            |ui| {
                ui.group(|ui| {
                    ui.push_id(egui::Id::new("time_options"), |ui| {
                        ui.horizontal(|ui| {
                            let response = ui.checkbox(&mut app.time_options.config.enabled, "");
                            if response.changed() {
                                debug!("Time options enabled checkbox changed");
                                app.config_changed = true;
                            }
                            ui.heading("Time Options");
                        });
                        if app.time_options.config.enabled {
                            let response = app.time_options.show_time_options(ui);
                            if response.changed() {
                                app.config_changed = true;
                            }
                        }
                    });
                });
            },
        );
        if app.pending_scroll_to == Some(egui::Id::new("time_options")) {
            scroll_to_rect = Some(time_response.response.rect);
        }
        ui.separator();

        // Component Stats Options
        let component_response = ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), 0.0),
            egui::Layout::top_down(Align::LEFT),
            |ui| {
                ui.group(|ui| {
                    ui.push_id(egui::Id::new("component_stats_options"), |ui| {
                        ui.horizontal(|ui| {
                            let response = ui.checkbox(&mut app.component_stats.enabled, "");
                            if response.changed() {
                                debug!("Component stats enabled checkbox changed");
                                app.config_changed = true;
                            }
                            ui.heading("Component Stats Options");
                        });
                        if app.component_stats.enabled {
                            let response = app.component_stats.show_component_stats_options(ui);
                            if response.changed() {
                                app.config_changed = true;
                            }
                        }
                    });
                });
            },
        );
        if app.pending_scroll_to == Some(egui::Id::new("component_stats_options")) {
            scroll_to_rect = Some(component_response.response.rect);
        }
        ui.separator();

        // Network Stats Options
        let network_response = ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), 0.0),
            egui::Layout::top_down(Align::LEFT),
            |ui| {
                ui.group(|ui| {
                    ui.push_id(egui::Id::new("network_stats_options"), |ui| {
                        ui.horizontal(|ui| {
                            let response = ui.checkbox(&mut app.network_stats.enabled, "");
                            if response.changed() {
                                debug!("Network stats enabled checkbox changed");
                                app.config_changed = true;
                            }
                            ui.heading("Network Stats Options");
                        });
                        if app.network_stats.enabled {
                            let response = app.network_stats.show_network_stats_options(ui);
                            if response.changed() {
                                app.config_changed = true;
                            }
                        }
                    });
                });
            },
        );
        if app.pending_scroll_to == Some(egui::Id::new("network_stats_options")) {
            scroll_to_rect = Some(network_response.response.rect);
        }
        ui.separator();

        // Chatting Options
        let chatting_response = ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), 0.0),
            egui::Layout::top_down(Align::LEFT),
            |ui| {
                ui.group(|ui| {
                    ui.push_id(egui::Id::new("chatting_options"), |ui| {
                        ui.horizontal(|ui| {
                            let response = ui.checkbox(&mut app.chat_options.enabled, "");
                            if response.changed() {
                                debug!("Chatting options enabled checkbox changed");
                                app.config_changed = true;
                            }
                            ui.heading("Chatting Options");
                        });
                        if app.chat_options.enabled {
                            let response = app.chat_options.show_chatting_options(ui);
                            if response.changed() {
                                app.config_changed = true;
                            }
                        }
                    });
                });
            },
        );
        if app.pending_scroll_to == Some(egui::Id::new("chatting_options")) {
            scroll_to_rect = Some(chatting_response.response.rect);
        }
        ui.separator();

        // MediaLink Options
        let medialink_response = ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), 0.0),
            egui::Layout::top_down(Align::LEFT),
            |ui| {
                ui.group(|ui| {
                    ui.push_id(egui::Id::new("medialink_options"), |ui| {
                        ui.horizontal(|ui| {
                            let response = ui.checkbox(&mut app.media_link.enabled, "");
                            if response.changed() {
                                debug!("MediaLink options enabled checkbox changed");
                                app.config_changed = true;
                            }
                            ui.heading("MediaLink Options");
                        });
                        if app.media_link.enabled {
                            let response = app.media_link.show_medialink_options(ui);
                            if response.changed() {
                                app.config_changed = true;
                            }
                        }
                    });
                });
            },
        );
        if app.pending_scroll_to == Some(egui::Id::new("medialink_options")) {
            scroll_to_rect = Some(medialink_response.response.rect);
        }
        ui.separator();

        // App Options
        let app_response = ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), 0.0),
            egui::Layout::top_down(Align::LEFT),
            |ui| {
                ui.group(|ui| {
                    ui.push_id(egui::Id::new("app_options"), |ui| {
                        ui.horizontal(|ui| {
                            let response = ui.checkbox(&mut app.app_options.enabled, "");
                            if response.changed() {
                                debug!("App options enabled checkbox changed");
                                app.config_changed = true;
                            }
                            ui.heading("App Options");
                        });
                        if app.app_options.enabled {
                            let response = app.app_options.show_app_options(ui);
                            if response.changed() {
                                app.config_changed = true;
                            }
                        }
                    });
                });
            },
        );
        if app.pending_scroll_to == Some(egui::Id::new("app_options")) {
            scroll_to_rect = Some(app_response.response.rect);
        }
        ui.separator();

        // Extra Options
        let extra_response = ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), 0.0),
            egui::Layout::top_down(Align::LEFT),
            |ui| {
                ui.group(|ui| {
                    ui.push_id(egui::Id::new("extra_options"), |ui| {
                        ui.horizontal(|ui| {
                            let response = ui.checkbox(&mut app.extra_options.enabled, "");
                            if response.changed() {
                                debug!("Extra options enabled checkbox changed");
                                app.config_changed = true;
                            }
                            ui.heading("Extra Options");
                        });
                        if app.extra_options.enabled {
                            let response = app.extra_options.show_extra_options(ui);
                            if response.changed() {
                                app.config_changed = true;
                            }
                        }
                    });
                });
            },
        );
        if app.pending_scroll_to == Some(egui::Id::new("extra_options")) {
            scroll_to_rect = Some(extra_response.response.rect);
        }

        // Perform scroll after rendering all sections
        if let Some(rect) = scroll_to_rect {
            ui.scroll_to_rect(rect, Some(Align::TOP));
            app.pending_scroll_to = None;
        }
    });
}