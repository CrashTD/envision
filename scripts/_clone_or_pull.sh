#!/bin/bash

# exit on error
# echo commands
set -ev

GIT_URL=$1
REPO_DIR=$2

_usage() {
    echo "Usage: $0 GIT_URL REPO_DIR"
    exit 1
}

if [[ -z $REPO_DIR ]]; then
    _usage
fi

if [[ -z $GIT_URL ]]; then
    _usage
fi

if [[ -d "$REPO_DIR" ]]; then
    if [[ ! -d "$REPO_DIR/.git" ]]; then
        echo "Error: $REPO_DIR exists but is not a git repository"
        exit 1
    fi
    git -C "$REPO_DIR" pull
else
    git clone "$GIT_URL" "$REPO_DIR"  --recurse-submodules
fi
