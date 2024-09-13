#!/bin/bash

echo "Preparing build directories."

# Remove old build (if exists)
rm -rf ./build

# Copy editor assets
mkdir -p ./build/editor/assets
cp -r ./assets/editor ./build/editor/assets/editor
cp -r ./assets/common ./build/editor/assets/common

# Copy player assets
mkdir -p ./build/player/assets
cp -r ./assets/player ./build/player/assets/player
cp -r ./assets/common ./build/player/assets/common

# Build Editor for linux
echo "Building Awgen Editor: Linux"
cargo build --features editor --release --target x86_64-unknown-linux-gnu
mv ./target/x86_64-unknown-linux-gnu/release/awgen ./build/editor/awgen_editor_linux

# Build Player for linux
echo "Building Awgen Player: Linux"
cargo build --release --target x86_64-unknown-linux-gnu
mv ./target/x86_64-unknown-linux-gnu/release/awgen ./build/player/awgen_player_linux

# Build Editor for windows
echo "Building Awgen Editor: Windows"
cargo build --features editor --release --target x86_64-pc-windows-gnu
mv ./target/x86_64-pc-windows-gnu/release/awgen.exe ./build/editor/awgen_editor_windows.exe

# Build Player for windows
echo "Building Awgen Player: Windows"
cargo build --release --target x86_64-pc-windows-gnu
mv ./target/x86_64-pc-windows-gnu/release/awgen.exe ./build/player/awgen_player_windows.exe
