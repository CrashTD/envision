# Envision

> **Fork note (CrashTD, 2026-10-02):** This is a modified copy of
> [Envision](https://gitlab.com/gabmus/envision) (AGPL-3.0-or-later). Branch
> `openvr-compat-disabled` adds an OpenVR compatibility option **"Disabled"**: Envision then
> neither builds nor manages an OpenVR compatibility module and leaves
> `openvrpaths.vrpath` alone, so an externally installed OpenVR client (e.g. OpenComposite
> Unleashed or a custom xrizer build) keeps working. Changes: `src/profile.rs`,
> `src/ui/app.rs`. Everything else is upstream.
>
> Not a coder – Claude wrote most of the patch, I tested it on my setup. Use at your own risk.

# IMPORTANT NOTES!

This is still highly experimental software, while it's unlikely that anything bad will happen, it's still unstable and there is no guarantee that it will work on your system, with your particular hardware. If you encounter any problems while using the app, make sure to open an issue.

Also consider that due to the unstable nature of the app, it's possible to encounter unexpected behavior that while in VR might cause motion sickness or physical injury. **Be very careful while in VR using this app!**

## Do you have a standalone/android based headset?

Envision is built for PCVR headsets only at this moment, the recommended tool for standalone headsets is [WiVRn](https://github.com/WiVRn/WiVRn/).

---

![](./data/icons/org.gabmus.envision.svg)

Orchestrator for the free XR stack.

# Download Envision

- You can download the latest [AppImage snapshot](https://gitlab.com/gabmus/envision/-/pipelines?ref=main&status=success).
- Once downloaded, unzip the archive and you should find a file named `Envision-x86_64.AppImage`.
- Make it executable
  - From your file manager: right click > properties > "executable as program", or under permissions enable "is executable" (this depends on your file manager)
  - From the terminal: `chmod +x Envision-x86_64.AppImage`
- Double click the AppImage file to run it.

<details>
<summary>

# Build instructions (for package maintainers and advanced users only)

</summary>

## Dependencies

### Arch

You can find the Arch dependencies in the `PKGBUILD` in `dist/arch/PKGBUILD`.

### Debian/Ubuntu

You can find the Debian dependencies in the `.gitlab-ci.yml` file in this repository, in particular you can find the install command used by the CI in the `appimage` stage.

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

</details>

# XR Plugin Spec

Envision supports running other applications as so called plugins. These plugins will be auto discovered via desktop files with specific properties.

This section describes these properties.

| Property | Description | Type | Possible values | Default |
|----------|-------------|------|-----------------|---------|
| `X-XR-Plugin` | Marks the desktop file as an XR plugin, making it discoverable by Envision | boolean | `true`, `false` | `false` |
| `X-XR-Plugin-Exec` | Will be used instead of `Exec` if present when launched by Envision. If neither `Exec` or `X-XR-Plugin-Exec` are set, the plugin will be skipped | string | A valid command | The value of `Exec` if present |
| `X-XR-Plugin-Execution-Time` | Defines when the plugin will be executed | string | `after-start` will start the plugin after the XR service starts; `after-stop` will start and detach the plugin after the XR service stops (useful for cleanup operations like turning off lighthouses) | `after-start` |

# Debugging

To view all the logs you need to run envision with the env var `RUST_LOG=trace`.

# Common issues

## NOSUID with systemd-homed

If you see this warning:

> Your current prefix is inside a partition mounted with the nosuid option. This will prevent the Envision runtime from acquiring certain privileges and will cause noticeable stutter when running XR applications.

And you're using systemd-homed to manage your home partition, you need to disable it using homectl. To do so, log out, log in as root in a tty and run: `homectl update <username> --nosuid=false`.

## Dependency checker erroneously reports missing dependencies

You can use the `--skip-dependency-check` option to disable dependency checking.

# LVRA Community

We're a community dedicated to VR on Linux, we've got a wiki you might want to check out: [LVRA Wiki](https://lvra.gitlab.io). You can also [chat with us](https://lvra.gitlab.io/docs/community/).
