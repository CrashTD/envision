# Rex2

![](./data/icons/org.gabmus.rex2.svg)

UI for building, configuring and running Monado, the open source OpenXR runtime.

## Running

```bash
git clone https://gitlab.com/gabmus/rex2
cd rex2
meson setup build -Dprefix="$PWD/build/localprefix" -Dprofile=development
ninja -C build
ninja -C build install
./build/localprefix/bin/rex2
```

## Build AppImage

```bash
git clone https://gitlab.com/gabmus/rex2
cd rex2
./dist/appimage/build_appimage.sh
```
