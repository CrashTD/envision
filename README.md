# Envision

# IMPORTANT NOTES!

This is still highly experimental software, while it's unlikely that anything bad will happen, it's still unstable and there is no guarantee that it will work on your system, with your particular hardware. If you encounter any problems while using the app, make sure to open an issue.

Also consider that due to the unstable nature of the app, it's possible to encounter unexpected behavior that while in VR might cause motion sickness or physical injury. **Be very careful while in VR using this app!**

---

![](./data/icons/org.gabmus.envision.svg)

UI for building, configuring and running Monado, the open source OpenXR runtime.

Download the latest AppImage snapshot: [GitLab Pipelines](https://gitlab.com/gabmus/envision/-/pipelines)

## Running

```bash
git clone https://gitlab.com/gabmusenvision/
cdenvision 
meson setup build -Dprefix="$PWD/build/localprefix" -Dprofile=development
ninja -C build
ninja -C build install
./build/localprefix/bin/envision
```

## Build AppImage

```bash
git clone https://gitlab.com/gabmus/envision
cd envision
./dist/appimage/build_appimage.sh
```

# Feature flags

|Env var|Values|Default|
|---|---|---|
|`ENVISION_FF_USE_LIBMONADO`|`1`: enabled; `0`: disabled|`0`|

# Common issues

## NOSUID with systemd-homed

If you see this warning:

> Your current prefix is inside a partition mounted with the nosuid option. This will prevent the Envision runtime from acquiring certain privileges and will cause noticeable stutter when running XR applications.

And you're using systemd-homed to manage your home partition, you need to disable it using homectl. To do so, log out, log in as root in a tty and run: `homectl update <username> --nosuid=false`.
