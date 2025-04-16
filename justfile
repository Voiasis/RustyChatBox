# Build the project and create AppImage using linuxdeploy
build: ensure-tools
    cargo build --release
    # Manually copy critical libraries to AppDir/usr/lib
    mkdir -p AppDir/usr/lib
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
    NO_STRIP=true ./linuxdeploy-x86_64.AppImage --appdir AppDir --output appimage

# Ensure linuxdeploy and appimagetool are available
ensure-tools:
    #!/bin/bash
    if [ ! -f linuxdeploy-x86_64.AppImage ]; then
        curl -L -o linuxdeploy-x86_64.AppImage \
            https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
        chmod +x linuxdeploy-x86_64.AppImage
    fi