#!/bin/bash

set -ev

REPO_DIR=$1

PREFIX=$2

if [[ -z $REPO_DIR ]] || [[ -z $PREFIX ]]; then
    echo "Usage: $0 REPO_DIR PREFIX"
    exit 1
fi

"$(dirname -- "$0")/_clone_or_pull.sh" "https://github.com/cntools/libsurvive" "$REPO_DIR"

cd "$REPO_DIR"
mkdir -p build
cd build
cmake -DCMAKE_BUILD_TYPE=Release \
    -DENABLE_api_example=OFF \
    -DCMAKE_SKIP_INSTALL_RPATH=YES \
    -Wno-dev \
    -DCMAKE_INSTALL_PREFIX="${PREFIX}" \
    -DCMAKE_INSTALL_LIBDIR="${PREFIX}/lib" \
    ..
make clean
make -j$(nproc)
make install
