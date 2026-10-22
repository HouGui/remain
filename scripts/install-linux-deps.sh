#!/usr/bin/env bash
# Native build dependencies for Debian/Ubuntu development and CI.
set -euo pipefail
sudo apt-get update
sudo apt-get install -y \
    build-essential pkg-config libx11-dev libxi-dev libxkbcommon-dev \
    libwayland-dev libgl1-mesa-dev libegl1-mesa-dev
