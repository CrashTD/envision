#!/bin/bash

# exit on error
# echo commands
set -ev

REPO_DIR=$1

PREFIX=$2

DO_PULL=$3

REPO_URL=$4

if [[ -z $REPO_DIR ]] || [[ -z $PREFIX ]] || [[ -z $DO_PULL ]]; then
    echo "Usage: $0 REPO_DIR PREFIX DO_PULL [REPO_URL]"
    exit 1
fi

if [[ -z $REPO_URL ]]; then
    REPO_URL="https://github.com/Meumeu/WiVRn"
fi

"$(dirname -- "$0")/_clone_or_pull.sh" "$REPO_URL" "$REPO_DIR" "$DO_PULL"

cd "$REPO_DIR"
rm -rf build
mkdir -p build
cd build
cmake -B build-server -DCMAKE_BUILD_TYPE=Release \
    -DWIVRN_BUILD_CLIENT=OFF \
    -DCMAKE_INSTALL_PREFIX="${PREFIX}" \
    .. -GNinja
cmake --build build-server
ninja -C build-server install
