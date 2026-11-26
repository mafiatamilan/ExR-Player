#!/bin/sh
# Build only the x86 system engines and image tool, without host frontends.
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
build_dir=${BUILD_DIR:-"$root/build"}

mkdir -p "$build_dir"
cd "$build_dir"

if [ ! -f config-host.mak ]; then
    "$root/configure" \
        --target-list=x86_64-softmmu \
        --without-default-features \
        --enable-kvm \
        --enable-slirp \
        --enable-vvfat \
        --enable-tools \
        --disable-gtk \
        --disable-sdl \
        --disable-cocoa \
        --disable-vnc \
        --disable-spice \
        --disable-opengl \
        --disable-virglrenderer \
        --disable-docs \
        --disable-guest-agent \
        --disable-debug-info \
        --disable-werror \
        --audio-drv-list=
    meson configure . -Dpixman=enabled
fi

ninja -j"${JOBS:-2}" exr-x86 exr-img libexr-x86.so

dist_dir=${DIST_DIR:-"$root/../dist/engine"}
mkdir -p "$dist_dir/bin" "$dist_dir/lib" "$dist_dir/include/exr" "$dist_dir/pc-bios"
cp -f exr-x86 exr-img "$dist_dir/bin/"
cp -f libexr-x86.so "$dist_dir/lib/"
cp -f "$root/include/exr/api.h" "$dist_dir/include/exr/"
cp -f "$root"/pc-bios/bios*.bin "$root"/pc-bios/vgabios*.bin "$root"/pc-bios/kvmvapic.bin "$root"/pc-bios/linuxboot*.bin "$root"/pc-bios/efi-*.rom "$dist_dir/pc-bios/" 2>/dev/null || true
