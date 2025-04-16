use std::io::Cursor;
use single_instance::SingleInstance;
use std::fs::{self, File};
use std::process;
use notify_rust::Notification;
use simplelog::{Config as LogConfig, LevelFilter, WriteLogger};
use serde_json;
use eframe::egui;
use eframe::IconData;
use image::io::Reader as ImageReader;

use crate::gui::{App, Config};
use crate::osc::OscClient;
use crate::modules::{
    appo_mod::AppOptions,
    chat_mod::ChatOptions,
    comp_mod::ComponentStatsOptions,
    extr_mod::ExtraOptions,
    medi_mod::MediaLinkOptions,
    netw_mod::{NetworkOptions, NetworkStatsOptions},
    stat_mod::StatusOptions,
    time_mod::TimeOptions,
};

mod deps;
mod osc;
mod gui;
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
        Self {
            app: App::new(osc_client, config),
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
    // Ensure only one instance runs
    let instance = SingleInstance::new("RustyChatBox").expect("Failed to create lock");
    if !instance.is_single() {
        Notification::new()
            .summary("RustyChatBox")
            .body("Another instance is already running!")
            .show()
            .expect("Failed to show notification");
        process::exit(1);
    }

    // Ensure the config directory exists
    let config_dir = dirs::config_dir()
        .unwrap()
        .join("RustyChatBox");
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).expect("Failed to create config directory");
    }

    // Initialize logger
    let log_file = config_dir.join("app.log");
    let _ = fs::remove_file(&log_file);
    WriteLogger::init(
        LevelFilter::Info,
        LogConfig::default(),
        File::create(&log_file).expect("Failed to create log file"),
    )
    .expect("Failed to initialize logger");

    // Load or create config
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
    let config = if config_path.exists() {
        match serde_json::from_str(&fs::read_to_string(&config_path).expect("Failed to read config")) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Invalid config file, using default: {}", e);
                Config {
                    app_options: AppOptions::new(),
                    personal_status_enabled: false,
                    component_stats_enabled: false,
                    network_stats_enabled: false,
                    current_time_enabled: false,
                    medialink_enabled: false,
                    chat_options: ChatOptions::new(),
                    component_stats_options: ComponentStatsOptions::new(),
                    extra_options: ExtraOptions::new(),
                    media_link_options: MediaLinkOptions::new(),
                    network_stats_options: NetworkStatsOptions::new(default_network_options.clone()),
                    status_options: StatusOptions::new(),
                    time_options: TimeOptions::new(),
                }
            }
        }
    } else {
        let config = Config {
            app_options: AppOptions::new(),
            personal_status_enabled: false,
            component_stats_enabled: false,
            network_stats_enabled: false,
            current_time_enabled: false,
            medialink_enabled: false,
            chat_options: ChatOptions::new(),
            component_stats_options: ComponentStatsOptions::new(),
            extra_options: ExtraOptions::new(),
            media_link_options: MediaLinkOptions::new(),
            network_stats_options: NetworkStatsOptions::new(default_network_options.clone()),
            status_options: StatusOptions::new(),
            time_options: TimeOptions::new(),
        };
        fs::write(&config_path, serde_json::to_string_pretty(&config).expect("Failed to serialize config"))
            .expect("Failed to write config");
        config
    };

    // Initialize OSC client
    let osc_client = OscClient::new(&config.app_options.osc_options.ip, config.app_options.osc_options.port)
        .expect("Failed to initialize OSC client");

    // Check dependencies
    if let Err(e) = deps::check_dependencies() {
        eprintln!("Dependency error: {}", e);
        process::exit(1);
    }

    // Load the embedded icon with enhanced debugging
    let icon_bytes = include_bytes!("../images/RustyChatBox_Icon.png");
    println!("Icon bytes length: {}", icon_bytes.len());
    let icon_image = match ImageReader::new(Cursor::new(icon_bytes))
        .with_guessed_format()
        .map_err(|e| format!("Failed to read icon bytes: {}", e))
    {
        Ok(reader) => reader,
        Err(e) => panic!("{}", e),
    }
    .decode()
    .map_err(|e| panic!("Error decoding icon: {}", e))
    .unwrap()
    .into_rgba8();
    // Save decoded icon for debugging
    icon_image
        .save_with_format("decoded_icon.png", image::ImageFormat::Png)
        .expect("Failed to decode icon");
    let (width, height) = icon_image.dimensions();
    let icon_data = IconData {
        rgba: icon_image.into_raw(),
        width,
        height,
    };
    println!("Embedded icon loaded: {}x{}", width, height);
    println!("Icon data length: {}, expected: {}", icon_data.rgba.len(), width * height * 4);

    // Launch the GUI with the icon
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