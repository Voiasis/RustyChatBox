use std::io::Cursor;
use single_instance::SingleInstance;
use std::fs;
use notify_rust::Notification;
use eframe::egui;
use eframe::egui::{IconData, FontDefinitions, FontFamily, Color32, RichText, Visuals, Stroke, Rounding};
use image::io::Reader as ImageReader;
use arboard::Clipboard;
use crate::config::Config;
use crate::ui::App;
use crate::osc::OscClient;
use crate::modules::network::NetworkOptions;

mod deps;
mod osc;
mod config;
mod ui;
mod modules {
    pub mod time;
    pub mod status;
    pub mod network;
    pub mod media;
    pub mod extra;
    pub mod component;
    pub mod chatting;
    pub mod app;
    pub mod activity;
}

#[derive(Clone, PartialEq)]
enum LoadingState {
    SingleInstance,
    ConfigDir,
    LoadConfig,
    OscClient,
    Dependencies,
    Icon,
    Fonts,
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
    font_definitions: Option<FontDefinitions>,
    error: Option<String>,
    spinner_frame: usize,
}

impl LoadingApp {
    fn new() -> Self {
        Self {
            state: LoadingState::SingleInstance,
            progress: 0.0,
            message: "Checking single instance...".to_string(),
            config_path: dirs::config_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("RustyChatBox")
                .join("config.json"),
            config: None,
            osc_client: None,
            icon_data: None,
            font_definitions: None,
            error: None,
            spinner_frame: 0,
        }
    }

    fn update_state(&mut self) -> Option<RustyGUI> {
        match self.state {
            LoadingState::SingleInstance => {
                log::info!("Checking single instance");
                let instance = match SingleInstance::new("RustyChatBox") {
                    Ok(instance) => instance,
                    Err(e) => {
                        let error_msg = format!("Failed to create lock: {}", e);
                        log::error!("{}", error_msg);
                        self.error = Some(format!("{} Please check for system permissions.", error_msg));
                        self.state = LoadingState::Done;
                        return None::<RustyGUI>;
                    }
                };
                if !instance.is_single() {
                    let error_msg = "Another instance is already running!";
                    log::error!("{}", error_msg);
                    Notification::new()
                        .summary("RustyChatBox")
                        .body(error_msg)
                        .show()
                        .expect("Failed to show notification");
                    self.error = Some(format!("{} Close the other instance and try again.", error_msg));
                    self.state = LoadingState::Done;
                    return None::<RustyGUI>;
                }
                self.state = LoadingState::ConfigDir;
                self.progress = 0.125;
                self.message = "Setting up config directory...".to_string();
            }
            LoadingState::ConfigDir => {
                log::info!("Setting up config directory");
                let config_dir = self.config_path.parent().unwrap();
                if !config_dir.exists() {
                    if let Err(e) = fs::create_dir_all(config_dir) {
                        let error_msg = format!("Failed to create config directory: {}", e);
                        log::error!("{}", error_msg);
                        self.error = Some(format!("{} Ensure you have write permissions to {}.", error_msg, config_dir.display()));
                        self.state = LoadingState::Done;
                        return None::<RustyGUI>;
                    }
                }
                self.state = LoadingState::LoadConfig;
                self.progress = 0.25;
                self.message = "Loading configuration...".to_string();
            }
            LoadingState::LoadConfig => {
                log::info!("Loading configuration");
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
                self.progress = 0.375;
                self.message = "Initializing OSC client...".to_string();
            }
            LoadingState::OscClient => {
                log::info!("Initializing OSC client");
                if let Some(config) = &self.config {
                    match OscClient::new(&config.app_options.osc_options.ip, config.app_options.osc_options.port) {
                        Ok(client) => self.osc_client = Some(client),
                        Err(e) => {
                            let error_msg = format!("Failed to initialize OSC client: {}", e);
                            log::error!("{}", error_msg);
                            self.error = Some(format!("{} Check the IP ({}) and port ({}).", error_msg, config.app_options.osc_options.ip, config.app_options.osc_options.port));
                            self.state = LoadingState::Done;
                            return None::<RustyGUI>;
                        }
                    }
                }
                self.state = LoadingState::Dependencies;
                self.progress = 0.5;
                self.message = "Checking dependencies...".to_string();
            }
            LoadingState::Dependencies => {
                log::info!("Checking dependencies");
                if let Err(e) = deps::check_dependencies() {
                    let error_msg = format!("Dependency error: {}", e);
                    log::error!("{}", error_msg);
                    self.error = Some(format!("{} Install missing dependencies (e.g., kdotool, xdotool).", error_msg));
                    self.state = LoadingState::Done;
                    return None::<RustyGUI>;
                }
                self.state = LoadingState::Icon;
                self.progress = 0.625;
                self.message = "Loading icon...".to_string();
            }
            LoadingState::Icon => {
                log::info!("Loading icon");
                let icon_bytes = include_bytes!("../images/RustyChatBox_Icon.png");
                let icon_image = match ImageReader::new(Cursor::new(icon_bytes))
                    .with_guessed_format()
                    .map_err(|e| format!("Failed to read icon bytes: {}", e))
                {
                    Ok(reader) => reader,
                    Err(e) => {
                        log::error!("{}", e);
                        self.error = Some(format!("{} Ensure the icon file is valid.", e));
                        self.state = LoadingState::Done;
                        return None::<RustyGUI>;
                    }
                };
                let decoded_image = match icon_image.decode() {
                    Ok(image) => image,
                    Err(e) => {
                        let error_msg = format!("Error decoding icon: {}", e);
                        log::error!("{}", error_msg);
                        self.error = Some(format!("{} Check the icon format (PNG expected).", error_msg));
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
                self.state = LoadingState::Fonts;
                self.progress = 0.75;
                self.message = "Loading fonts...".to_string();
            }
            LoadingState::Fonts => {
                log::info!("Loading fonts");
                let mut fonts = FontDefinitions::default();
                fonts.font_data.insert(
                    "NotoEmoji".to_owned(),
                    egui::FontData::from_static(include_bytes!("../assets/NotoEmoji-Regular.ttf")),
                );
                fonts.families.entry(FontFamily::Proportional).or_insert_with(Vec::new).push("NotoEmoji".to_owned());
                fonts.families.entry(FontFamily::Monospace).or_insert_with(Vec::new).push("NotoEmoji".to_owned());
                self.font_definitions = Some(fonts);
                self.state = LoadingState::InitializeApp;
                self.progress = 0.875;
                self.message = "Initializing application...".to_string();
            }
            LoadingState::InitializeApp => {
                log::info!("Initializing application");
                if let (Some(config), Some(osc_client), Some(font_definitions)) = (self.config.take(), self.osc_client.take(), self.font_definitions.take()) {
                    let clipboard = match Clipboard::new() {
                        Ok(clipboard) => clipboard,
                        Err(e) => {
                            let error_msg = format!("Failed to initialize clipboard: {}", e);
                            log::error!("{}", error_msg);
                            self.error = Some(format!("{} Ensure clipboard access is available.", error_msg));
                            self.state = LoadingState::Done;
                            return None::<RustyGUI>;
                        }
                    };
                    let app = App::new(osc_client, config, clipboard);
                    self.state = LoadingState::Done;
                    self.progress = 1.0;
                    self.message = "Complete".to_string();
                    log::info!("Application initialized");
                    return Some(RustyGUI {
                        app,
                        config_path: self.config_path.clone(),
                        font_definitions,
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
    font_definitions: FontDefinitions,
}

impl eframe::App for RustyGUI {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.set_fonts(self.font_definitions.clone());
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
        let hex_to_color = |hex: &str| {
            let hex = hex.trim_start_matches('#');
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
            Color32::from_rgb(r, g, b)
        };

        let border_color = hex_to_color("ff3f00");
        let background_color = hex_to_color("141414");
        let font_color = hex_to_color("ffffff");
        let disabled_color = hex_to_color("7200ff");
        let enabled_color = hex_to_color("ff3f00");
        let button_color = hex_to_color("3f3f3f");
        let slider_color = hex_to_color("ff3f00");
        let dropdown_color = hex_to_color("3f3f3f");
        let dropdown_outline_color = hex_to_color("b7410e");
        let dropdown_hover_color = hex_to_color("b7410e");
        let input_field_color = hex_to_color("000000");
        let scrollbar_color = hex_to_color("3f3f3f");
        let _inactive_tab_color = hex_to_color("3f3f3f");

        let mut visuals = Visuals::default();
        visuals.dark_mode = true;
        visuals.panel_fill = background_color;
        visuals.window_fill = dropdown_color;
        visuals.widgets.noninteractive.bg_fill = button_color;
        visuals.widgets.inactive.bg_fill = button_color;
        visuals.widgets.active.bg_fill = enabled_color;
        visuals.widgets.hovered.bg_fill = enabled_color;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, border_color);
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, border_color);
        visuals.widgets.active.bg_stroke = Stroke::new(1.0, border_color);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, border_color);
        visuals.window_stroke = Stroke::new(1.0, dropdown_outline_color);
        visuals.widgets.noninteractive.rounding = Rounding::same(4.0);
        visuals.override_text_color = Some(font_color);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, font_color);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, font_color);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, font_color);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, font_color);
        visuals.widgets.inactive.bg_fill = disabled_color;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, border_color);
        visuals.widgets.active.bg_fill = enabled_color;
        visuals.widgets.active.bg_stroke = Stroke::new(1.0, border_color);
        visuals.widgets.hovered.bg_fill = enabled_color;
        visuals.widgets.noninteractive.bg_fill = button_color;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, border_color);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, font_color);
        visuals.widgets.active.bg_fill = slider_color;
        visuals.widgets.hovered.bg_fill = slider_color;
        visuals.widgets.inactive.bg_fill = slider_color.gamma_multiply(0.5);
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, border_color);
        visuals.widgets.open.bg_fill = dropdown_color;
        visuals.widgets.open.bg_stroke = Stroke::new(1.0, dropdown_outline_color);
        visuals.widgets.open.fg_stroke = Stroke::new(1.0, font_color);
        visuals.selection.bg_fill = dropdown_hover_color;
        visuals.extreme_bg_color = input_field_color;
        visuals.selection.stroke = Stroke::new(1.0, font_color);
        visuals.widgets.noninteractive.bg_fill = scrollbar_color;
        visuals.widgets.inactive.bg_fill = scrollbar_color.gamma_multiply(0.5);
        visuals.widgets.noninteractive.bg_fill = button_color;
        visuals.hyperlink_color = Color32::from_rgb(0x72, 0x00, 0xff);
        visuals.faint_bg_color = Color32::from_rgb(0x72, 0x00, 0xff).gamma_multiply(0.5);
        visuals.code_bg_color = Color32::from_rgb(0x72, 0x00, 0xff).gamma_multiply(0.2);

        match &mut self.state {
            AppState::Loading(loading_app) => {
                ctx.set_visuals(visuals.clone());

                loading_app.spinner_frame = (loading_app.spinner_frame + 1) % 8;
                let spinner = ["⠋", "⠙", "⠚", "⠞", "⠖", "⠦", "⠴", "⠷"][loading_app.spinner_frame];

                egui::CentralPanel::default()
                    .frame(egui::Frame {
                        fill: background_color,
                        inner_margin: egui::Margin::same(10.0),
                        rounding: Rounding::same(10.0),
                        ..Default::default()
                    })
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.heading(RichText::new("RustyChatBox").size(20.0).strong().color(font_color));
                            ui.add_space(15.0);
                            ui.label(RichText::new(format!("Loading {} {}", spinner, loading_app.message)).size(14.0).color(font_color));
                            ui.add_space(10.0);
                            ui.add(
                                egui::ProgressBar::new(loading_app.progress)
                                    .show_percentage()
                                    .animate(true)
                                    .desired_width(250.0)
                                    .fill(slider_color),
                            );
                            if let Some(error) = &loading_app.error {
                                ui.add_space(15.0);
                                ui.colored_label(
                                    Color32::RED,
                                    RichText::new(error).size(12.0).italics(),
                                );
                                ui.add_space(10.0);
                                ui.label(RichText::new("Please resolve the issue and restart.").size(12.0).color(font_color));
                            }
                        });
                    });

                if let Some(rusty_gui) = loading_app.update_state() {
                    self.state = AppState::Running(rusty_gui);
                }

                ctx.request_repaint();
            }
            AppState::Running(rusty_gui) => {
                ctx.set_visuals(visuals);
                rusty_gui.update(ctx, frame);
            }
        }
    }
}

fn setup_logger() -> Result<(), fern::InitError> {
    let log_dir = dirs::config_dir()
        .ok_or_else(|| fern::InitError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "Config dir not found")))?
        .join("RustyChatBox");
    let logs_folder = log_dir.join("logs");
    std::fs::create_dir_all(&logs_folder)?;
    let latest_log = log_dir.join("latestlog.txt");
    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let timestamped_log = logs_folder.join(format!("{}.txt", timestamp));
    let _ = std::fs::remove_file(&latest_log);
    let log_level = match std::env::var("RUST_LOG").as_deref() {
        Ok("debug") => log::LevelFilter::Debug,
        Ok("info") => log::LevelFilter::Info,
        Ok("warn") => log::LevelFilter::Warn,
        Ok("error") => log::LevelFilter::Error,
        _ => log::LevelFilter::Info,
    };

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log_level)
        .chain(std::io::stdout())
        .chain(fern::log_file(latest_log)?)
        .chain(fern::log_file(timestamped_log)?)
        .apply()?;
    Ok(())
}

fn main() {
    if let Err(e) = setup_logger() {
        eprintln!("Failed to initialize logger: {}", e);
        std::process::exit(1);
    }
    log::info!("Starting RustyChatBox");

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_icon(load_icon())
            .with_inner_size([900.0,600.0]),
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        "RustyChatBox",
        native_options,
        Box::new(|_cc| Ok(Box::new(RustyChatBoxApp::new()))),
    ) {
        log::error!("Failed to run the application: {}", e);
        eprintln!("Failed to run the application: {}", e);
    }
}

fn load_icon() -> IconData {
    let icon_bytes = include_bytes!("../images/RustyChatBox_Icon.png");
    let icon_image = ImageReader::new(Cursor::new(icon_bytes))
        .with_guessed_format()
        .expect("Failed to read icon bytes")
        .decode()
        .expect("Failed to decode icon");
    let rgba_image = icon_image.into_rgba8();
    let (width, height) = rgba_image.dimensions();
    IconData {
        rgba: rgba_image.into_raw(),
        width,
        height,
    }
}