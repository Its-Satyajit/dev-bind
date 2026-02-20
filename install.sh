#!/usr/bin/env bash

set -e

echo "🚀 Installing DevBind..."

# Ensure ~/.local/bin exists
mkdir -p ~/.local/bin

# Build the workspace in release mode
echo "📦 Building Release binaries (this may take a few minutes)..."
cargo build --release

# Copy binaries to ~/.local/bin
echo "📋 Copying binaries to ~/.local/bin..."
cp target/release/devbind-cli ~/.local/bin/devbind
cp target/release/devbind-gui ~/.local/bin/devbind-gui

# Ensure executable permissions
chmod +x ~/.local/bin/devbind
chmod +x ~/.local/bin/devbind-gui

# Allow binding to 443 without root (if setcap is available)
if command -v setcap &> /dev/null; then
    echo "🔐 Granting CAP_NET_BIND_SERVICE to DevBind CLI..."
    sudo setcap 'cap_net_bind_service=+ep' ~/.local/bin/devbind || echo "⚠️  Failed to setcap. You might need to run proxy via sudo for privileged ports."
else
    echo "⚠️  setcap not found. Binding to low ports (like 443) will require sudo."
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

echo "✅ DevBind is installed!"
echo "➡️  You can now run 'devbind' from your terminal or launch 'DevBind' from your app launcher."
