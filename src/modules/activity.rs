use serde::{Deserialize, Serialize};
use wayland_client::{Connection, QueueHandle};
use wayland_client::protocol::wl_registry::WlRegistry;
use wayland_protocols_wlr::foreign_toplevel::v1::client::{
    zwlr_foreign_toplevel_handle_v1::{self, ZwlrForeignToplevelHandleV1},
    zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1,
};
use std::sync::{Arc, Mutex};
use eframe::egui;
use log::{debug, error, info};
use std::process::Command;
use openxr as xr;
extern crate ini;
use ini::Ini;

const MAX_LINE_WIDTH: usize = 27;

#[derive(Clone, Serialize, Deserialize)]
pub struct WindowActivityOptions {
    pub enabled: bool,
    pub max_title_length: u32,
    pub show_desktop_app: bool,
    pub desktop_prefix: String,
    pub desktop_middle: String,
    pub desktop_suffix: String,
    pub show_vr_app: bool,
    pub vr_prefix: String,
    pub vr_middle: String,
    pub vr_suffix: String,
}

impl Default for WindowActivityOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            max_title_length: 50,
            show_desktop_app: true,
            desktop_prefix: "On desktop".to_string(),
            desktop_middle: "in".to_string(),
            desktop_suffix: "%app%".to_string(),
            show_vr_app: true,
            vr_prefix: "In VR".to_string(),
            vr_middle: "focusing in".to_string(),
            vr_suffix: "%app%".to_string(),
        }
    }
}

impl WindowActivityOptions {
    pub fn show_window_activity_options(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let mut response = ui.label("Debug: Rendering Window Activity Options");

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Max title length");
                let r = ui.add(egui::DragValue::new(&mut self.max_title_length).range(1..=100));
                response |= r;
            });

            ui.heading("Desktop Activity");
            let r = ui.checkbox(&mut self.show_desktop_app, "Show desktop focused app details");
            response |= r;

            ui.horizontal(|ui| {
                ui.label("Prefix: ");
                let r = ui.add(
                    egui::TextEdit::singleline(&mut self.desktop_prefix).desired_width(100.0)
                );
                response |= r;
                if self.show_desktop_app {
                    ui.label("Middle: ");
                    let r = ui.add(
                        egui::TextEdit::singleline(&mut self.desktop_middle).desired_width(100.0)
                    );
                    response |= r;
                    ui.label("Suffix: ");
                    let r = ui.add(
                        egui::TextEdit::singleline(&mut self.desktop_suffix).desired_width(100.0)
                    );
                    response |= r;
                }
            });

            ui.heading("VR Activity");
            let r = ui.checkbox(&mut self.show_vr_app, "Show VR focused app details");
            response |= r;

            ui.horizontal(|ui| {
                ui.label("Prefix: ");
                let r = ui.add(
                    egui::TextEdit::singleline(&mut self.vr_prefix).desired_width(100.0)
                );
                response |= r;
                if self.show_vr_app {
                    ui.label("Middle: ");
                    let r = ui.add(
                        egui::TextEdit::singleline(&mut self.vr_middle).desired_width(100.0)
                    );
                    response |= r;
                    ui.label("Suffix: ");
                    let r = ui.add(
                        egui::TextEdit::singleline(&mut self.vr_suffix).desired_width(100.0)
                    );
                    response |= r;
                }
            });
        });

        response
    }
}

#[allow(dead_code)]
pub struct WindowActivityModule {
    wayland_conn: Option<Connection>,
    x11_worker: Option<std::thread::JoinHandle<()>>,
    is_vr_active: Arc<Mutex<bool>>,
    current_title: Arc<Mutex<String>>,
}

impl Drop for WindowActivityModule {
    fn drop(&mut self) {
        if let Some(worker) = self.x11_worker.take() {
            debug!("Joining X11 worker thread");
            worker.join().expect("Failed to join X11 worker thread");
        }
    }
}

impl WindowActivityModule {
    /*pub fn is_vr_active(&self) -> bool {
        *self.is_vr_active.lock().unwrap()
    }*/

    pub fn new(_options: &WindowActivityOptions) -> Self {
        let wayland_conn = Connection::connect_to_env().ok();
        let current_title = Arc::new(Mutex::new("Unknown".to_string()));
        let is_vr_active = Arc::new(Mutex::new(false));

        let mut module = Self {
            wayland_conn,
            x11_worker: None,
            is_vr_active,
            current_title,
        };

        module.start_window_detection();
        debug!("Starting VR detection");
        // Delay to ensure Envision/Monado is ready
        std::thread::sleep(std::time::Duration::from_millis(2000));
        module.start_vr_detection();

        module
    }

    fn start_vr_detection(&self) {
        let is_vr_active = Arc::clone(&self.is_vr_active);
        debug!("Starting VR detection thread");
        std::thread::spawn(move || {
            // Cache OpenXR instance
            let mut openxr_error_logged = false;
            let entry = unsafe { xr::Entry::load() }.map_err(|e| {
                if !openxr_error_logged {
                    error!("Failed to load OpenXR: {:?}", e);
                    openxr_error_logged = true;
                }
                format!("Load error: {:?}", e)
            });
            let instance = entry.and_then(|e| {
                let mut exts = xr::ExtensionSet::default();
                exts.khr_composition_layer_depth = true; // Optional for Monado
                e.create_instance(
                    &xr::ApplicationInfo {
                        application_name: "RustyChatBox",
                        application_version: 1,
                        engine_name: "None",
                        engine_version: 0,
                    },
                    &exts,
                    &[],
                )
                .map_err(|e| {
                    if !openxr_error_logged {
                        error!("Failed to create OpenXR instance: {:?}", e);
                        openxr_error_logged = true;
                    }
                    format!("Instance creation failed: {:?}", e)
                })
            });
            let system = instance.as_ref().map_err(|e| e.clone()).and_then(|i: &xr::Instance| {
                i.system(xr::FormFactor::HEAD_MOUNTED_DISPLAY)
                    .map_err(|e| {
                        if !openxr_error_logged {
                            error!("Failed to get OpenXR system: {:?}", e);
                            openxr_error_logged = true;
                        }
                        format!("System retrieval failed: {:?}", e)
                    })
            });
    
            // Log OpenXR runtime path once
            match std::env::var("XR_RUNTIME_JSON") {
                Ok(path) => debug!("XR_RUNTIME_JSON: {}", path),
                Err(_) => error!("XR_RUNTIME_JSON not set, OpenXR may fail"),
            }
    
            let mut last_vr_state = false;
            let mut last_error_log_time = std::time::Instant::now();
            let force_vr = std::env::var("FORCE_VR_ACTIVE").is_ok();
    
            loop {
                // Check for VR headset state
                let (new_title, new_vr_state) = if force_vr {
                    debug!("Forced VR active due to FORCE_VR_ACTIVE env");
                    ("Forced VR".to_string(), true)
                } else if system.is_ok() {
                    // Verify OpenXR session is actually active
                    match instance.as_ref().map(|i| {
                        i.system(xr::FormFactor::HEAD_MOUNTED_DISPLAY)
                            .map(|_| ("VR Session".to_string(), true))
                    }) {
                        Ok(Ok((title, active))) => {
                            debug!("OpenXR session active");
                            (title, active)
                        }
                        _ => {
                            debug!("No active OpenXR session, checking processes");
                            // Check VR processes
                            let processes = [
                                "monado",
                                "monado-service",
                                "monado-comp",
                                "monado-cli",
                                "monado-gui",
                                "steamvr",
                                "vrmonitor",
                                "vrcompositor",
                                "vrserver",
                                "vrdashboard",
                                "openvr",
                                "xrcompositor",
                            ];
                            let mut vr_active = false;
                            let mut title = "No VR".to_string();
                            for proc in processes {
                                let output = Command::new("pgrep")
                                    .arg("-l")
                                    .arg(proc)
                                    .output();
                                match output {
                                    Ok(output) if output.status.success() => {
                                        let output_str = String::from_utf8_lossy(&output.stdout);
                                        debug!("pgrep found {}: {}", proc, output_str);
                                        vr_active = true;
                                        title = proc.to_string();
                                        break;
                                    }
                                    Ok(_) => {
                                        debug!("pgrep did not find {}", proc);
                                    }
                                    Err(e) => {
                                        if last_error_log_time.elapsed().as_secs() >= 60 {
                                            error!("pgrep failed for {}: {}", proc, e);
                                            last_error_log_time = std::time::Instant::now();
                                        }
                                    }
                                }
                            }
                            (title, vr_active)
                        }
                    }
                } else {
                    if last_error_log_time.elapsed().as_secs() >= 60 {
                        error!("OpenXR unavailable: {}, checking processes", system.as_ref().unwrap_err());
                        last_error_log_time = std::time::Instant::now();
                    }
                    // Check VR processes as fallback
                    let processes = [
                        "monado",
                        "monado-service",
                        "monado-comp",
                        "monado-cli",
                        "monado-gui",
                        "steamvr",
                        "vrmonitor",
                        "vrcompositor",
                        "vrserver",
                        "vrdashboard",
                        "openvr",
                        "xrcompositor",
                    ];
                    let mut vr_active = false;
                    let mut title = "No VR".to_string();
                    for proc in processes {
                        let output = Command::new("pgrep")
                            .arg("-l")
                            .arg(proc)
                            .output();
                        match output {
                            Ok(output) if output.status.success() => {
                                let output_str = String::from_utf8_lossy(&output.stdout);
                                debug!("pgrep found {}: {}", proc, output_str);
                                vr_active = true;
                                title = proc.to_string();
                                break;
                            }
                            Ok(_) => {
                                debug!("pgrep did not find {}", proc);
                            }
                            Err(e) => {
                                if last_error_log_time.elapsed().as_secs() >= 60 {
                                    error!("pgrep failed for {}: {}", proc, e);
                                    last_error_log_time = std::time::Instant::now();
                                }
                            }
                        }
                    }
                    (title, vr_active)
                };
    
                // Update state only if it has changed
                if new_vr_state != last_vr_state {
                    info!("VR state changed: active={}, title={}", new_vr_state, new_title);
                    let mut vr_active_lock = is_vr_active.lock().unwrap();
                    *vr_active_lock = new_vr_state;
                    last_vr_state = new_vr_state;
                } else {
                    debug!("VR state unchanged: active={}, title={}", new_vr_state, new_title);
                }
    
                std::thread::sleep(std::time::Duration::from_millis(1000));
            }
        });
    }

    fn start_window_detection(&mut self) {
        let current_title = Arc::clone(&self.current_title);
    
        if let Some(conn) = &self.wayland_conn {
            debug!("Starting Wayland window detection");
            let mut state = WaylandState {
                toplevel_manager: None,
                current_title: Arc::clone(&self.current_title),
                toplevels: Vec::new(),
                active_toplevel: None,
            };
            let mut event_queue = conn.new_event_queue();
            let qh = event_queue.handle();
            let display = conn.display();
            display.get_registry(&qh, ());
            debug!("Requesting Wayland registry");
            std::thread::spawn(move || {
                if let Err(e) = event_queue.roundtrip(&mut state) {
                    error!("Initial Wayland roundtrip failed: {}", e);
                    let mut title_lock = current_title.lock().unwrap();
                    *title_lock = "Wayland detection failed".to_string();
                } else {
                    debug!("Initial Wayland roundtrip completed");
                    if state.toplevel_manager.is_none() {
                        debug!("No zwlr_foreign_toplevel_manager_v1, falling back to kdotool");
                        let mut title_lock = current_title.lock().unwrap();
                        *title_lock = "".to_string();
                    }
                }
                let mut last_title = String::new();
                loop {
                    if let Err(e) = event_queue.roundtrip(&mut state) {
                        if last_title != "Wayland error" {
                            error!("Wayland roundtrip error: {}", e);
                            let mut title_lock = current_title.lock().unwrap();
                            *title_lock = "Wayland error".to_string();
                            last_title = "Wayland error".to_string();
                        }
                    }
                    if state.toplevel_manager.is_none() {
                        match get_kwin_active_application_name() {
                            Ok(title) => {
                                if title != last_title {
                                    debug!("kdotool title updated: {}", title);
                                    let mut title_lock = current_title.lock().unwrap();
                                    *title_lock = title.clone();
                                    last_title = title;
                                }
                            }
                            Err(e) => {
                                if last_title != "kdotool error" {
                                    error!("kdotool failed: {}", e);
                                    let title = get_x11_window_title()
                                        .unwrap_or_else(|| "Window detection unavailable".to_string());
                                    debug!("xdotool title updated: {}", title);
                                    let mut title_lock = current_title.lock().unwrap();
                                    *title_lock = title.clone();
                                    last_title = title;
                                }
                            }
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                }
            });
        } else {
            debug!("Starting X11 window detection (via xdotool)");
            let current_title = Arc::clone(&self.current_title);
            self.x11_worker = Some(std::thread::spawn(move || {
                let mut last_title = String::new();
                loop {
                    let title = get_x11_window_title()
                        .unwrap_or_else(|| "No active window".to_string());
                    if title != last_title {
                        debug!("xdotool title updated: {}", title);
                        let mut title_lock = current_title.lock().unwrap();
                        *title_lock = title.clone();
                        last_title = title;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                }
            }));
        }
    }

    pub fn get_formatted_activity(&self, options: &WindowActivityOptions) -> Option<String> {
        let title = self.current_title.lock().unwrap().clone();
        let is_vr_active = *self.is_vr_active.lock().unwrap();
        let (prefix, middle, suffix) = if is_vr_active {
            (&options.vr_prefix, &options.vr_middle, &options.vr_suffix)
        } else {
            (&options.desktop_prefix, &options.desktop_middle, &options.desktop_suffix)
        };
    
        if title.is_empty() || title == "No active window" || title == "No display server" || 
           title == "Window detection unavailable" || title == "Wayland detection failed" {
            return if !prefix.is_empty() {
                Some(prefix.to_string())
            } else {
                None
            };
        }
    
        let formatted_title = if title.len() > options.max_title_length as usize {
            format!("{}...", &title[..options.max_title_length as usize - 3])
        } else {
            title
        };
    
        let mut parts = Vec::new();
        if (is_vr_active && options.show_vr_app) || (!is_vr_active && options.show_desktop_app) {
            parts.push(prefix);
            parts.push(middle);
            parts.push(suffix);
        } else {
            parts.push(prefix);
        }
    
        let result = parts
            .into_iter()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .join(" ");
        
        let final_result = result.replace("%app%", &formatted_title);
    
        let mut lines = Vec::new();
        let mut current_line = String::new();
        let words = final_result.split_whitespace();
    
        for word in words {
            let word_len = word.len();
            let space_len = if current_line.is_empty() { 0 } else { 1 };
            let potential_len = current_line.len() + space_len + word_len;
    
            if potential_len > MAX_LINE_WIDTH && !current_line.is_empty() {
                lines.push(current_line);
                current_line = word.to_string();
            }
            else if word_len > MAX_LINE_WIDTH {
                if !current_line.is_empty() {
                    lines.push(current_line);
                }
                lines.push(word[..MAX_LINE_WIDTH].to_string());
                current_line = String::new();
            }
            else {
                if !current_line.is_empty() {
                    current_line.push(' ');
                }
                current_line.push_str(word);
            }
        }
    
        if !current_line.is_empty() {
            lines.push(current_line);
        }
    
        Some(lines.join("\n"))
    }

}

fn get_kwin_active_application_name() -> Result<String, String> {
    let uuid_output = Command::new("kdotool")
        .arg("getactivewindow")
        .output()
        .map_err(|e| format!("kdotool getactivewindow failed: {}", e))?;

    if !uuid_output.status.success() {
        let stderr = String::from_utf8_lossy(&uuid_output.stderr);
        return Err(format!(
            "kdotool getactivewindow failed with status {}: {}",
            uuid_output.status, stderr
        ));
    }

    let uuid = String::from_utf8(uuid_output.stdout)
        .map_err(|e| format!("kdotool getactivewindow invalid UTF-8: {}", e))?
        .trim()
        .to_string();

    if uuid.is_empty() {
        return Ok("No active window".to_string());
    }

    let class_output = Command::new("kdotool")
        .args(["getwindowclassname", &uuid])
        .output()
        .map_err(|e| format!("kdotool getwindowclassname failed: {}", e))?;

    if !class_output.status.success() {
        let stderr = String::from_utf8_lossy(&class_output.stderr);
        return Err(format!(
            "kdotool getwindowclassname failed with status {}: {}",
            class_output.status, stderr
        ));
    }

    let class_name = String::from_utf8(class_output.stdout)
        .map_err(|e| format!("kdotool getwindowclassname invalid UTF-8: {}", e))?
        .trim()
        .to_lowercase();

    if class_name.is_empty() {
        return Ok("Unknown".to_string());
    }

    let desktop_names = vec![
        class_name.to_string(),
        class_name.split('.').last().unwrap_or(&class_name).to_string(),
        class_name.replace("org.kde.", ""),
        class_name.replace("org.gnome.", ""),
    ];

    let search_paths = vec![
        "/usr/share/applications".to_string(),
        format!("{}/.local/share/applications", std::env::var("HOME").unwrap_or_default()),
    ];

    let mut last_desktop_file = String::new();
    for desktop_name in desktop_names.iter().filter(|name| !name.is_empty()) {
        for path in &search_paths {
            let desktop_file = format!("{}/{}.desktop", path, desktop_name);
            if desktop_file != last_desktop_file {
                debug!("Checking .desktop file: {}", desktop_file);
                last_desktop_file = desktop_file.clone();
            }

            if let Ok(conf) = Ini::load_from_file(&desktop_file) {
                if let Some(section) = conf.section(Some("Desktop Entry")) {
                    if let Some(name) = section.get("Name") {
                        debug!("Found application name: {} for class: {}", name, class_name);
                        return Ok(name.to_string());
                    }
                }
            }
        }
    }

    let title_output = Command::new("kdotool")
        .args(["getwindowtitle", &uuid])
        .output()
        .map_err(|e| format!("kdotool getwindowtitle failed: {}", e))?;

    if title_output.status.success() {
        let title = String::from_utf8(title_output.stdout)
            .map_err(|e| format!("kdotool getwindowtitle invalid UTF-8: {}", e))?
            .trim()
            .to_string();
        if !title.is_empty() {
            debug!("Falling back to window title: {} for class: {}", title, class_name);
            return Ok(title);
        }
    }

    if class_name.to_string() != "rustychatbox" {
        debug!("No .desktop file or title found, using class name: {}", class_name);
        Ok(class_name.to_string().chars().take(1).flat_map(|f| f.to_uppercase()).chain(class_name.chars().skip(1)).collect())
    } else {
        debug!("No .desktop file or title found, using class name: {}", class_name);
        Ok("RustyChatBox".to_string())
    }
}

fn get_x11_window_title() -> Option<String> {
    Command::new("xdotool")
        .args(["getactivewindow", "getwindowname"])
        .output()
        .map_err(|e| error!("xdotool command failed: {}", e))
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout)
                    .map_err(|e| error!("xdotool invalid UTF-8: {}", e))
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                error!("xdotool failed with status: {}", output.status);
                None
            }
        })
}

struct WaylandState {
    toplevel_manager: Option<ZwlrForeignToplevelManagerV1>,
    current_title: Arc<Mutex<String>>,
    toplevels: Vec<ZwlrForeignToplevelHandleV1>,
    active_toplevel: Option<ZwlrForeignToplevelHandleV1>,
}

impl wayland_client::Dispatch<WlRegistry, ()> for WaylandState {
    fn event(
        state: &mut Self,
        registry: &WlRegistry,
        event: wayland_client::protocol::wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wayland_client::protocol::wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            if interface == <ZwlrForeignToplevelManagerV1 as wayland_client::Proxy>::interface().name {
                debug!("Binding ZwlrForeignToplevelManagerV1, name={}, version={}", name, version);
                let manager = registry.bind::<ZwlrForeignToplevelManagerV1, _, _>(name, version, qh, ());
                state.toplevel_manager = Some(manager);
            }
        }
    }
}

impl wayland_client::Dispatch<ZwlrForeignToplevelManagerV1, ()> for WaylandState {
    fn event(
        state: &mut Self,
        _: &ZwlrForeignToplevelManagerV1,
        event: wayland_protocols_wlr::foreign_toplevel::v1::client::zwlr_foreign_toplevel_manager_v1::Event,
        _: &(),
        _: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            wayland_protocols_wlr::foreign_toplevel::v1::client::zwlr_foreign_toplevel_manager_v1::Event::Toplevel { toplevel } => {
                state.toplevels.push(toplevel);
                debug!("New toplevel added");
            }
            _ => {}
        }
    }
}

impl wayland_client::Dispatch<ZwlrForeignToplevelHandleV1, ()> for WaylandState {
    fn event(
        state: &mut Self,
        handle: &ZwlrForeignToplevelHandleV1,
        event: zwlr_foreign_toplevel_handle_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_foreign_toplevel_handle_v1::Event::Title { title } => {
                if state.active_toplevel.as_ref() == Some(handle) {
                    let new_title = if title.is_empty() { "Unknown".to_string() } else { title };
                    let mut title_lock = state.current_title.lock().unwrap();
                    if *title_lock != new_title {
                        debug!("Wayland title updated: {}", new_title);
                        *title_lock = new_title;
                    }
                }
            }
            zwlr_foreign_toplevel_handle_v1::Event::State { state: toplevel_state } => {
                let toplevel_state: Vec<_> = toplevel_state
                    .iter()
                    .filter_map(|&s| match s {
                        0 => Some(zwlr_foreign_toplevel_handle_v1::State::Maximized),
                        1 => Some(zwlr_foreign_toplevel_handle_v1::State::Minimized),
                        2 => Some(zwlr_foreign_toplevel_handle_v1::State::Activated),
                        3 => Some(zwlr_foreign_toplevel_handle_v1::State::Fullscreen),
                        _ => None
                    })
                    .collect();

                if toplevel_state.contains(&zwlr_foreign_toplevel_handle_v1::State::Activated) {
                    if state.active_toplevel.as_ref() != Some(handle) {
                        state.active_toplevel = Some(handle.clone());
                        let mut title_lock = state.current_title.lock().unwrap();
                        if *title_lock != "Unknown" {
                            debug!("Activated new toplevel: {:?}", handle);
                            *title_lock = "Unknown".to_string();
                        }
                    }
                } else if state.active_toplevel.as_ref() == Some(handle) {
                    state.active_toplevel = None;
                    let mut title_lock = state.current_title.lock().unwrap();
                    if *title_lock != "No active window" {
                        debug!("Deactivated toplevel: No active window");
                        *title_lock = "No active window".to_string();
                    }
                }
            }
            zwlr_foreign_toplevel_handle_v1::Event::Closed => {
                state.toplevels.retain(|t| t != handle);
                if state.active_toplevel.as_ref() == Some(handle) {
                    state.active_toplevel = None;
                    let mut title_lock = state.current_title.lock().unwrap();
                    if *title_lock != "No active window" {
                        debug!("Toplevel closed: No active window");
                        *title_lock = "No active window".to_string();
                    }
                }
            }
            _ => {}
        }
    }
}