// src/deps.rs
use std::process::Command;

pub fn check_dependencies() -> Result<(), String> {
    let required_packages = vec!["playerctl", "lshw"];
    for package in required_packages {
        if Command::new("which")
            .arg(package)
            .output()
            .map(|output| !output.status.success())
            .unwrap_or(true)
        {
            return Err(format!("Missing dependency: {}", package));
        }
    }

    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        if wayland_client::Connection::connect_to_env().is_err() {
            println!("Warning: Wayland display server is not available");
        }
    }

    let xlib = x11_dl::xlib::Xlib::open().map_err(|e| format!("Failed to load X11 library: {}", e))?;
    unsafe {
        let display = (xlib.XOpenDisplay)(std::ptr::null());
        if display.is_null() {
            return Err("X11 display server is not available".to_string());
        }
        (xlib.XCloseDisplay)(display);
    }

    Ok(())
}