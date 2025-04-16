# Build the project and create AppImage
build: ensure-tools
    cargo build --release
    rm -rf AppDir
    mkdir -p AppDir/usr/bin AppDir/usr/lib AppDir/usr/share/applications AppDir/usr/share/icons/hicolor/64x64/apps
    cp target/release/rustychatbox AppDir/usr/bin/rustychatbox
    cp images/RustyChatBox_Icon.png AppDir/rustychatbox.png
    cp images/RustyChatBox_Icon.png AppDir/usr/share/icons/hicolor/64x64/apps/rustychatbox.png
    printf '[Desktop Entry]\nName=RustyChatBox\nExec=rustychatbox\nType=Application\nIcon=rustychatbox\nTerminal=false\nCategories=Utility;\nStartupWMClass=RustyChatBox\nComment=A chat application built with Rust\n' > AppDir/rustychatbox.desktop
    cp AppDir/rustychatbox.desktop AppDir/usr/share/applications/
    printf '#!/bin/bash\nHERE="$(dirname "$(readlink -f "${0}")")"\nexec "${HERE}/usr/bin/rustychatbox" "$@"\n' > AppDir/AppRun
    chmod +x AppDir/AppRun
    # Copy Fedora libraries
    cp /lib64/libgtk-3.so.0 AppDir/usr/lib/ || true
    cp /lib64/libgdk-3.so.0 AppDir/usr/lib/ || true
    cp /lib64/libegl.so.1 AppDir/usr/lib/ || true
    cp /lib64/libgl.so.1 AppDir/usr/lib/ || true
    cp /lib64/libxkbcommon.so.0 AppDir/usr/lib/ || true
    cp /lib64/libwayland-client.so.0 AppDir/usr/lib/ || true
    cp /lib64/libwayland-egl.so.1 AppDir/usr/lib/ || true
    cp /lib64/libwayland-cursor.so.0 AppDir/usr/lib/ || true
    cp /lib64/libz.so.1 AppDir/usr/lib/ || true
    cp /lib64/libpng16.so.16 AppDir/usr/lib/ || true
    cp /lib64/libdbus-1.so.3 AppDir/usr/lib/ || true
    cp /lib64/libgdk_pixbuf-2.0.so.0 AppDir/usr/lib/ || true
    cp /lib64/libpango-1.0.so.0 AppDir/usr/lib/ || true
    cp /lib64/libcairo.so.2 AppDir/usr/lib/ || true
    cp /lib64/libfontconfig.so.1 AppDir/usr/lib/ || true
    cp /lib64/libfreetype.so.6 AppDir/usr/lib/ || true
    cp /lib64/libX11.so.6 AppDir/usr/lib/ || true
    cp /lib64/libXext.so.6 AppDir/usr/lib/ || true
    cp /lib64/libXau.so.6 AppDir/usr/lib/ || true
    cp /lib64/libffi.so.8 AppDir/usr/lib/ || true
    # Copy Ubuntu/Debian libraries
    cp /usr/lib/x86_64-linux-gnu/libgtk-3.so.0 AppDir/usr/lib/ || true
    cp /usr/lib/x86_64-linux-gnu/libgdk-3.so.0 AppDir/usr/lib/ || true
    cp /usr/lib/x86_64-linux-gnu/libegl.so.1 AppDir/usr/lib/ || true
    cp /usr/lib/x86_64-linux-gnu/libGL.so.1 AppDir/usr/lib/ || true
    cp /usr/lib/x86_64-linux-gnu/libxkbcommon.so.0 AppDir/usr/lib/ || true
    cp /usr/lib/x86_64-linux-gnu/libwayland-client.so.0 AppDir/usr/lib/ || true
    cp /usr/lib/x86_64-linux-gnu/libwayland-egl.so.1 AppDir/usr/lib/ || true
    cp /usr/lib/x86_64-linux-gnu/libwayland-cursor.so.0 AppDir/usr/lib/ || true
    cp /usr/lib/x86_64-linux-gnu/libz.so.1 AppDir/usr/lib/ || true
    cp /usr/lib/x86_64-linux-gnu/libpng16.so.16 AppDir/usr/lib/ || true
    cp /usr/lib/x86_64-linux-gnu/libdbus-1.so.3 AppDir/usr/lib/ || true
    cp /usr/lib/x86_64-linux-gnu/libgdk_pixbuf-2.0.so.0 AppDir/usr/lib/ || true
    cp /usr/lib/x86_64-linux-gnu/libpango-1.0.so.0 AppDir/usr/lib/ || true
    cp /usr/lib/x86_64-linux-gnu/libcairo.so.2 AppDir/usr/lib/ || true
    cp /usr/lib/x86_64-linux-gnu/libfontconfig.so.1 AppDir/usr/lib/ || true
    cp /usr/lib/x86_64-linux-gnu/libfreetype.so.6 AppDir/usr/lib/ || true
    cp /usr/lib/x86_64-linux-gnu/libX11.so.6 AppDir/usr/lib/ || true
    cp /usr/lib/x86_64-linux-gnu/libXext.so.6 AppDir/usr/lib/ || true
    cp /usr/lib/x86_64-linux-gnu/libXau.so.6 AppDir/usr/lib/ || true
    cp /usr/lib/x86_64-linux-gnu/libffi.so.8 AppDir/usr/lib/ || true
    # Copy Arch libraries
    cp /usr/lib/libgtk-3.so.0 AppDir/usr/lib/ || true
    cp /usr/lib/libgdk-3.so.0 AppDir/usr/lib/ || true
    cp /usr/lib/libEGL.so.1 AppDir/usr/lib/ || true
    cp /usr/lib/libGL.so.1 AppDir/usr/lib/ || true
    cp /usr/lib/libxkbcommon.so.0 AppDir/usr/lib/ || true
    cp /usr/lib/libwayland-client.so.0 AppDir/usr/lib/ || true
    cp /usr/lib/libwayland-egl.so.1 AppDir/usr/lib/ || true
    cp /usr/lib/libwayland-cursor.so.0 AppDir/usr/lib/ || true
    cp /usr/lib/libz.so.1 AppDir/usr/lib/ || true
    cp /usr/lib/libpng16.so.16 AppDir/usr/lib/ || true
    cp /usr/lib/libdbus-1.so.3 AppDir/usr/lib/ || true
    cp /usr/lib/libgdk_pixbuf-2.0.so.0 AppDir/usr/lib/ || true
    cp /usr/lib/libpango-1.0.so.0 AppDir/usr/lib/ || true
    cp /usr/lib/libcairo.so.2 AppDir/usr/lib/ || true
    cp /usr/lib/libfontconfig.so.1 AppDir/usr/lib/ || true
    cp /usr/lib/libfreetype.so.6 AppDir/usr/lib/ || true
    cp /usr/lib/libX11.so.6 AppDir/usr/lib/ || true
    cp /usr/lib/libXext.so.6 AppDir/usr/lib/ || true
    cp /usr/lib/libXau.so.6 AppDir/usr/lib/ || true
    cp /usr/lib/libffi.so.8 AppDir/usr/lib/ || true
    ARCH=x86_64 ./appimagetool-x86_64.AppImage AppDir RustyChatBox-x86_64.AppImage

# Ensure appimagetool are available
ensure-tools:
    #!/bin/bash
    if [ ! -f appimagetool-x86_64.AppImage ]; then
        wget https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage
        chmod +x appimagetool-x86_64.AppImage
    fi