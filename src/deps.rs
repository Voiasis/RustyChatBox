// Dependency checks for RustyChatBox
use std::process::Command;

pub fn check_dependencies() -> Result<(), String> {
    let required_packages = vec!["playerctl", "qt5ct", "lshw"];
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
    Ok(())
}