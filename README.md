# Envision

# IMPORTANT NOTES!

This is still highly experimental software, while it's unlikely that anything bad will happen, it's still unstable and there is no guarantee that it will work on your system, with your particular hardware. If you encounter any problems while using the app, make sure to open an issue.

Also consider that due to the unstable nature of the app, it's possible to encounter unexpected behavior that while in VR might cause motion sickness or physical injury. **Be very careful while in VR using this app!**

---

![](./data/icons/org.gabmus.envision.svg)

UI for building, configuring and running Monado, the open source OpenXR runtime.

Download the latest AppImage snapshot: [GitLab Pipelines](https://gitlab.com/gabmus/envision/-/pipelines?page=1&scope=all&ref=main)

<details>
<summary>

## Dependencies

</summary>

### Arch

You can find the Arch dependencies in the `PKGBUILD` in `dist/arch/PKGBUILD`.

### Debian/Ubuntu

You can find the Debian dependencies in the `.gitlab-ci.yml` file in this repository, in particular you can find the install command used by the CI in the `appimage` stage.

</details>



## Building and running from source


```bash
git clone https://gitlab.com/gabmus/envision/
cd envision
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

# Common issues

## NOSUID with systemd-homed

If you see this warning:

> Your current prefix is inside a partition mounted with the nosuid option. This will prevent the Envision runtime from acquiring certain privileges and will cause noticeable stutter when running XR applications.

And you're using systemd-homed to manage your home partition, you need to disable it using homectl. To do so, log out, log in as root in a tty and run: `homectl update <username> --nosuid=false`.

## Dependency checker erroneously reports missing dependencies

You can use the `--skip-dependency-check` option to disable dependency checking.

# LVRA Community

We're a community dedicated to VR on Linux, we've got a wiki you might want to check out: [LVRA Wiki](https://lvra.gitlab.io). You can also [chat with us](https://lvra.gitlab.io/docs/community/).
