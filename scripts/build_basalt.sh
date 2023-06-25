#!/bin/bash

set -ev

echo "Basalt is currently unsupported"
exit 1
#
# REPO_DIR=$1
#
# PREFIX=$2
#
# DO_PULL=$3
#
# REPO_URL=$4
#
# if [[ -z $REPO_DIR ]] || [[ -z $PREFIX ]] || [[ -z $DO_PULL ]]; then
#     echo "Usage: $0 REPO_DIR PREFIX DO_PULL [REPO_URL]"
#     exit 1
# fi
#
# if [[ -z $REPO_URL ]]; then
#     REPO_URL="https://gitlab.freedesktop.org/mateosss/basalt"
# fi
#
# "$(dirname -- "$0")/_clone_or_pull.sh" "$REPO_URL" "$REPO_DIR" "$DO_PULL"
#
# cd "$REPO_DIR"
# rm -rf build
# mkdir -p build
# cd build
