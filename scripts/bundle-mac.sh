#!/bin/bash
set -e

# Simple macOS app bundle creation script
# Much lighter than cargo-bundle

APP_NAME="Chroma Tuner"
BUNDLE_ID="com.cinbarker.chroma-tuner"
VERSION="0.1.1"

echo "Creating macOS app bundle..."

# Build the binary first
cargo build --release

# Create app bundle structure
BUNDLE_DIR="target/release/$APP_NAME.app"
rm -rf "$BUNDLE_DIR"
mkdir -p "$BUNDLE_DIR/Contents/MacOS"
mkdir -p "$BUNDLE_DIR/Contents/Resources"

# Copy the binary
cp "target/release/chroma-tuner" "$BUNDLE_DIR/Contents/MacOS/"

# Copy the icon
cp "assets/icons/icon.icns" "$BUNDLE_DIR/Contents/Resources/"

# Create Info.plist
cat > "$BUNDLE_DIR/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>chroma-tuner</string>
    <key>CFBundleIdentifier</key>
    <string>$BUNDLE_ID</string>
    <key>CFBundleName</key>
    <string>$APP_NAME</string>
    <key>CFBundleDisplayName</key>
    <string>$APP_NAME</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleIconFile</key>
    <string>icon.icns</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.12</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSMicrophoneUsageDescription</key>
    <string>Chroma Tuner needs microphone access to detect instrument pitch for tuning.</string>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.utilities</string>
</dict>
</plist>
EOF

echo "App bundle created at: $BUNDLE_DIR"
echo "Size: $(du -sh "$BUNDLE_DIR" | cut -f1)"

# Test that it works
if [[ -f "$BUNDLE_DIR/Contents/MacOS/chroma-tuner" ]]; then
    echo "✅ Bundle created successfully!"
else
    echo "❌ Bundle creation failed!"
    exit 1
fi
