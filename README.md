# Rex2

![](./data/icons/org.gabmus.rex2.svg)

UI for building, configuring and running Monado, the open source OpenXR runtime.

Download the latest AppImage snapshot: [GitLab Pipelines](https://gitlab.com/gabmus/rex2/-/pipelines)

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

# Common issues

## NOSUID with systemd-homed

If you see this warning:

> Your current prefix is inside a partition mounted with the nosuid option. This will prevent the Rex2 runtime from acquiring certain privileges and will cause noticeable stutter when running XR applications.

And you're using systemd-homed to manage your home partition, you need to disable it using homectl. To do so, log out, log in as root in a tty and run: `homectl update <username> --nosuid=false`.
