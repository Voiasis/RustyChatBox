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
 printf '#!/bin/bash\nHERE="$(dirname "$(readlink -f "${0}")")"\nLD_LIBRARY_PATH="${HERE}/usr/lib" PATH="${HERE}/usr/bin:$PATH" exec "${HERE}/usr/bin/rustychatbox" "$@"\n' > AppDir/AppRun
 chmod +x AppDir/AppRun
 # Copy required libraries using ldd
 ldd target/release/rustychatbox | grep "=> /" | awk '{print $3}' | xargs -I {} cp {} AppDir/usr/lib/ || true
 # Copy transitive dependencies for bundled libraries
 /bin/bash -c 'for lib in AppDir/usr/lib/*.so*; do ldd "$$lib" | grep "=> /" | awk "{print $$3}" | xargs -I {} cp {} AppDir/usr/lib/ 2>/dev/null || true; done'
 # Copy playerctl binary if available
 if command -v playerctl >/dev/null 2>&1; then \
 cp $(which playerctl) AppDir/usr/bin/ 2>/dev/null || true; \
 echo "playerctl binary copied to AppDir/usr/bin"; \
 else \
 echo "Warning: playerctl binary not found on system. MediaLink functionality may not work."; \
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
 # Copy lshw-related libraries (e.g., libpci.so.3)
 cp /usr/lib/libpci.so.3 AppDir/usr/lib/ 2>/dev/null || true
 cp /usr/lib64/libpci.so.3 AppDir/usr/lib/ 2>/dev/null || true
 cp /lib64/libpci.so.3 AppDir/usr/lib/ 2>/dev/null || true
 cp /usr/lib/x86_64-linux-gnu/libpci.so.3 AppDir/usr/lib/ 2>/dev/null || true
 # Copy libraries required by eframe/egui
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
 cp /usr/lib/x86_64-linux-gnu/libdbus-1.so. 3 AppDir/usr/lib/ 2>/dev/null || true
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
 # Create AppImage
 ARCH=x86_64 ./appimagetool-x86_64.AppImage AppDir RustyChatBox-x86_64.AppImage

# Ensure appimagetool is available
ensure-tools:
 #!/bin/bash
 if [ ! -f appimagetool-x86_64.AppImage ]; then
 wget https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage
 chmod +x appimagetool-x86_64.AppImage
 fi