#!/usr/bin/env bash

declare -ra ROMS=(1-chip8-logo.ch8 2-ibm-logo.ch8 3-corax+.ch8 4-flags.ch8 7-beep.ch8)
declare -r BASE_URL="https://github.com/Timendus/chip8-test-suite/raw/refs/heads/main/bin"
declare -r BASE_DIR=$(dirname "$0")

for rom in "${ROMS[@]}"; do
    echo -n "Downloading ${rom} to ${BASE_DIR} ... "
    curl -s -L -o "${BASE_DIR}/${rom}" "${BASE_URL}/${rom}"
    echo "done."
done
