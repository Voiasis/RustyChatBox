use serde::{Deserialize, Serialize};
use sysinfo::{CpuExt, SystemExt};
use std::fs;
use std::path::Path;
use std::process::Command;
use eframe::egui;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentStatsOptions {
    pub enabled: bool,
    pub show_cpu: bool,
    pub show_gpu: bool,
    pub show_vram: bool,
    pub show_ram: bool,
    pub cpu_display_model: bool,
    pub cpu_custom_model: Option<String>,
    pub cpu_round_usage: bool,
    pub cpu_stylized_uppercase: bool,
    pub gpu_display_model: bool,
    pub gpu_custom_model: Option<String>,
    pub gpu_round_usage: bool,
    pub gpu_stylized_uppercase: bool,
    pub vram_round_usage: bool,
    pub vram_show_max: bool,
    pub vram_stylized_uppercase: bool,
    pub ram_round_usage: bool,
    pub ram_show_max: bool,
    pub ram_stylized_uppercase: bool,
}

impl ComponentStatsOptions {

    pub fn show_component_stats_options(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let mut response = ui.interact(
            egui::Rect::EVERYTHING,
            ui.id().with("component_stats_options"),
            egui::Sense::hover(),
        );
        response |= ui.checkbox(&mut self.show_cpu, "Show CPU stats");
        if self.show_cpu {
            response |= ui.checkbox(&mut self.cpu_display_model, "Display CPU model");
            if self.cpu_display_model {
                ui.label("Custom CPU model:");
                response |= ui.text_edit_singleline(self.cpu_custom_model.as_mut().unwrap_or(&mut String::new()));
            }
            response |= ui.checkbox(&mut self.cpu_round_usage, "Round CPU usage");
            response |= ui.checkbox(&mut self.cpu_stylized_uppercase, "Stylized uppercase for CPU");
        }
        response |= ui.checkbox(&mut self.show_gpu, "Show GPU stats");
        if self.show_gpu {
            response |= ui.checkbox(&mut self.gpu_display_model, "Display GPU model");
            if self.gpu_display_model {
                ui.label("Custom GPU model:");
                response |= ui.text_edit_singleline(self.gpu_custom_model.as_mut().unwrap_or(&mut String::new()));
            }
            response |= ui.checkbox(&mut self.gpu_round_usage, "Round GPU usage");
            response |= ui.checkbox(&mut self.gpu_stylized_uppercase, "Stylized uppercase for GPU");
        }
        response |= ui.checkbox(&mut self.show_vram, "Show VRAM stats");
        if self.show_vram {
            response |= ui.checkbox(&mut self.vram_round_usage, "Round VRAM usage");
            response |= ui.checkbox(&mut self.vram_show_max, "Show max VRAM");
            response |= ui.checkbox(&mut self.vram_stylized_uppercase, "Stylized uppercase for VRAM");
        }
        response |= ui.checkbox(&mut self.show_ram, "Show RAM stats");
        if self.show_ram {
            response |= ui.checkbox(&mut self.ram_round_usage, "Round RAM usage");
            response |= ui.checkbox(&mut self.ram_show_max, "Show max RAM");
            response |= ui.checkbox(&mut self.ram_stylized_uppercase, "Stylized uppercase for RAM");
        }
        response
    }
}

pub struct ComponentStatsModule {
    system: sysinfo::System,
    gpu_device: Option<String>,
    last_update: std::time::Instant,
    cached_stats: String,
}

impl ComponentStatsModule {
    pub fn new() -> Self {
        let mut system = sysinfo::System::new_all();
        system.refresh_all();
        let gpu_device = Self::detect_gpu();
        Self {
            system,
            gpu_device,
            last_update: std::time::Instant::now(),
            cached_stats: String::new(),
        }
    }

    pub fn get_primary_gpu() -> Option<String> {
        if let Ok(entries) = fs::read_dir("/sys/class/drm/") {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.join("device").exists() {
                    if let Ok(file_name) = entry.file_name().into_string() {
                        if file_name.starts_with("card") {
                            return Some(file_name);
                        }
                    }
                }
            }
        }
        None
    }

    pub fn get_available_gpus() -> Vec<String> {
        let mut gpus = Vec::new();
        if let Ok(output) = Command::new("lshw")
            .arg("-short")
            .arg("-C")
            .arg("display")
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            for line in output_str.lines() {
                if let Some(start) = line.find('[') {
                    if let Some(end) = line.find(']') {
                        let gpu_name = &line[start + 1..end];
                        gpus.push(gpu_name.to_string());
                    }
                }
            }
        } else {
            eprintln!("Failed to execute lshw command");
        }
        gpus.sort();
        gpus
    }

    pub fn extract_model_name(full_name: &str) -> String {
        if let Some(start) = full_name.rfind(' ') {
            return full_name[start + 1..].to_string();
        }
        full_name.to_string()
    }

    fn detect_gpu() -> Option<String> {
        let drm_path = Path::new("/sys/class/drm");
        if !drm_path.exists() {
            eprintln!("No /sys/class/drm. Unable to detect GPU.");
            return None;
        }
        if let Ok(entries) = fs::read_dir(drm_path) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("card") && !name.contains('-') {
                    let device_path = entry.path().join("device/vendor");
                    if let Ok(vendor) = fs::read_to_string(&device_path) {
                        let vendor_id = vendor.trim();
                        if vendor_id == "0x1002" || vendor_id == "0x10de" || vendor_id == "0x8086" {
                            return Some(name);
                        }
                    }
                }
            }
        }
        Self::get_primary_gpu()
    }

    pub fn get_cpu_usage(&mut self) -> f32 {
        self.system.refresh_cpu();
        self.system.global_cpu_info().cpu_usage()
    }

    pub fn get_cpu_model(&self) -> String {
        self.system.global_cpu_info().brand().to_string()
    }

    pub fn get_memory_usage(&self) -> (u64, u64) {
        let total = self.system.total_memory();
        let used = self.system.used_memory();
        (used, total)
    }

    pub fn get_gpu_usage(&self) -> Option<f32> {
        if let Some(card) = &self.gpu_device {
            let busy_path = format!("/sys/class/drm/{}/device/gpu_busy_percent", card);
            if let Ok(s) = fs::read_to_string(&busy_path) {
                if let Ok(usage) = s.trim().parse::<f32>() {
                    return Some(usage);
                } else {
                    eprintln!("Failed to parse GPU usage from {}: {}", busy_path, s);
                }
            } else {
                eprintln!("Failed to read GPU usage file: {}", busy_path);
            }
            if let Ok(output) = Command::new("nvidia-smi")
                .arg("--query-gpu=utilization.gpu")
                .arg("--format=csv,noheader")
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if let Some(usage_str) = output_str.lines().next() {
                    if let Some(usage) = usage_str.trim().strip_suffix(" %") {
                        if let Ok(usage) = usage.parse::<f32>() {
                            return Some(usage);
                        } else {
                            eprintln!("Failed to parse NVIDIA GPU usage: {}", usage_str);
                        }
                    }
                }
            } else {
                eprintln!("Failed to execute nvidia-smi command");
            }
        } else {
            eprintln!("No GPU device detected for usage");
        }
        None
    }

    pub fn get_gpu_vram_usage(&self) -> Option<(u64, u64)> {
        if let Some(card) = &self.gpu_device {
            let mem_info_path = format!("/sys/class/drm/{}/device/mem_info_vram_total", card);
            let used_path = format!("/sys/class/drm/{}/device/mem_info_vram_used", card);
            if let Ok(total_str) = fs::read_to_string(&mem_info_path) {
                if let Ok(total) = total_str.trim().parse::<u64>() {
                    if let Ok(used_str) = fs::read_to_string(&used_path) {
                        if let Ok(used) = used_str.trim().parse::<u64>() {
                            return Some((used, total));
                        } else {
                            eprintln!("Failed to parse VRAM used: {}", used_str);
                        }
                    } else {
                        eprintln!("Failed to read VRAM used file: {}", used_path);
                    }
                } else {
                    eprintln!("Failed to parse VRAM total: {}", total_str);
                }
            } else {
                eprintln!("Failed to read VRAM total file: {}", mem_info_path);
            }
        } else {
            eprintln!("No GPU device detected for VRAM usage");
        }
        None
    }

    fn update_stats(&mut self, options: &ComponentStatsOptions) {
        self.system.refresh_all();
        let mut parts = Vec::new();

        if options.show_cpu {
            let cpu_usage = self.get_cpu_usage();
            let cpu_label = if options.cpu_display_model {
                options.cpu_custom_model
                    .clone()
                    .unwrap_or_else(|| self.get_cpu_model())
            } else {
                "CPU".to_string()
            };
            let cpu_usage = if options.cpu_round_usage {
                cpu_usage.round()
            } else {
                cpu_usage
            };
            let cpu_text = format!("{}: {}%", cpu_label, cpu_usage);
            let cpu_text = if options.cpu_stylized_uppercase {
                cpu_text.to_uppercase()
            } else {
                cpu_text
            };
            parts.push(cpu_text);
        }
        if options.show_gpu {
            if let Some(gpu_usage) = self.get_gpu_usage() {
                let gpu_label = if options.gpu_display_model {
                    options.gpu_custom_model.clone().unwrap_or_else(|| {
                        Self::get_available_gpus()
                            .first()
                            .map(|s| Self::extract_model_name(s))
                            .unwrap_or("GPU".to_string())
                    })
                } else {
                    "GPU".to_string()
                };
                let gpu_usage = if options.gpu_round_usage {
                    gpu_usage.round()
                } else {
                    gpu_usage
                };
                let gpu_text = format!("{}: {}%", gpu_label, gpu_usage);
                let gpu_text = if options.gpu_stylized_uppercase {
                    gpu_text.to_uppercase()
                } else {
                    gpu_text
                };
                parts.push(gpu_text);
            } else {
                eprintln!("GPU usage unavailable, skipping");
            }
        }
        if options.show_vram {
            if let Some((used_vram, total_vram)) = self.get_gpu_vram_usage() {
                let used_gb = used_vram as f64 / 1_073_741_824.0;
                let total_gb = total_vram as f64 / 1_073_741_824.0;
                let vram_usage = if options.vram_round_usage {
                    used_gb.round()
                } else {
                    (used_gb * 100.0).round() / 100.0
                };
                let mut vram_text = format!("VRAM: {}GB", vram_usage);
                if options.vram_show_max {
                    vram_text = format!("VRAM: {}/{}GB", vram_usage, total_gb.round());
                }
                let vram_text = if options.vram_stylized_uppercase {
                    vram_text.to_uppercase()
                } else {
                    vram_text
                };
                parts.push(vram_text);
            } else {
                eprintln!("VRAM usage unavailable, skipping");
            }
        }
        if options.show_ram {
            let (used_memory, total_memory) = self.get_memory_usage();
            let used_gb = used_memory as f64 / 1_073_741_824.0;
            let total_gb = total_memory as f64 / 1_073_741_824.0;
            let ram_usage = if options.ram_round_usage {
                used_gb.round()
            } else {
                (used_gb * 100.0).round() / 100.0
            };
            let mut ram_text = format!("RAM: {}GB", ram_usage);
            if options.ram_show_max {
                ram_text = format!("RAM: {}/{}GB", ram_usage, total_gb.round());
            }
            let ram_text = if options.ram_stylized_uppercase {
                ram_text.to_uppercase()
            } else {
                ram_text
            };
            parts.push(ram_text);
        }
        self.cached_stats = parts.join("|");
    }

    pub fn get_formatted_stats(&mut self, options: &ComponentStatsOptions) -> String {
        if self.last_update.elapsed().as_secs() >= 1 {
            self.update_stats(options);
            self.last_update = std::time::Instant::now();
        }
        self.cached_stats.clone()
    }
}