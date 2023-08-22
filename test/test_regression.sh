#!/bin/bash
set -ex

TARGET_DIR="$(pwd)/powdr"
if [ ! -d "$TARGET_DIR" ]; then
    git clone https://github.com/powdr-labs/powdr.git "$TARGET_DIR"
    cd powdr
    # Install powdr_cli
    cargo install --path ./powdr_cli
    cd ..
fi
# Test regression
powdr rust regression -o ./test_regression -f