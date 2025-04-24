use eframe::egui::{self, Ui, Color32};
use log::debug;
use crate::ui::{toggle::toggle_switch, App};

pub fn show_integrations_tab(ui: &mut Ui, app: &mut App) {
    ui.heading("Integrations");

    // Personal Status
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.heading("Personal Status");
            if ui.button("⚙").clicked() {
                debug!("Personal Status settings button clicked");
                app.current_tab = crate::ui::types::Tab::Options;
                app.pending_scroll_to = Some(egui::Id::new("status_options"));
                app.status_options.enabled = true;
                app.config_changed = true;
                ui.ctx().request_repaint();
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let response = toggle_switch(ui, &mut app.integrations_tab.personal_status_enabled, "personal_status_toggle");
                if response.changed() {
                    app.config_changed = true;
                }
            });
        });
        ui.label(egui::RichText::new("Manage your personal status messages.").color(Color32::from_rgb(0x3f, 0x3f, 0x3f)));
    });
    ui.separator();

    // Window Activity
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.heading("Window Activity");
            if ui.button("⚙").clicked() {
                debug!("Window Activity settings button clicked");
                app.current_tab = crate::ui::types::Tab::Options;
                app.pending_scroll_to = Some(egui::Id::new("window_activity_options"));
                app.window_activity.enabled = true;
                app.integrations_tab.window_activity_enabled = true;
                app.config_changed = true;
                ui.ctx().request_repaint();
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let response = toggle_switch(ui, &mut app.integrations_tab.window_activity_enabled, "window_activity_toggle");
                if response.changed() {
                    app.window_activity.enabled = app.integrations_tab.window_activity_enabled;
                    app.config_changed = true;
                }
            });
        });
        ui.label(egui::RichText::new("Show the currently focused desktop or VR application.").color(Color32::from_rgb(0x3f, 0x3f, 0x3f)));
        /*if app.integrations_tab.window_activity_enabled {
            ui.label(format!("VR Active: {}", if app.window_activity_module.is_vr_active() { "Yes" } else { "No" }));
            if let Some(activity) = &app.cached_activity {
                ui.label(format!("Activity: {}", activity));
            } else {
                ui.label("No activity detected.");
                let current_title = app.window_activity_module.get_current_title();
                ui.label(egui::RichText::new(format!("Debug: Current title = {}", current_title)).color(Color32::from_rgb(0xff, 0x3f, 0x00)));
                if current_title.contains("No window detection") {
                    ui.label(egui::RichText::new("Window detection failed: KWin does not support required protocols").color(Color32::RED));
                }
            }
        } else {
            ui.label(egui::RichText::new("Window Activity Disabled").color(Color32::from_rgb(0xff, 0x3f, 0x3f)));
        }*/
    });
    ui.separator();

    // Component Stats
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.heading("Component Stats");
            if ui.button("⚙").clicked() {
                debug!("Component Stats settings button clicked");
                app.current_tab = crate::ui::types::Tab::Options;
                app.pending_scroll_to = Some(egui::Id::new("component_stats_options"));
                app.component_stats.enabled = true;
                app.config_changed = true;
                ui.ctx().request_repaint();
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let response = toggle_switch(ui, &mut app.integrations_tab.component_stats_enabled, "component_stats_toggle");
                if response.changed() {
                    app.config_changed = true;
                }
            });
        });
        ui.label(egui::RichText::new("View system performance stats (CPU, GPU, RAM, etc.).").color(Color32::from_rgb(0x3f, 0x3f, 0x3f)));
        if app.integrations_tab.component_stats_enabled {
            ui.horizontal(|ui| {
                if ui.toggle_value(&mut app.component_stats.show_cpu, "CPU").changed() {
                    debug!("CPU toggle changed");
                    app.config_changed = true;
                }
                if ui.toggle_value(&mut app.component_stats.show_gpu, "GPU").changed() {
                    debug!("GPU toggle changed");
                    app.config_changed = true;
                }
                if ui.toggle_value(&mut app.component_stats.show_vram, "VRAM").changed() {
                    debug!("VRAM toggle changed");
                    app.config_changed = true;
                }
                if ui.toggle_value(&mut app.component_stats.show_ram, "RAM").changed() {
                    debug!("RAM toggle changed");
                    app.config_changed = true;
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
                debug!("Network Stats settings button clicked");
                app.current_tab = crate::ui::types::Tab::Options;
                app.pending_scroll_to = Some(egui::Id::new("network_stats_options"));
                app.network_stats.enabled = true;
                app.config_changed = true;
                ui.ctx().request_repaint();
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let response = toggle_switch(ui, &mut app.integrations_tab.network_stats_enabled, "network_stats_toggle");
                if response.changed() {
                    app.config_changed = true;
                }
            });
        });
        ui.label(egui::RichText::new("Monitor real-time network statistics (download/upload speeds).").color(Color32::from_rgb(0x3f, 0x3f, 0x3f)));
        if app.integrations_tab.network_stats_enabled {
            let interfaces = crate::modules::network::NetworkStats::get_interfaces();
            if let Some(iface) = interfaces.first() {
                let stats = crate::modules::network::NetworkStats::get_formatted_stats(&app.network_stats.config, &iface.name);
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
                debug!("Current Time settings button clicked");
                app.current_tab = crate::ui::types::Tab::Options;
                app.pending_scroll_to = Some(egui::Id::new("time_options"));
                app.time_options.config.enabled = true;
                app.config_changed = true;
                ui.ctx().request_repaint();
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let response = toggle_switch(ui, &mut app.integrations_tab.current_time_enabled, "current_time_toggle");
                if response.changed() {
                    app.config_changed = true;
                }
            });
        });
        ui.label(egui::RichText::new("Display the current local time and other time zones.").color(Color32::from_rgb(0x3f, 0x3f, 0x3f)));
    });
    ui.separator();

    // MediaLink
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.heading("MediaLink");
            if ui.button("⚙").clicked() {
                debug!("MediaLink settings button clicked");
                app.current_tab = crate::ui::types::Tab::Options;
                app.pending_scroll_to = Some(egui::Id::new("medialink_options"));
                app.media_link.enabled = true;
                app.config_changed = true;
                ui.ctx().request_repaint();
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let response = toggle_switch(ui, &mut app.integrations_tab.medialink_enabled, "medialink_toggle");
                if response.changed() {
                    app.config_changed = true;
                }
            });
        });
        ui.label(egui::RichText::new("Show the current media track and artist.").color(Color32::from_rgb(0x3f, 0x3f, 0x3f)));
        if app.integrations_tab.medialink_enabled {
            if let Some(track) = app.media_module.get_formatted_track(&app.media_link) {
                ui.label(format!("Now playing: {}", track));
            } else {
                ui.label("No media playing.");
            }
            ui.horizontal(|ui| {
                ui.label("Progress");
                let duration = app.media_module.get_duration().unwrap_or(1.0);
                let mut position = app.media_module.get_position().unwrap_or(0.0);
                let response = ui.add(
                    egui::Slider::new(&mut position, 0.0..=duration)
                        .show_value(false)
                        .text(""),
                );
                if response.changed() {
                    debug!("Media seek slider changed to position {}", position);
                    app.media_module.seek(position);
                }
                if ui.button("⏮").clicked() {
                    debug!("Media previous button clicked");
                    app.media_module.previous();
                }
                let is_playing = app.media_module.is_playing();
                let play_pause_icon = if is_playing { "⏸" } else { "▶" };
                if ui.button(play_pause_icon).clicked() {
                    debug!("Media play/pause button clicked, is_playing={}", is_playing);
                    app.media_module.play_pause();
                }
                if ui.button("⏭").clicked() {
                    debug!("Media next button clicked");
                    app.media_module.next();
                }
            });
        }
    });
}