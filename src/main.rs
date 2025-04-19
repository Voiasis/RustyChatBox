use std::io::Cursor;
use single_instance::SingleInstance;
use std::fs::{self, File};
use notify_rust::Notification;
use simplelog::{Config as LogConfig, LevelFilter, WriteLogger};
use eframe::egui;
use eframe::IconData;
use image::io::Reader as ImageReader;
use arboard::Clipboard;
use crate::config::Config;
use crate::gui::App;
use crate::osc::OscClient;
use crate::modules::netw_mod::NetworkOptions;

mod deps;
mod osc;
mod gui;
mod config;
mod modules {
    pub mod time_mod;
    pub mod stat_mod;
    pub mod netw_mod;
    pub mod medi_mod;
    pub mod extr_mod;
    pub mod comp_mod;
    pub mod chat_mod;
    pub mod appo_mod;
}

#[derive(Clone, PartialEq)]
enum LoadingState {
    SingleInstance,
    ConfigDir,
    LoadConfig,
    OscClient,
    Dependencies,
    Icon,
    InitializeApp,
    Done,
}

struct LoadingApp {
    state: LoadingState,
    progress: f32,
    message: String,
    config_path: std::path::PathBuf,
    config: Option<Config>,
    osc_client: Option<OscClient>,
    icon_data: Option<IconData>,
    error: Option<String>,
}

impl LoadingApp {
    fn new() -> Self {
        Self {
            state: LoadingState::SingleInstance,
            progress: 0.0,
            message: "Checking single instance".to_string(),
            config_path: dirs::config_dir()
                .unwrap()
                .join("RustyChatBox")
                .join("config.json"),
            config: None,
            osc_client: None,
            icon_data: None,
            error: None,
        }
    }

    fn update_state(&mut self) -> Option<RustyGUI> {
        match self.state {
            LoadingState::SingleInstance => {
                let instance = match SingleInstance::new("RustyChatBox") {
                    Ok(instance) => instance,
                    Err(e) => {
                        self.error = Some(format!("Failed to create lock: {}", e));
                        self.state = LoadingState::Done;
                        return None::<RustyGUI>;
                    }
                };
                if !instance.is_single() {
                    self.error = Some("Another instance is already running!".to_string());
                    Notification::new()
                        .summary("RustyChatBox")
                        .body("Another instance is already running!")
                        .show()
                        .expect("Failed to show notification");
                    self.state = LoadingState::Done;
                    return None::<RustyGUI>;
                }
                self.state = LoadingState::ConfigDir;
                self.progress = 0.14; // 1/7 tasks
                self.message = "Setting up config directory".to_string();
            }
            LoadingState::ConfigDir => {
                let config_dir = self.config_path.parent().unwrap();
                if !config_dir.exists() {
                    if let Err(e) = fs::create_dir_all(config_dir) {
                        self.error = Some(format!("Failed to create config directory: {}", e));
                        self.state = LoadingState::Done;
                        return None::<RustyGUI>;
                    }
                }
                let log_file = config_dir.join("app.log");
                let _ = fs::remove_file(&log_file);
                if let Err(e) = WriteLogger::init(
                    LevelFilter::Info,
                    LogConfig::default(),
                    File::create(&log_file).expect("Failed to create log file"),
                ) {
                    self.error = Some(format!("Failed to initialize logger: {}", e));
                    self.state = LoadingState::Done;
                    return None::<RustyGUI>;
                }
                self.state = LoadingState::LoadConfig;
                self.progress = 0.28; // 2/7
                self.message = "Loading configuration".to_string();
            }
            LoadingState::LoadConfig => {
                let default_network_options = NetworkOptions {
                    use_interface_max_speed: false,
                    show_download_speed: false,
                    show_upload_speed: false,
                    show_max_download: false,
                    show_max_upload: false,
                    show_total_download: false,
                    show_total_upload: false,
                    show_utilization: false,
                    stylized_chars: false,
                };
                let config = Config::load_or_create(&self.config_path, default_network_options);
                self.config = Some(config);
                self.state = LoadingState::OscClient;
                self.progress = 0.42; // 3/7
                self.message = "Initializing OSC client".to_string();
            }
            LoadingState::OscClient => {
                if let Some(config) = &self.config {
                    match OscClient::new(&config.app_options.osc_options.ip, config.app_options.osc_options.port) {
                        Ok(client) => self.osc_client = Some(client),
                        Err(e) => {
                            self.error = Some(format!("Failed to initialize OSC client: {}", e));
                            self.state = LoadingState::Done;
                            return None::<RustyGUI>;
                        }
                    }
                }
                self.state = LoadingState::Dependencies;
                self.progress = 0.57; // 4/7
                self.message = "Checking dependencies".to_string();
            }
            LoadingState::Dependencies => {
                if let Err(e) = deps::check_dependencies() {
                    self.error = Some(format!("Dependency error: {}", e));
                    self.state = LoadingState::Done;
                    return None::<RustyGUI>;
                }
                self.state = LoadingState::Icon;
                self.progress = 0.71; // 5/7
                self.message = "Loading icon".to_string();
            }
            LoadingState::Icon => {
                let icon_bytes = include_bytes!("../images/RustyChatBox_Icon.png");
                let icon_image = match ImageReader::new(Cursor::new(icon_bytes))
                    .with_guessed_format()
                    .map_err(|e| format!("Failed to read icon bytes: {}", e))
                {
                    Ok(reader) => reader,
                    Err(e) => {
                        self.error = Some(e);
                        self.state = LoadingState::Done;
                        return None::<RustyGUI>;
                    }
                };
                let decoded_image = match icon_image.decode() {
                    Ok(image) => image,
                    Err(e) => {
                        self.error = Some(format!("Error decoding icon: {}", e));
                        self.state = LoadingState::Done;
                        return None::<RustyGUI>;
                    }
                };
                let rgba_image = decoded_image.into_rgba8();
                let (width, height) = rgba_image.dimensions();
                self.icon_data = Some(IconData {
                    rgba: rgba_image.into_raw(),
                    width,
                    height,
                });
                self.state = LoadingState::InitializeApp;
                self.progress = 0.86; // 6/7
                self.message = "Initializing application".to_string();
            }
            LoadingState::InitializeApp => {
                if let (Some(config), Some(osc_client)) = (self.config.take(), self.osc_client.take()) {
                    let clipboard = match Clipboard::new() {
                        Ok(clipboard) => clipboard,
                        Err(e) => {
                            self.error = Some(format!("Failed to initialize clipboard: {}", e));
                            self.state = LoadingState::Done;
                            return None::<RustyGUI>;
                        }
                    };
                    let app = App::new(osc_client, config, clipboard);
                    self.state = LoadingState::Done;
                    self.progress = 1.0; // 7/7
                    self.message = "Complete".to_string();
                    return Some(RustyGUI {
                        app,
                        config_path: self.config_path.clone(),
                    });
                }
            }
            LoadingState::Done => {}
        }
        None::<RustyGUI>
    }
}

struct RustyGUI {
    app: App,
    config_path: std::path::PathBuf,
}

impl eframe::App for RustyGUI {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.app.update(ctx, frame);
        self.app.save_config_if_needed(&self.config_path);
    }
}

enum AppState {
    Loading(LoadingApp),
    Running(RustyGUI),
}

struct RustyChatBoxApp {
    state: AppState,
}

impl RustyChatBoxApp {
    fn new() -> Self {
        Self {
            state: AppState::Loading(LoadingApp::new()),
        }
    }
}

impl eframe::App for RustyChatBoxApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        match &mut self.state {
            AppState::Loading(loading_app) => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("RustyChatBox");
                        ui.add_space(20.0);
                        ui.label("Loading RustyChatBox");
                        ui.add(egui::ProgressBar::new(loading_app.progress).show_percentage());
                        ui.add_space(10.0);
                        ui.label(&loading_app.message);
                        if let Some(error) = &loading_app.error {
                            ui.add_space(10.0);
                            ui.colored_label(egui::Color32::RED, error);
                        }
                    });
                });

                if let Some(rusty_gui) = loading_app.update_state() {
                    self.state = AppState::Running(rusty_gui);
                }

                ctx.request_repaint();
            }
            AppState::Running(rusty_gui) => {
                rusty_gui.update(ctx, frame);
            }
        }
    }
}

fn main() {
    let icon_bytes = include_bytes!("../images/RustyChatBox_Icon.png");
    let icon_image = ImageReader::new(Cursor::new(icon_bytes))
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap()
        .into_rgba8();
    let (width, height) = icon_image.dimensions();
    let icon_data = IconData {
        rgba: icon_image.into_raw(),
        width,
        height,
    };

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(900.0, 500.0)),
        resizable: true,
        icon_data: Some(icon_data),
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        "RustyChatBox",
        options,
        Box::new(|_cc| Box::new(RustyChatBoxApp::new())),
    ) {
        eprintln!("Failed to run the application: {}", e);
    }
}