#!/bin/bash

set -e

if [[ ! -f Cargo.toml ]]; then
    echo "Please run this script from the repo root"
    exit 1
fi

meson setup appimage_build -Dprefix=/usr -Dprofile=development
DESTDIR="$PWD/AppDir" ninja -C appimage_build install
curl -SsLO https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
cp dist/appimage/linuxdeploy-plugin-gtk.sh ./

./linuxdeploy-x86_64.AppImage \
    --appdir AppDir \
    --plugin gtk \
    --output appimage \
    --icon-file AppDir/usr/share/icons/hicolor/scalable/apps/org.gabmus.rex2.Devel.svg \
    --desktop-file AppDir/usr/share/applications/org.gabmus.rex2.Devel.desktop

rm ./linuxdeploy-plugin-gtk.sh
