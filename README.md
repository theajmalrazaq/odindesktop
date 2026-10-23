# Odin Desktop

A **Linux desktop application** for Samsung firmware flashing, built with Tauri 2, React, TypeScript and Rust around [Llucs/odin4](https://github.com/Llucs/odin4). Install it, open it from your application menu, and connect your phone by USB. The interface, firmware processing and USB operations all run on your computer.

## Download and install

Download packages from [GitHub Releases](https://github.com/theajmalrazaq/odinweb/releases). The release pipeline publishes a prerelease only after native builds and packaged application checks pass. These first releases still need testing with real Samsung phones.

Packages currently target **x86_64** computers. ARM64 installers are not included.

| Distribution | Package | Install |
| --- | --- | --- |
| Ubuntu 24.04+, compatible Debian systems | `.deb` | `sudo apt install ./odin-desktop_0.1.1_amd64.deb` |
| Arch Linux / compatible Arch derivatives | `.pkg.tar.zst` | `sudo pacman -U ./odin-desktop-bin-0.1.1-1-x86_64.pkg.tar.zst` |
| Fedora / compatible RPM systems | `.rpm` | `sudo dnf install ./odin-desktop-0.1.1-1.x86_64.rpm` |
| Other compatible Linux desktops | `.AppImage` | `chmod +x ./odin-desktop-0.1.1-x86_64.AppImage` then run the file |

The binaries are built on Ubuntu 24.04 and require **glibc 2.39 or newer**. Native packages also need their distribution's GTK 3 and WebKitGTK 4.1 libraries. AppImage does not make older glibc versions compatible; some systems also need FUSE 2. RPM installation on every Fedora derivative is not independently verified. For older distributions, build from source on that system with GCC 14 or newer.

Verify the download using the release's `SHA256SUMS`. Native packages contain USB permissions rules. AppImage users should download `60-odin-desktop.rules` from the same release and install it:

```sh
sudo install -Dm644 60-odin-desktop.rules /etc/udev/rules.d/60-odin-desktop.rules
sudo udevadm control --reload-rules
sudo udevadm trigger
```

After installing any package, reload udev rules and reconnect your phone. The rules grant access to the active local desktop session through `uaccess`; they do not require an Ubuntu-specific `plugdev` group. Run the application as your normal user.

The Arch package is downloadable directly; it is not published to the AUR or official Arch repositories. The release includes a `PKGBUILD` with the actual Debian source-package checksum.

## Using the application

1. Back up your data and obtain firmware for the exact model and bootloader revision.
2. Enter Samsung Download Mode, connect via USB and click **Scan devices**.
3. Select the device and matching BL, AP, CP, CSC and/or UMS archives.
4. Click **Validate firmware** and review the Odin4 log.
5. Click **Start flashing**, review the confirmation and keep the cable connected until completion.

Odin4 performs archive and partition checks. A successful validation is required before flashing, and SHA-256 fingerprints are checked again before writes. Do not change firmware files or swap devices during an operation: a USB path identifies a connection, not a permanent device identity. The application serializes operations and blocks ordinary window closure while an operation is running.

Logs are shown in the application and can be exported. Native engine logs are stored in the application's platform log directory. The engine is bundled with the installer, so no separate Odin4 installation is needed.

There is no mid-flash cancellation control. Incorrect firmware and interrupted flashing can erase data or make a phone unbootable. A `.tar.md5` checksum is an integrity check, not proof of authenticity. The GUI uses upstream Odin4's validation and protocol implementation; it does not independently guarantee firmware/model compatibility.

## Build from source

Use the existing checkout; each cloud task is already isolated and does not need a worktree. Install Node.js 24 and stable Rust, then your system's development libraries.

Ubuntu 24.04:

```sh
sudo apt-get update
sudo apt-get install -y g++-14 gcc-14 cmake ninja-build pkg-config \
  libusb-1.0-0-dev libcrypto++-dev libgtk-3-dev libwebkit2gtk-4.1-dev \
  librsvg2-dev patchelf rpm
```

Arch Linux:

```sh
sudo pacman -S --needed base-devel cmake ninja pkgconf nodejs npm rust \
  libusb crypto++ gtk3 webkit2gtk-4.1 librsvg patchelf
```

From the repository root:

```sh
npm ci
# Ubuntu: use GCC 14 explicitly. On Arch, use the installed GCC >= 14.
CC=gcc-14 CXX=g++-14 npm run engine:build
npm run tauri dev
```

On Arch, replace the engine command with `npm run engine:build`. This fetches pinned upstream revision `b8630fb39624c76cdce0e196c57f7285d63286d3`, compiles the CLI with its Qt interface disabled, runs upstream tests and prepares Tauri's bundled engine. The script refuses to overwrite modified upstream sources.

### Checks and installers

```sh
npm run build
cargo test --manifest-path scripts/core-tests/Cargo.toml --locked
cargo test --manifest-path src-tauri/Cargo.toml --locked
npm run tauri -- build -- --locked
```

Packages are placed in `src-tauri/target/release/bundle/`. The engine's Apache-2.0 license and NOTICE are included. UI automation uses Chromium with a mocked native bridge; start `npm run dev` in a separate terminal and run `npm test`. This checks UI behavior, not physical USB hardware.

The GitHub Actions workflow builds `.deb`, `.rpm` and AppImage, launches the installed Debian app and its bundled engine under a virtual display, repackages and checks the Arch installation, and creates checksums. A version tag such as `v0.1.1` publishes the packages only after these jobs pass. Regular pushes build artifacts without publishing a release.
