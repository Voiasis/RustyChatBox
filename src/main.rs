use std::io::Cursor;
use single_instance::SingleInstance;
use std::fs::{self, File};
use std::process;
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
struct RustyGUI {
    app: App,
    config_path: std::path::PathBuf,
}
impl RustyGUI {
    fn new(config: Config, config_path: std::path::PathBuf, osc_client: OscClient) -> Self {
        let clipboard = Clipboard::new().expect("Failed to initialize clipboard");
        Self {
            app: App::new(osc_client, config, clipboard),
            config_path,
        }
    }
}
impl eframe::App for RustyGUI {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.app.update(ctx, frame);
        self.app.save_config_if_needed(&self.config_path);
    }
}
fn main() {
    let instance = SingleInstance::new("RustyChatBox").expect("Failed to create lock");
    if !instance.is_single() {
        Notification::new()
            .summary("RustyChatBox")
            .body("Another instance is already running!")
            .show()
            .expect("Failed to show notification");
        process::exit(1);
    }
    let config_dir = dirs::config_dir()
        .unwrap()
        .join("RustyChatBox");
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).expect("Failed to create config directory");
    }
    let log_file = config_dir.join("app.log");
    let _ = fs::remove_file(&log_file);
    WriteLogger::init(
        LevelFilter::Info,
        LogConfig::default(),
        File::create(&log_file).expect("Failed to create log file"),
    )
    .expect("Failed to initialize logger");
    let config_path = config_dir.join("config.json");
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
    let config = Config::load_or_create(&config_path, default_network_options);
    let osc_client = OscClient::new(&config.app_options.osc_options.ip, config.app_options.osc_options.port)
        .expect("Failed to initialize OSC client");
    if let Err(e) = deps::check_dependencies() {
        eprintln!("Dependency error: {}", e);
        process::exit(1);
    }
    let icon_bytes = include_bytes!("../images/RustyChatBox_Icon.png");
    println!("Icon bytes length: {}", icon_bytes.len());
    let icon_image = match ImageReader::new(Cursor::new(icon_bytes))
        .with_guessed_format()
        .map_err(|e| format!("Failed to read icon bytes: {}", e)) {
            Ok(reader) => reader,
            Err(e) => panic!("{}", e),
        }
        .decode()
        .map_err(|e| panic!("Error decoding icon: {}", e))
        .unwrap()
        .into_rgba8();
    let (width, height) = icon_image.dimensions();
    let icon_data = IconData {
        rgba: icon_image.into_raw(),
        width,
        height,
    };
    println!("Embedded icon loaded: {}x{}", width, height);
    println!("Icon data length: {}, expected: {}", icon_data.rgba.len(), width * height * 4);
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(900.0, 500.0)),
        resizable: true,
        icon_data: Some(icon_data),
        ..Default::default()
    };
    if let Err(e) = eframe::run_native(
        "RustyChatBox",
        options,
        Box::new(|_cc| Box::new(RustyGUI::new(config, config_path, osc_client))),
    ) {
        eprintln!("Failed to run the application: {}", e);
    }
}