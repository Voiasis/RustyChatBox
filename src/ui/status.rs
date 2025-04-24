use eframe::egui::Ui;
use log::{debug, info, error};
use crate::ui::App;

pub fn show_status_tab(ui: &mut Ui, app: &mut App) {
    ui.heading("Status");
    ui.label("Manage personal status messages.");
    ui.horizontal(|ui| {
        ui.label("New status: ");
        if ui.text_edit_singleline(&mut app.status_tab.new_message).changed() {
            debug!("New status message edited");
            app.config_changed = true;
        }
        if ui.button("Add").clicked() && !app.status_tab.new_message.is_empty() {
            info!("Adding new status message: {}", app.status_tab.new_message);
            app.status_module.add_message(app.status_tab.new_message.clone());
            app.status_tab.new_message.clear();
            app.config_changed = true;
        }
    });
    if let Some(current) = app.status_module.get_current_message(&app.status_options) {
        ui.label(format!("Current status: {}", current));
        if ui.button("Send to OSC").clicked() {
            debug!("Send to OSC button clicked for status: {}", current);
            if app.last_osc_send.elapsed().as_secs_f32() >= app.app_options.app_options.osc_options.update_rate {
                if let Err(e) = app.osc_client.send_chatbox_message(current.as_str(), false, app.extra_options.slim_mode) {
                    error!("Failed to send status to OSC: {}", e);
                } else {
                    info!("Sent status to OSC: {}", current);
                }
                app.last_osc_send = std::time::Instant::now();
            }
        }
    } else {
        ui.label("No status messages set.");
    }
    ui.label("Stored messages:");
    let mut to_remove = None;
    for (i, msg) in app.status_module.messages.iter().enumerate() {
        ui.horizontal(|ui| {
            ui.label(msg);
            if ui.button("Remove").clicked() {
                debug!("Remove status message button clicked for: {}", msg);
                to_remove = Some(i);
                app.config_changed = true;
            }
        });
    }
    if let Some(index) = to_remove {
        info!("Removing status message at index {}", index);
        app.status_module.remove_message(index);
        app.config_changed = true;
    }
}