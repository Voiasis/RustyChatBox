# Build the project and create AppImage
build: ensure-tools
    cargo build --release
    rm -rf AppDir
    mkdir -p AppDir/usr/bin AppDir/usr/lib AppDir/usr/share/applications AppDir/usr/share/icons/hicolor/64x64/apps AppDir/usr/lib/qt5/plugins
    cp target/release/rustychatbox AppDir/usr/bin/rustychatbox
    cp images/RustyChatBox_Icon.png AppDir/rustychatbox.png
    cp images/RustyChatBox_Icon.png AppDir/usr/share/icons/hicolor/64x64/apps/rustychatbox.png
    printf '[Desktop Entry]\nName=RustyChatBox\nExec=rustychatbox\nType=Application\nIcon=rustychatbox\nTerminal=false\nCategories=Utility;\nStartupWMClass=RustyChatBox\nComment=A chat application built with Rust\n' > AppDir/rustychatbox.desktop
    cp AppDir/rustychatbox.desktop AppDir/usr/share/applications/
    printf '#!/bin/bash\nHERE="$(dirname "$(readlink -f "${0}")")"\nLD_LIBRARY_PATH="${HERE}/usr/lib" QT_LOGGING_RULES="qt5ct.debug=false" QT_QPA_PLATFORM_PLUGIN_PATH="${HERE}/usr/lib/qt5/plugins" PATH="${HERE}/usr/bin:$PATH" exec "${HERE}/usr/bin/rustychatbox" "$@"\n' > AppDir/AppRun
    chmod +x AppDir/AppRun
    # Copy required libraries using ldd
    ldd target/release/rustychatbox | grep "=> /" | awk '{print $3}' | xargs -I {} cp {} AppDir/usr/lib/ || true
    # Copy transitive dependencies for bundled libraries
    /bin/bash -c 'for lib in AppDir/usr/lib/*.so*; do ldd "$$lib" | grep "=> /" | awk "{print $$3}" | xargs -I {} cp {} AppDir/usr/lib/ 2>/dev/null || true; done'
    # Copy playerctl binary if available (unchanged from your fix)
    if command -v playerctl >/dev/null 2>&1; then \
        cp $(which playerctl) AppDir/usr/bin/ 2>/dev/null || true; \
        echo "playerctl binary copied to AppDir/usr/bin"; \
    else \
        echo "Warning: playerctl binary not found on system. MediaLink functionality may not work."; \
    fi
    # Copy qt5ct binary if available
    if command -v qt5ct >/dev/null 2>&1; then \
        cp $(which qt5ct) AppDir/usr/bin/ 2>/dev/null || true; \
        echo "qt5ct binary copied to AppDir/usr/bin"; \
    else \
        echo "Warning: qt5ct binary not found on system. Qt5 configuration (theming) may not work."; \
    fi
    # Copy lshw binary if available
    if command -v lshw >/dev/null 2>&1; then \
        cp $(which lshw) AppDir/usr/bin/ 2>/dev/null || true; \
        echo "lshw binary copied to AppDir/usr/bin"; \
    else \
        echo "Warning: lshw binary not found on system. Hardware information (ComponentStats) may not work."; \
    fi
    # Copy playerctl library from multiple possible paths
    cp /usr/lib/libplayerctl.so.2 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libplayerctl.so.2 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libplayerctl.so.2 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libplayerctl.so.2 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/libplayerctl.so AppDir/usr/lib/ 2>/dev/null || true
    # Copy Qt libraries
    cp /usr/lib/libQt5Core.so.5 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libQt5Core.so.5 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libQt5Core.so.5 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libQt5Core.so.5 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/libQt5Gui.so.5 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libQt5Gui.so.5 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libQt5Gui.so.5 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libQt5Gui.so.5 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/libQt5Widgets.so.5 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libQt5Widgets.so.5 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libQt5Widgets.so.5 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libQt5Widgets.so.5 AppDir/usr/lib/ 2>/dev/null || true
    # Copy lshw-related libraries (e.g., libpci.so.3)
    cp /usr/lib/libpci.so.3 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libpci.so.3 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libpci.so.3 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libpci.so.3 AppDir/usr/lib/ 2>/dev/null || true
    # Copy additional Qt-related libraries for qt5ct
    cp /usr/lib/libQt5DBus.so.5 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libQt5DBus.so.5 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libQt5DBus.so.5 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libQt5DBus.so.5 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/libQt5XcbQpa.so.5 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libQt5XcbQpa.so.5 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libQt5XcbQpa.so.5 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libQt5XcbQpa.so.5 AppDir/usr/lib/ 2>/dev/null || true
    # Copy additional dependencies for lshw and qt5ct
    cp /usr/lib/libglib-2.0.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libglib-2.0.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libglib-2.0.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libglib-2.0.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/libgio-2.0.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libgio-2.0.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libgio-2.0.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libgio-2.0.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/libgobject-2.0.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libgobject-2.0.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libgobject-2.0.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libgobject-2.0.so.0 AppDir/usr/lib/ 2>/dev/null || true
    # Copy other common libraries
    cp /usr/lib/libgtk-3.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libgtk-3.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libgtk-3.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libgtk-3.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/libgdk-3.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libgdk-3.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libgdk-3.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libgdk-3.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/libegl.so.1 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libegl.so.1 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libegl.so.1 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libegl.so.1 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/libGL.so.1 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libGL.so.1 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libGL.so.1 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libGL.so.1 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/libxkbcommon.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libxkbcommon.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libxkbcommon.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libxkbcommon.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/libwayland-client.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libwayland-client.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libwayland-client.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libwayland-client.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/libwayland-egl.so.1 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libwayland-egl.so.1 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libwayland-egl.so.1 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libwayland-egl.so.1 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/libwayland-cursor.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libwayland-cursor.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libwayland-cursor.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libwayland-cursor.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/libz.so.1 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libz.so.1 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libz.so.1 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libz.so.1 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/libpng16.so.16 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libpng16.so.16 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libpng16.so.16 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libpng16.so.16 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/libdbus-1.so.3 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libdbus-1.so.3 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libdbus-1.so.3 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libdbus-1.so.3 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/libgdk_pixbuf-2.0.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libgdk_pixbuf-2.0.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libgdk_pixbuf-2.0.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libgdk_pixbuf-2.0.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/libpango-1.0.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libpango-1.0.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libpango-1.0.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libpango-1.0.so.0 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/libcairo.so.2 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libcairo.so.2 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libcairo.so.2 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libcairo.so.2 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/libfontconfig.so.1 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libfontconfig.so.1 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libfontconfig.so.1 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libfontconfig.so.1 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/libfreetype.so.6 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libfreetype.so.6 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libfreetype.so.6 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libfreetype.so.6 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/libX11.so.6 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libX11.so.6 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libX11.so.6 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libX11.so.6 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/libXext.so.6 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libXext.so.6 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libXext.so.6 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libXext.so.6 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/libXau.so.6 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libXau.so.6 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libXau.so.6 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libXau.so.6 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/libffi.so.8 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libffi.so.8 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libffi.so.8 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libffi.so.8 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/libxcb.so.1 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libxcb.so.1 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libxcb.so.1 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libxcb.so.1 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/libX11-xcb.so.1 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libX11-xcb.so.1 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libX11-xcb.so.1 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libX11-xcb.so.1 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/libXrender.so.1 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libXrender.so.1 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libXrender.so.1 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libXrender.so.1 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/libXi.so.6 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libXi.so.6 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libXi.so.6 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libXi.so.6 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/libSM.so.6 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libSM.so.6 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libSM.so.6 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libSM.so.6 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/libICE.so.6 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libICE.so.6 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libICE.so.6 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libICE.so.6 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/libuuid.so.1 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib64/libuuid.so.1 AppDir/usr/lib/ 2>/dev/null || true
    cp /lib64/libuuid.so.1 AppDir/usr/lib/ 2>/dev/null || true
    cp /usr/lib/x86_64-linux-gnu/libuuid.so.1 AppDir/usr/lib/ 2>/dev/null || true
    # Copy Qt plugins
    cp -r /usr/lib/x86_64-linux-gnu/qt5/plugins/* AppDir/usr/lib/qt5/plugins/ 2>/dev/null || true
    cp -r /usr/lib64/qt5/plugins/* AppDir/usr/lib/qt5/plugins/ 2>/dev/null || true
    cp -r /usr/lib/qt5/plugins/* AppDir/usr/lib/qt5/plugins/ 2>/dev/null || true
    # Create AppImage
    ARCH=x86_64 ./appimagetool-x86_64.AppImage AppDir RustyChatBox-x86_64.AppImage

# Ensure appimagetool is available
ensure-tools:
    #!/bin/bash
    if [ ! -f appimagetool-x86_64.AppImage ]; then
        wget https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage
        chmod +x appimagetool-x86_64.AppImage
    fi