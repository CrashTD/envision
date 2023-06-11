#!/bin/bash

set -ev

REPO_DIR=$1

"$(dirname -- "$0")/_clone_or_pull.sh" "https://github.com/cntools/libsurvive" "$REPO_DIR"

cd "$REPO_DIR"
mkdir -p build
cd build
cmake -DCMAKE_BUILD_TYPE=Release -DENABLE_api_example=OFF -DCMAKE_SKIP_INSTALL_RPATH=YES -Wno-dev ..
make clean
make -j$(nproc)
