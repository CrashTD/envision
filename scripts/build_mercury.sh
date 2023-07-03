#!/bin/bash

set -ev

PREFIX=$1

CACHE_DIR=$2

if [[ -z $PREFIX ]] || [[ -z $CACHE_DIR ]]; then
    echo "Usage: $0 PREFIX CACHE_DIR"
    exit 1
fi

ONNX_VER=$(curl -sSL "https://api.github.com/repos/microsoft/onnxruntime/releases/latest" | jq -r .tag_name | tr -d v)
SYS_ARCH=$(uname -m)

if [[ $SYS_ARCH == x*64 ]]; then
    ARCH="x64"
elif [[ $SYS_ARCH == arm64 ]] || [[ $ARCH == aarch64 ]]; then
    ARCH="aarch64"
else
    echo "CPU architecture '$SYS_ARCH' is not supported"
    exit 1
fi

ONNX="onnxruntime-linux-${ARCH}-${ONNX_VER}"
ONNX_URL="https://github.com/microsoft/onnxruntime/releases/download/v${ONNX_VER}/${ONNX}.tgz"

mkdir -p "$CACHE_DIR"

curl -sSL "$ONNX_URL" -o "${CACHE_DIR}/onnxruntime.tgz"

tar xf "${CACHE_DIR}/onnxruntime.tgz" --directory="${CACHE_DIR}"

mkdir -p "${PREFIX}/lib"
mkdir -p "${PREFIX}/include"

cp -r "${CACHE_DIR}/${ONNX}/include/"* "${PREFIX}/include/"
cp -r "${CACHE_DIR}/${ONNX}/lib/"* "${PREFIX}/lib/"

if [[ -z $XDG_DATA_HOME ]]; then
    DATA_HOME=$HOME/.local/share
else
    DATA_HOME=$XDG_DATA_HOME
fi

if [[ ! -d "$DATA_HOME/monado/hand-tracking-models" ]]; then
    git clone "https://gitlab.freedesktop.org/monado/utilities/hand-tracking-models" "$DATA_HOME/monado/hand-tracking-models"
    # Some weird distros aren't configured to automagically do the LFS things; do them just in case.
    cd "$DATA_HOME/monado/hand-tracking-models"
    git lfs install
    git lfs fetch
    git lfs pull
fi

cd "$DATA_HOME/monado/hand-tracking-models"
git pull
