# Build the project and create AppImage
build:
    cargo build --release
    mkdir -p AppDir/usr/bin AppDir/usr/lib AppDir/usr/share/applications AppDir/usr/share/icons/hicolor/64x64/apps
    cp target/release/rustychatbox AppDir/usr/bin/rustychatbox
    cp images/RustyChatBox_Icon.png AppDir/rustychatbox.png
    cp images/RustyChatBox_Icon.png AppDir/usr/share/icons/hicolor/64x64/apps/rustychatbox.png
    printf '[Desktop Entry]\nName=RustyChatBox\nExec=rustychatbox\nType=Application\nIcon=rustychatbox\nTerminal=false\nCategories=Utility;\nStartupWMClass=RustyChatBox\nComment=A chat application built with Rust\n' > AppDir/rustychatbox.desktop
    cp AppDir/rustychatbox.desktop AppDir/usr/share/applications/
    printf '#!/bin/bash\nHERE="$(dirname "$(readlink -f "${0}")")"\nexec "${HERE}/usr/bin/rustychatbox" "$@"\n' > AppDir/AppRun
    chmod +x AppDir/AppRun
    cp /lib64/libgcc_s.so.1 AppDir/usr/lib/ || true
    cp /usr/lib64/libGL.so.1 AppDir/usr/lib/ || true
    cp /usr/lib64/libEGL.so.1 AppDir/usr/lib/ || true
    cp /usr/lib64/libwayland-client.so.0 AppDir/usr/lib/ || true
    cp /usr/lib64/libwayland-egl.so.1 AppDir/usr/lib/ || true
    cp /usr/lib64/libwayland-cursor.so.0 AppDir/usr/lib/ || true
    cp /usr/lib64/libxkbcommon.so.0 AppDir/usr/lib/ || true
    appimagetool AppDir RustyChatBox.AppImage