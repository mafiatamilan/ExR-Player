# Building

## Minimal headless build

Prerequisites are a supported C toolchain, Python 3, Meson, Ninja, pkg-config,
GLib, Pixman, zlib and (for user-mode networking) libslirp. QEMU's configure and
Meson probes select host-specific facilities.

```sh
./scripts/build-headless.sh
```

The script configures `build/` on its first run, then builds
`qemu-system-x86_64`, `qemu-system-i386` and `qemu-img`. These upstream artifact
names are expected until Phase 3 lands. Set `BUILD_DIR` to use another build
directory and `JOBS` to control parallelism (default: 2).

To reconfigure after changing options, remove only the chosen build directory
or use Meson's reconfiguration support. Do not edit generated build files.

### NixOS example used for verification

The following supplies tools ephemerally and does not alter the system profile:

```sh
nix-shell -p gcc meson ninja pkg-config python3 glib pixman zlib libslirp \
  --run './scripts/build-headless.sh'
```

The verified build used GCC 16.2.0, Python 3.14.10, Meson 1.12.1, Ninja 1.13.2,
GLib 2.90.0 and Pixman 0.46.4. These are observations, not minimum versions.

## Configuration boundary

The build includes only i386 and x86_64 system emulation. It explicitly enables
KVM, TCG (upstream's mandatory default), libslirp and tools, and disables GTK,
SDL, Cocoa, VNC, SPICE, OpenGL, virglrenderer, documentation, guest-agent and
audio drivers. VGA and other guest hardware models required by operating systems
are not removed merely because host display frontends are disabled.

## Other configurations and hosts

Debug, release, development and packaging presets are not finalized. Windows
(MSVC/MinGW/WHPX), macOS (HVF, Intel and Apple Silicon), cross-compilation and
reproducible release packaging have not been built on their respective hosts;
no instructions are presented as tested until CI supplies that evidence.
