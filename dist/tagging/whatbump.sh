#!/bin/bash

set -e

if [ ! "$(command -v what-bump)" ]; then
    >&2 cargo install --git https://github.com/sky-uk/what-bump
fi
if [ ! "$(command -v what-bump)" ]; then
    >&2 echo "Error: tried installing what-bump, but command is still not available"
    exit 1
fi

OLD_TAG="$(git describe --tags --abbrev=0 || true)"

if [ ! "$OLD_TAG" ]; then
    >&2 echo "No prior tag found, this is the first tag"
    what-bump HEAD --from 0.0.1 \
        -c ./changelog_metainfo.xml \
        -t ./dist/tagging/metainfo_template.xml
else
    rm ./changelog_metainfo.xml || true
    what-bump "$OLD_TAG" --from "$OLD_TAG" \
        -c ./changelog_metainfo.xml \
        -t ./dist/tagging/metainfo_template.xml
fi
