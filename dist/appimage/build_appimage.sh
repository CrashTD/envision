#!/bin/bash

set -e

if [[ ! -f Cargo.toml ]]; then
    echo "Please run this script from the repo root"
    exit 1
fi

meson setup appimage_build -Dprefix=/usr -Dprofile=default
meson test -C appimage_build --print-errorlogs
DESTDIR="$PWD/AppDir" ninja -C appimage_build install
curl -SsLO https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
chmod +x linuxdeploy-x86_64.AppImage
cp dist/appimage/linuxdeploy-plugin-gtk.sh ./

if [ -f "AppDir/usr/share/icons/hicolor/scalable/apps/org.gabmus.envision.Devel.svg" ]; then
    APPID="org.gabmus.envision.Devel"
else
    APPID="org.gabmus.envision"
fi

./linuxdeploy-x86_64.AppImage \
    --appimage-extract-and-run \
    --appdir AppDir \
    --plugin gtk \
    --output appimage \
    --icon-file AppDir/usr/share/icons/hicolor/scalable/apps/$APPID.svg \
    --desktop-file AppDir/usr/share/applications/$APPID.desktop

rm ./linuxdeploy-plugin-gtk.sh
