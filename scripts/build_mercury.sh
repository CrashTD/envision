#!/bin/bash

set -ev

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
