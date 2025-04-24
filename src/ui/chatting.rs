use eframe::egui::{self, Ui, Align};
use log::{debug, info, error};
use crate::ui::App;

pub fn show_chatting_tab(ui: &mut Ui, app: &mut App) {
    // Input section pinned to the bottom
    egui::TopBottomPanel::bottom("chat_input").show(ui.ctx(), |ui| {
        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let placeholder = if app.chat_tab.is_focused || !app.chat_tab.message.is_empty() {
                    ""
                } else {
                    "Send a chat message"
                };
                let response = ui.add(
                    egui::TextEdit::singleline(&mut app.chat_tab.message)
                        .desired_width(200.0)
                        .hint_text(placeholder),
                );
                if response.changed() {
                    debug!("Chat message input changed");
                    app.config_changed = true;
                }
                if response.has_focus() != app.chat_tab.is_focused {
                    debug!("Chat input focus changed to {}", response.has_focus());
                    app.chat_tab.is_focused = response.has_focus();
                    app.config_changed = true;
                }
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && !app.chat_tab.message.is_empty() && app.chat_tab.message.len() <= 140 {
                    let message = app.chat_tab.message.clone();
                    debug!("Enter key pressed to send chat message: {}", message);
                    if app.chat_options.can_send() && app.last_osc_send.elapsed().as_secs_f32() >= app.app_options.app_options.osc_options.update_rate {
                        let formatted_message = if app.chat_options.add_speech_bubble {
                            format!("🗨 {}", message)
                        } else {
                            message.clone()
                        };
                        app.osc_preview = formatted_message.clone();
                        if app.send_to_vrchat {
                            if let Err(e) = app.osc_client.send_chatbox_message(&formatted_message, app.chat_options.play_fx_sound, app.extra_options.slim_mode) {
                                error!("Failed to send OSC chat message: {}", e);
                            } else {
                                info!("Sent chat message to OSC: {}", formatted_message);
                            }
                            app.last_osc_send = std::time::Instant::now();
                        }
                        app.chat_options.add_message(message);
                        app.chat_tab.message.clear();
                        app.chat_tab.is_focused = false;
                        app.config_changed = true;
                    } else {
                        debug!("Queuing chat message: {}", message);
                        app.chat_options.set_queued_message(message);
                        app.chat_tab.message.clear();
                        app.chat_tab.is_focused = false;
                        app.config_changed = true;
                    }
                }
                ui.label(format!("{}/140", app.chat_tab.message.len()));
                if ui.button("Paste").clicked() {
                    debug!("Paste button clicked");
                    if let Ok(text) = app.clipboard.get_text() {
                        app.chat_tab.message = text.chars().take(140).collect();
                        app.chat_tab.is_focused = true;
                        app.config_changed = true;
                        info!("Pasted text into chat input");
                    } else {
                        error!("Failed to paste from clipboard");
                    }
                }
                if ui.button("Send").clicked() && !app.chat_tab.message.is_empty() && app.chat_tab.message.len() <= 140 {
                    let message = app.chat_tab.message.clone();
                    debug!("Send button clicked for chat message: {}", message);
                    if app.chat_options.can_send() && app.last_osc_send.elapsed().as_secs_f32() >= app.app_options.app_options.osc_options.update_rate {
                        let formatted_message = if app.chat_options.add_speech_bubble {
                            format!("🗨 {}", message)
                        } else {
                            message.clone()
                        };
                        app.osc_preview = formatted_message.clone();
                        if app.send_to_vrchat {
                            if let Err(e) = app.osc_client.send_chatbox_message(&formatted_message, app.chat_options.play_fx_sound, app.extra_options.slim_mode) {
                                error!("Failed to send OSC chat message: {}", e);
                            } else {
                                info!("Sent chat message to OSC: {}", formatted_message);
                            }
                            app.last_osc_send = std::time::Instant::now();
                        }
                        app.chat_options.add_message(message);
                        app.chat_tab.message.clear();
                        app.chat_tab.is_focused = false;
                        app.config_changed = true;
                    } else {
                        debug!("Queuing chat message: {}", message);
                        app.chat_options.set_queued_message(message);
                        app.chat_tab.message.clear();
                        app.chat_tab.is_focused = false;
                        app.config_changed = true;
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
                for message in app.chat_options.messages.iter() {
                    if ((now_ms - message.sent_at_ms) / 1000) < app.chat_options.chat_timeout as u64 {
                        remaining_times.push(app.chat_options.get_remaining_time(message));
                    } else {
                        remaining_times.push(0);
                    }
                }
                let mut index = 0;
                for (_index, message) in app.chat_options.messages.iter_mut().enumerate().rev() {
                    if ((now_ms - message.sent_at_ms) / 1000) < app.chat_options.chat_timeout as u64 {
                        let remaining_time = remaining_times[index];
                        ui.horizontal(|ui| {
                            if message.editing {
                                let response = ui.add(
                                    egui::TextEdit::singleline(&mut message.edit_text)
                                        .desired_width(200.0)
                                        .hint_text("Edit message"),
                                );
                                if response.changed() {
                                    debug!("Editing chat message");
                                    app.config_changed = true;
                                }
                                if !app.chat_options.live_editing && response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                    message.text = message.edit_text.clone();
                                    message.sent_at_ms = now_ms;
                                    let formatted_message = if app.chat_options.add_speech_bubble {
                                        format!("🗨 {}", message.text)
                                    } else {
                                        message.text.clone()
                                    };
                                    app.osc_preview = formatted_message.clone();
                                    if app.send_to_vrchat && app.last_osc_send.elapsed().as_secs_f32() >= app.app_options.app_options.osc_options.update_rate {
                                        if let Err(e) = app.osc_client.send_chatbox_message(&formatted_message, app.chat_options.play_fx_sound, app.extra_options.slim_mode) {
                                            error!("Failed to send edited OSC message: {}", e);
                                        } else {
                                            info!("Sent edited chat message to OSC: {}", formatted_message);
                                        }
                                        app.last_osc_send = std::time::Instant::now();
                                    }
                                    message.editing = false;
                                    app.config_changed = true;
                                    info!("Finished editing chat message");
                                }
                                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                                    if app.chat_options.live_editing && ui.button("X").clicked() {
                                        debug!("Cancel edit button clicked");
                                        message.editing = false;
                                        message.edit_text = message.text.clone();
                                        app.osc_preview = app.previous_osc_preview.clone();
                                        if app.send_to_vrchat && !app.osc_preview.is_empty() && app.last_osc_send.elapsed().as_secs_f32() >= app.app_options.app_options.osc_options.update_rate {
                                            if let Err(e) = app.osc_client.send_chatbox_message(&app.osc_preview, false, app.extra_options.slim_mode) {
                                                error!("Failed to send OSC message after cancel: {}", e);
                                            } else {
                                                info!("Sent previous OSC message after cancel: {}", app.osc_preview);
                                            }
                                            app.last_osc_send = std::time::Instant::now();
                                        }
                                        app.config_changed = true;
                                    }
                                    let edit_label = if app.chat_options.live_editing {
                                        format!("Live Edit ({})", remaining_time)
                                    } else {
                                        format!("Edit ({})", remaining_time)
                                    };
                                    if ui.button(&edit_label).clicked() {
                                        debug!("Edit button clicked for message");
                                        message.editing = !message.editing;
                                        if !message.editing {
                                            message.edit_text = message.text.clone();
                                            app.osc_preview = app.previous_osc_preview.clone();
                                            if app.send_to_vrchat && !app.osc_preview.is_empty() && app.last_osc_send.elapsed().as_secs_f32() >= app.app_options.app_options.osc_options.update_rate {
                                                if let Err(e) = app.osc_client.send_chatbox_message(&app.osc_preview, false, app.extra_options.slim_mode) {
                                                    error!("Failed to send OSC message after edit toggle: {}", e);
                                                } else {
                                                    info!("Sent previous OSC message after edit toggle: {}", app.osc_preview);
                                                }
                                                app.last_osc_send = std::time::Instant::now();
                                            }
                                        }
                                        app.config_changed = true;
                                    }
                                    if ui.button("Copy").clicked() {
                                        debug!("Copy button clicked for message");
                                        if let Err(e) = app.clipboard.set_text(&message.text) {
                                            error!("Failed to copy message to clipboard: {}", e);
                                        } else {
                                            info!("Copied message to clipboard: {}", message.text);
                                        }
                                    }
                                    if ui.button("Resend").clicked() {
                                        debug!("Resend button clicked for message");
                                        if app.last_osc_send.elapsed().as_secs_f32() >= app.app_options.app_options.osc_options.update_rate {
                                            let formatted_message = if app.chat_options.add_speech_bubble {
                                                format!("🗨 {}", message.text)
                                            } else {
                                                message.text.clone()
                                            };
                                            message.sent_at_ms = now_ms;
                                            app.osc_preview = formatted_message.clone();
                                            if app.send_to_vrchat {
                                                if let Err(e) = app.osc_client.send_chatbox_message(&formatted_message, app.chat_options.play_fx_resend && app.chat_options.play_fx_sound, app.extra_options.slim_mode) {
                                                    error!("Failed to resend OSC message: {}", e);
                                                } else {
                                                    info!("Resent chat message to OSC: {}", formatted_message);
                                                }
                                                app.last_osc_send = std::time::Instant::now();
                                            }
                                            app.config_changed = true;
                                        }
                                    }
                                });
                            } else {
                                ui.label(&message.text);
                                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                                    if app.chat_options.edit_messages {
                                        let edit_label = if app.chat_options.live_editing && app.live_edit_enabled {
                                            format!("Live Edit ({})", remaining_time)
                                        } else {
                                            format!("Edit ({})", remaining_time)
                                        };
                                        if ui.button(&edit_label).clicked() {
                                            debug!("Edit button clicked for message");
                                            message.editing = true;
                                            app.config_changed = true;
                                        }
                                    }
                                    if ui.button("Copy").clicked() {
                                        debug!("Copy button clicked for message");
                                        if let Err(e) = app.clipboard.set_text(&message.text) {
                                            error!("Failed to copy message to clipboard: {}", e);
                                        } else {
                                            info!("Copied message to clipboard: {}", message.text);
                                        }
                                    }
                                    if ui.button("Resend").clicked() {
                                        debug!("Resend button clicked for message");
                                        if app.last_osc_send.elapsed().as_secs_f32() >= app.app_options.app_options.osc_options.update_rate {
                                            let formatted_message = if app.chat_options.add_speech_bubble {
                                                format!("🗨 {}", message.text)
                                            } else {
                                                message.text.clone()
                                            };
                                            message.sent_at_ms = now_ms;
                                            app.osc_preview = formatted_message.clone();
                                            if app.send_to_vrchat {
                                                if let Err(e) = app.osc_client.send_chatbox_message(&formatted_message, app.chat_options.play_fx_resend && app.chat_options.play_fx_sound, app.extra_options.slim_mode) {
                                                    error!("Failed to resend OSC message: {}", e);
                                                } else {
                                                    info!("Resent chat message to OSC: {}", formatted_message);
                                                }
                                                app.last_osc_send = std::time::Instant::now();
                                            }
                                            app.config_changed = true;
                                        }
                                    }
                                });
                            }
                        });
                        ui.separator();
                        index += 1;
                    }
                }
                // Control buttons (Stop, Clear history) only when messages exist
                if !app.chat_options.messages.is_empty() {
                    ui.group(|ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                                if ui.button("Clear history").clicked() {
                                    debug!("Clear history button clicked");
                                    app.chat_options.clear_messages();
                                    app.config_changed = true;
                                    info!("Cleared chat message history");
                                }
                                if ui.button("Stop").clicked() {
                                    debug!("Stop button clicked");
                                    app.osc_preview = app.previous_osc_preview.clone();
                                    if app.send_to_vrchat && !app.osc_preview.is_empty() && app.last_osc_send.elapsed().as_secs_f32() >= app.app_options.app_options.osc_options.update_rate {
                                        if let Err(e) = app.osc_client.send_chatbox_message(&app.osc_preview, false, app.extra_options.slim_mode) {
                                            error!("Failed to send OSC message after stop: {}", e);
                                        } else {
                                            info!("Sent previous OSC message after stop: {}", app.osc_preview);
                                        }
                                        app.last_osc_send = std::time::Instant::now();
                                    }
                                }
                            });
                        });
                    });
                }
            }); // Close ui.group
        }); // Close ScrollArea
    }); // Close CentralPanel
}