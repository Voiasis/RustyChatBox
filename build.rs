use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    // Create AppDir structure
    let app_dir = "AppDir";
    fs::create_dir_all(format!("{}/usr/bin", app_dir)).expect("Failed to create AppDir/usr/bin");
    fs::create_dir_all(format!("{}/usr/lib", app_dir)).expect("Failed to create AppDir/usr/lib");
    fs::create_dir_all(format!("{}/usr/share/applications", app_dir))
        .expect("Failed to create AppDir/usr/share/applications");
    fs::create_dir_all(format!("{}/usr/share/icons/hicolor/64x64/apps", app_dir))
        .expect("Failed to create AppDir/usr/share/icons");

    // Copy icon
    let icon_src = "images/RustyChatBox_Icon.png";
    if !Path::new(icon_src).exists() {
        panic!("Icon not found at {}.", icon_src);
    }
    fs::copy(icon_src, format!("{}/rustychatbox.png", app_dir))
        .expect("Failed to copy icon");
    fs::copy(
        icon_src,
        format!("{}/usr/share/icons/hicolor/64x64/apps/rustychatbox.png", app_dir),
    )
    .expect("Failed to copy icon");

    // Create .desktop file
    let desktop_content = "\
[Desktop Entry]
Name=RustyChatBox
Exec=rustychatbox
Type=Application
Icon=rustychatbox
Terminal=false
Categories=Utility;
StartupWMClass=RustyChatBox
Comment=A chat application built with Rust
";
    fs::write(format!("{}/rustychatbox.desktop", app_dir), desktop_content)
        .expect("Failed to write .desktop file");
    fs::copy(
        format!("{}/rustychatbox.desktop", app_dir),
        format!("{}/usr/share/applications/rustychatbox.desktop", app_dir),
    )
    .expect("Failed to copy .desktop file");

    // Create AppRun
    let apprun_content = "\
#!/bin/bash
HERE=\"$(dirname \"$(readlink -f \"${0}\")\")\"\n\
exec \"${HERE}/usr/bin/rustychatbox\" \"$@\"\
";
    fs::write(format!("{}/AppRun", app_dir), apprun_content).expect("Failed to write AppRun");
    Command::new("chmod")
        .args(["+x", &format!("{}/AppRun", app_dir)])
        .status()
        .expect("Failed to chmod AppRun");

    // Print post-build instructions
    println!("cargo:warning=AppDir created at {}.", app_dir);
    println!("cargo:warning=To create AppImage, run `just build` or manually:");
    println!("cargo:warning=  cp target/release/rustychatbox AppDir/usr/bin/rustychatbox");
    println!("cargo:warning=  cp /lib64/libgcc_s.so.1 AppDir/usr/lib/");
    println!("cargo:warning=  cp /usr/lib64/libGL.so.1 AppDir/usr/lib/ || true");
    println!("cargo:warning=  cp /usr/lib64/libEGL.so.1 AppDir/usr/lib/ || true");
    println!("cargo:warning=  cp /usr/lib64/libwayland-client.so.0 AppDir/usr/lib/ || true");
    println!("cargo:warning=  cp /usr/lib64/libwayland-egl.so.1 AppDir/usr/lib/ || true");
    println!("cargo:warning=  cp /usr/lib64/libwayland-cursor.so.0 AppDir/usr/lib/ || true");
    println!("cargo:warning=  cp /usr/lib64/libxkbcommon.so.0 AppDir/usr/lib/ || true");
    println!("cargo:warning=  appimagetool AppDir RustyChatBox.AppImage");
}