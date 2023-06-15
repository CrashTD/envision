#!/bin/bash

# exit on error
# echo commands
set -ev

REPO_DIR=$1

"$(dirname -- "$0")/_clone_or_pull.sh" "https://gitlab.com/znixian/OpenOVR.git" "$REPO_DIR"

cd "$REPO_DIR"
mkdir -p build
cd build
cmake -DCMAKE_BUILD_TYPE=Release ..
make clean
make -j$(nproc)
make install
