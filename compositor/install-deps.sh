#!/bin/bash

echo "Installing Hanami Compositor dependencies..."
echo "=============================================="
echo ""

# Update package list
echo "Updating package list..."
sudo apt update

# Install build tools
echo ""
echo "Installing build tools..."
sudo apt install -y build-essential meson ninja-build pkg-config

# Install wlroots and dependencies
echo ""
echo "Installing wlroots and Wayland dependencies..."
sudo apt install -y \
    libwlroots-dev \
    libwayland-dev \
    wayland-protocols \
    libxkbcommon-dev \
    libinput-dev \
    libdrm-dev \
    libgbm-dev \
    libpixman-1-dev \
    libudev-dev \
    libseat-dev

echo ""
echo "=============================================="
echo "Dependencies installed successfully!"
echo ""
echo "You can now build the compositor with:"
echo "  cd compositor"
echo "  make"
