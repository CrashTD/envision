#!/bin/bash

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
    REPO_URL="https://github.com/cntools/libsurvive"
fi

"$(dirname -- "$0")/_clone_or_pull.sh" "$REPO_URL" "$REPO_DIR" "$DO_PULL"

cd "$REPO_DIR"
rm -rf build
mkdir -p build
cd build
cmake -DCMAKE_BUILD_TYPE=Release \
    -DENABLE_api_example=OFF \
    -DCMAKE_SKIP_INSTALL_RPATH=YES \
    -Wno-dev \
    -DCMAKE_INSTALL_PREFIX="${PREFIX}" \
    -DCMAKE_INSTALL_LIBDIR="${PREFIX}/lib" \
    ..
make -j$(nproc)
make install
