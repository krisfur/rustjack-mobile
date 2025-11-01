#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

echo "🎰 Building Rustjack for Android..."

# Check if xbuild is installed
if ! command -v x &> /dev/null; then
    echo "📦 Installing xbuild..."
    cargo install xbuild
fi

# Add Android target
echo "📱 Adding Android target..."
rustup target add aarch64-linux-android

# --- MODIFIED BUILD STEP ---
# Build APK using xbuild
echo "🔨 Building DEBUG APK for testing..."
# We are removing the --release flag.
# The --release flag builds an Android App Bundle (.aab) for the Play Store.
# For local testing on a device, we want a normal Debug APK (.apk).
x build --platform android --arch arm64

echo "✅ Build complete!"

# --- MODIFIED INSTALL SCRIPT ---

# 1. Define the directory for the DEBUG APK
APK_DIR="target/x/debug/android"
echo "📦 Searching for APK in: $APK_DIR"

# 2. Find the APK file.
# This command finds the first file ending in .apk in that directory.
APK_FILE=$(find "$APK_DIR" -name "*.apk" -print -quit)

# 3. Check if we found an APK
if [ -z "$APK_FILE" ]; then
    echo "❌ Error: Could not find an APK file in $APK_DIR"
    echo "Please check the build output to confirm the APK location."
    exit 1
fi

echo "Found APK: $APK_FILE"

# 4. Optional: Install to connected device

# --- MODIFICATION: Start ADB server *before* prompting ---
# This fixes a race condition where the ADB daemon doesn't
# find the device in time for the 'adb install' command.

echo "📲 Starting ADB server and looking for devices..."
# Explicitly start the server
adb start-server
# Give it a couple of seconds to find USB devices
sleep 2

echo "Connected devices:"
# Show connected devices so the user can see it's working
adb devices
# --- End of modification ---

read -p "Install $APK_FILE to a connected device? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "Installing $APK_FILE..."

    # 5. Use 'adb install -r'
    # The '-r' flag means "reinstall", which will replace the
    # existing application if it's already installed.
    adb install -r "$APK_FILE"

    if [ $? -eq 0 ]; then
        echo "✅ Install successful!"
        echo "You can now open the app on your device."
    else
        echo "❌ Install failed. Check the output from adb above."
    fi
fi

