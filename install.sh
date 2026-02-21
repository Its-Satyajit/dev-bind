#!/usr/bin/env bash

set -e

echo "[START] Installing DevBind..."

# Ensure ~/.local/bin exists
mkdir -p ~/.local/bin

# Build the workspace in release mode
echo "[PKG] Building Release binaries (this may take a few minutes)..."
cargo build --release

if systemctl --user is-active --quiet devbind.service 2>/dev/null; then
    echo "[INFO] Stopping active DevBind background service..."
    systemctl --user stop devbind.service || true
    WAS_RUNNING=1
else
    WAS_RUNNING=0
fi

if pgrep -x devbind >/dev/null || pgrep -x devbind-gui >/dev/null; then
    echo "[INFO] Terminating running DevBind processes..."
    pkill -x devbind || true
    pkill -x devbind-gui || true
    sleep 1 # wait for processes to exit
fi



# Copy binaries to ~/.local/bin
echo "[INFO] Copying binaries to ~/.local/bin..."
rm -f ~/.local/bin/devbind ~/.local/bin/devbind-gui
cp target/release/devbind-cli ~/.local/bin/devbind
cp target/release/devbind-gui ~/.local/bin/devbind-gui

if [ "$WAS_RUNNING" -eq 1 ]; then
    echo "[INFO] Restarting DevBind background service..."
    systemctl --user start devbind.service || true
fi

# Ensure executable permissions
chmod +x ~/.local/bin/devbind
chmod +x ~/.local/bin/devbind-gui

# Allow binding to 443 without root (if setcap is available)
if command -v setcap &> /dev/null; then
    echo "[SEC] Granting CAP_NET_BIND_SERVICE to DevBind CLI..."
    sudo setcap 'cap_net_bind_service=+ep' ~/.local/bin/devbind || echo "[WARN]  Failed to setcap. You might need to run proxy via sudo for privileged ports."
else
    echo "[WARN]  setcap not found. Binding to low ports (like 443) will require sudo."
fi

# Optional: Create a .desktop file so it appears in app menus
DESKTOP_FILE="$HOME/.local/share/applications/devbind.desktop"
mkdir -p "$HOME/.local/share/applications"

cat > "$DESKTOP_FILE" << EOF
[Desktop Entry]
Name=DevBind
Comment=Local Dev SSL Reverse Proxy
Exec=$HOME/.local/bin/devbind-gui
Icon=utilities-terminal
Terminal=false
Type=Application
Categories=Development;Utility;
EOF

echo "[OK] DevBind is installed!"
echo "[GO]  You can now run 'devbind' from your terminal or launch 'DevBind' from your app launcher."
