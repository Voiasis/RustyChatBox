## RustyChatBox

### What:
RustyChatBox is a customizable VRChat Chatbox toolkit for Linux, heavily inspired by the popular [MagicChatBox](https://github.com/BoiHanny/vrcosc-magicchatbox).

### Why:
RustyChatBox was created to address the lack of widely available and decent VRChat OSCs on Linux.

### Who:
Currently, the RustyChatBox development team consists of only me, but I'm open to collaboration and contributions.

### Where:
The project is in its early stages and serves as a semi-functional proof-of-concept. Pre-built images will be released once the project is more stable. If you'd like to try it out, follow the steps below.

### Features: (wip)
- 

## Building & Running

### Prerequisites
To build RustyChatBox, you need the following tools and libraries installed:

- **Rust**: Install via [rustup](https://rustup.rs/) with:
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source $HOME/.cargo/env
    ```

- **just**: A command runner. Install with:
    ```bash
    cargo install just
    ```

- **System Libraries**: Required for GUI and other dependencies. Install based on your distribution:

  - **Fedora**:
    ```bash
    sudo dnf install -y \
        gtk3-devel \
        mesa-libEGL-devel \
        mesa-libGL-devel \
        libxkbcommon-devel \
        wayland-devel \
        zlib-devel \
        libpng-devel \
        dbus-devel \
        gdk-pixbuf2-devel \
        pango-devel \
        cairo-devel \
        fontconfig-devel \
        freetype-devel \
        libX11-devel \
        libXext-devel \
        libXau-devel \
        libffi-devel \
        git \
        playerctl \
        qt5ct
    ```

  - **Ubuntu/Debian**:
    ```bash
    sudo apt-get install -y \
        libgtk-3-dev \
        libegl1-mesa-dev \
        libgl1-mesa-dev \
        libxkbcommon-dev \
        libwayland-dev \
        zlib1g-dev \
        libpng-dev \
        libdbus-1-dev \
        libgdk-pixbuf2.0-dev \
        libpango1.0-dev \
        libcairo2-dev \
        libfontconfig1-dev \
        libfreetype6-dev \
        libx11-dev \
        libxext-dev \
        libxau-dev \
        libffi-dev \
        git \
        playerctl \
        qt5ct
    ```

  - **Arch**:
    ```bash
    sudo pacman -S --needed \
        gtk3 \
        mesa \
        libxkbcommon \
        wayland \
        zlib \
        libpng \
        dbus \
        gdk-pixbuf2 \
        pango \
        cairo \
        fontconfig \
        freetype2 \
        libx11 \
        libxext \
        libxau \
        libffi \
        git \
        playerctl \
        qt5ct
    ```

### Steps
1. Clone the repository:
    ```bash
    git clone https://github.com/Voiasis/RustyChatBox.git
    ```

2. Enter the directory:
    ```bash
    cd RustyChatBox
    ```

3. Build the project (this will compile the code and create an AppImage):
    ```bash
    just build
    ```

4. Run the AppImage:
    ```bash
    ./RustyChatBox.AppImage
    ```

**Troubleshooting**:
It *should* build the first time unless you run into a dependency issue.  
Try deleting the `AppDir` and `target` folders, then running `just build` again.

Let me know if you run into further issues or have suggestions here or on [Discord](https://discord.gg/kzYjRnppFn).

## License
This project is licensed under the [GPLv3 License](https://github.com/Voiasis/RustyChatBox?tab=GPL-3.0-1-ov-file#readme).