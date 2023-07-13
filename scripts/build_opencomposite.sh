#!/bin/bash

# exit on error
# echo commands
set -ev

REPO_DIR=$1

DO_PULL=$2

REPO_URL=$3

if [[ -z $REPO_DIR ]] || [[ -z $DO_PULL ]]; then
    echo "Usage: $0 REPO_DIR DO_PULL [REPO_URL]"
    exit 1
fi

if [[ -z $REPO_URL ]]; then
    REPO_URL="https://gitlab.com/znixian/OpenOVR.git"
fi

"$(dirname -- "$0")/_clone_or_pull.sh" "$REPO_URL" "$REPO_DIR" "$DO_PULL"

cd "$REPO_DIR"
rm -rf build
mkdir -p build
cd build
cmake -DCMAKE_BUILD_TYPE=Release .. -G Ninja
cmake --build .
