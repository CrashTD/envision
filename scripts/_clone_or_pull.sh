#!/bin/bash

# exit on error
# echo commands
set -ev

GIT_URL=$1
REPO_DIR=$2
DO_PULL=$3

_usage() {
    echo "Usage: $0 GIT_URL REPO_DIR [DO_PULL]"
    exit 1
}

if [[ -z $REPO_DIR ]]; then
    _usage
fi

if [[ -z $GIT_URL ]]; then
    _usage
fi

if [[ -z $DO_PULL ]]; then
    DO_PULL=1
fi

if [[ -d "$REPO_DIR" ]]; then
    if [[ ! -d "$REPO_DIR/.git" ]]; then
        rmdir "$REPO_DIR"
        git clone "$GIT_URL" "$REPO_DIR"  --recurse-submodules
    else
        if [[ "$DO_PULL" -eq 1 ]]; then
            git -C "$REPO_DIR" pull
        fi
    fi
else
    git clone "$GIT_URL" "$REPO_DIR"  --recurse-submodules
fi
