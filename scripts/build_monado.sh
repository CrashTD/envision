#!/bin/bash

# exit on error
# echo commands
set -ev

REPO_DIR=$1

if [[ -z $REPO_DIR ]]; then
    echo "Usage: $0 MONADO_REPO_DIR"
    exit 1
fi

if [[ -d "$REPO_DIR" ]]; then
    if [[ ! -d "$REPO_DIR/.git" ]]; then
        echo "Error: $REPO_DIR exists but is not a git repository"
        exit 1
    fi
    git -C "$REPO_DIR" pull
else
    git clone https://gitlab.freedesktop.org/monado/monado "$REPO_DIR"
fi

cd "$REPO_DIR"
mkdir -p build
cd build
cmake -DCMAKE_BUILD_TYPE=Release ..
make clean
make -j$(nproc)
