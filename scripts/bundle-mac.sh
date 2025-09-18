#!/bin/bash
set -e

# Build macOS app bundle using cargo-bundle

echo "Building macOS app bundle..."

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Must be run from project root directory"
    exit 1
fi

# Install cargo-bundle if not present
if ! command -v cargo-bundle &> /dev/null; then
    echo "Installing cargo-bundle..."
    cargo install cargo-bundle
fi

# Clean previous builds
echo "Cleaning previous builds..."
rm -rf target/release/bundle/

# Build the app bundle
echo "Building app bundle..."
cargo bundle --release

# Check if bundle was created successfully
BUNDLE_PATH="target/release/bundle/osx/Chroma Tuner.app"
if [ ! -d "$BUNDLE_PATH" ]; then
    echo "Error: Bundle creation failed"
    exit 1
fi

echo "Bundle created at: $BUNDLE_PATH"

# Add microphone permission to Info.plist
echo "Adding microphone permission..."
PLIST_PATH="$BUNDLE_PATH/Contents/Info.plist"
if ! grep -q "NSMicrophoneUsageDescription" "$PLIST_PATH"; then
    /usr/libexec/PlistBuddy -c "Add :NSMicrophoneUsageDescription string 'Chroma Tuner needs microphone access to detect instrument pitch for tuning.'" "$PLIST_PATH" 2>/dev/null || true
    echo "Microphone permission added"
else
    echo "Microphone permission already present"
fi

# Create DMG for distribution (requires create-dmg)
if command -v create-dmg &> /dev/null; then
    echo "Creating DMG installer..."
    rm -f "Chroma Tuner.dmg"
    create-dmg \
        --volname "Chroma Tuner" \
        --window-pos 200 120 \
        --window-size 600 300 \
        --icon-size 100 \
        --icon "Chroma Tuner.app" 175 120 \
        --hide-extension "Chroma Tuner.app" \
        --app-drop-link 425 120 \
        "Chroma Tuner.dmg" \
        "$BUNDLE_PATH"
    echo "DMG created: Chroma Tuner.dmg"
else
    echo "Install 'create-dmg' (brew install create-dmg) to generate DMG installer"
fi

# Bundle info
echo ""
echo "Bundle Information:"
echo "  Path: $BUNDLE_PATH"
echo "  Size: $(du -sh "$BUNDLE_PATH" | cut -f1)"
echo "  Executable: $(file "$BUNDLE_PATH/Contents/MacOS/chroma-tuner" | cut -d: -f2-)"

echo ""
echo "macOS app bundle ready!"
echo "Drag '$BUNDLE_PATH' to your Applications folder"
