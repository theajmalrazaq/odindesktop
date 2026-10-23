Linux desktop prerelease with a bundled Odin4 engine. Open the installed application from your desktop's application menu.

Downloads (x86_64 only):
- Ubuntu 24.04 / compatible Debian: `.deb`
- Arch Linux / compatible derivatives: `.pkg.tar.zst` (install with `pacman -U`; not an AUR publication)
- Fedora-family systems: `.rpm`
- Other compatible Linux desktops: `.AppImage`

Requires glibc 2.39 or newer. AppImage users must install the supplied USB udev rules separately. Native packages include those rules; reload udev rules and reconnect your phone after installation. See the repository README for installation commands and dependencies. ARM64, Windows and macOS packages are not provided.

Includes device scanning, firmware file selection, validation before flashing, explicit confirmation, local logs and optional reboot. Packages are published only after the Linux build, native unit tests and Debian/Arch launch checks pass. Physical Samsung device detection and flashing have not been verified by this release pipeline. RPM and AppImage installation are not tested across every distribution.

Verify downloads against SHA256SUMS. Back up your phone and select firmware for its exact model before using firmware-writing operations.
