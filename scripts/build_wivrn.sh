#!/bin/bash

# exit on error
# echo commands
set -ev

REPO_DIR=$1

PREFIX=$2

if [[ -z $REPO_DIR ]] || [[ -z $PREFIX ]]; then
    echo "Usage: $0 REPO_DIR PREFIX"
    exit 1
fi

"$(dirname -- "$0")/_clone_or_pull.sh" "https://github.com/Meumeu/WiVRn" "$REPO_DIR"

cd "$REPO_DIR"
mkdir -p build
cd build
cmake -B build-server -DCMAKE_BUILD_TYPE=Release \
    -DWIVRN_BUILD_CLIENT=OFF \
    -DCMAKE_INSTALL_PREFIX="${PREFIX}" \
    .. -GNinja
cmake --build build-server
ninja install
