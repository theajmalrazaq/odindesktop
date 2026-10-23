# Odin Desktop

A Linux desktop firmware workspace built with **Tauri 2, React 19, TypeScript, and Rust**, wrapping [Llucs/odin4](https://github.com/Llucs/odin4). All USB communication and firmware processing happen locally. The browser build is an interface preview; it cannot access the native engine.

## What is implemented

- Native Download Mode device discovery and explicit device selection.
- Native file dialogs for BL, AP, CP, CSC and UMS `.tar` / `.tar.md5` archives.
- Odin4 `--check-only` validation against the connected device.
- Flash confirmation and optional reboot after a successful operation.
- Backend-enforced validation before flashing. SHA-256 fingerprints bind validation to the selected file contents, paths, slots and USB device path. Files are hashed again before flashing; a failed attempt invalidates the receipt. Do not modify files or swap devices during an operation. USB paths identify connections, not a phone's permanent identity.
- One native engine operation at a time, streamed stdout/stderr, bounded visible logs and log export.
- No shell execution of frontend input, arbitrary engine paths, EFS erase, forced repartitioning, or unknown-partition overrides.

**Status:** frontend production build, browser interaction checks and four Rust operation tests pass in the development container. Full Tauri compilation and native engine compilation require Linux development packages unavailable in this read-only container. No installer or real-device flashing has been verified. The GitHub workflow builds and uploads a `.deb` when run; it has not been run here. Linux is the only supported target; Windows/macOS support is not implied by Tauri's cross-platform shell.

## Linux development

Use the existing checkout; each cloud task is already isolated and does not need a worktree.

On Ubuntu 24.04 (or equivalent packages on Debian 13):

```sh
sudo apt-get update
sudo apt-get install -y g++-14 gcc-14 cmake ninja-build pkg-config \
  libusb-1.0-0-dev libcrypto++-dev libgtk-3-dev libwebkit2gtk-4.1-dev \
  librsvg2-dev patchelf
```

Install Node.js 24 and stable Rust using their official distributions. Then, from the repository root:

```sh
npm ci
CC=gcc-14 CXX=g++-14 npm run engine:build
npm run tauri dev
```

The engine script fetches pinned upstream revision `b8630fb39624c76cdce0e196c57f7285d63286d3`, disables its Qt GUI, compiles the CLI and runs its existing tests. It places the executable under `src-tauri/binaries/odin4-<host-triple>`, as required by Tauri's sidecar bundler. It refuses to overwrite a modified engine source checkout. Native Odin4 logs are written under the application's platform log directory, not the source tree. Allow enough disk space for both C++ and Rust builds.

### USB permissions

Inspect and install the upstream udev rules appropriate to your machine:

```sh
cat .cache/odin4/udev/60-odin4.rules
sudo install -m 0644 .cache/odin4/udev/60-odin4.rules /etc/udev/rules.d/60-odin4.rules
sudo udevadm control --reload-rules
sudo udevadm trigger
```

Reconnect your device, in Samsung Download Mode. Follow upstream group/session requirements if present. Run the GUI as a normal user. Close other flash tools claiming the USB interface. The app does not silently elevate privileges or install system rules.

### Checks and packaging

```sh
npm run build
cargo test --manifest-path scripts/core-tests/Cargo.toml --locked
# After building the engine and installing native packages:
cargo test --manifest-path src-tauri/Cargo.toml --locked
npm run tauri build
```

The `.deb` is produced under `src-tauri/target/release/bundle/deb/`. The engine is bundled alongside the app, with the upstream Apache-2.0 license and NOTICE. USB udev rules must be installed separately. Architecture support depends on building the matching native engine; x86_64 and aarch64 GNU/Linux executable names are supported.

### Browser preview and smoke checks

```sh
npm run dev
# In another terminal, with Chromium installed:
npm test
```

`CHROMIUM_PATH` can override `/usr/bin/chromium`; `APP_URL` can override the local test origin. Browser checks cover real preview interactions and mocked desktop command flows, not USB hardware. A static host such as Vercel can show this preview but cannot run the desktop backend.

## Device use

1. Back up data; select firmware for the exact device model and bootloader revision.
2. Put the phone in Download Mode and scan for it.
3. Select matching firmware packages and run Validate firmware.
4. Review the log, then explicitly confirm flashing.
5. Keep the app open, computer awake and USB connected until the engine finishes.

There is deliberately no mid-flash cancellation control. Firmware writes can erase data or make a device unbootable; validation cannot guarantee that every firmware/model combination is safe. A `.tar.md5` checksum is an integrity check, not proof of authenticity. This GUI relies on upstream Odin4's protocol and validation behavior; it does not implement independent Samsung model compatibility checks.
