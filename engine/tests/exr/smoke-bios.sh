#!/bin/sh
# Linux smoke test: BIOS execution proves the CPU loop is running, not OS boot.
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
build_dir=${BUILD_DIR:-"$root/build"}
timeout_seconds=${SMOKE_TIMEOUT:-8}

run_bios() {
    arch=$1
    accel=$2
    log="$build_dir/exr-smoke-$arch-$accel.log"
    config="$build_dir/exr-smoke-$arch-$accel.json"

    engine="$build_dir/exr-x86"
    if [ "$arch" = i386 ]; then
        cpu="qemu32"
    else
        cpu="max"
    fi
    cat >"$config" <<EOF
{"schema-version":1,"engine":{"architecture":"$arch","acceleration":"$accel","memory":"32M","cpu":"$cpu","display":"none"},"runtime":{"serial":"stdio"}}
EOF

    set +e
    timeout "$timeout_seconds" "$engine" run "$config" < /dev/null >"$log" 2>&1
    status=$?
    set -e

    if [ "$status" -ne 124 ]; then
        echo "FAIL $arch/$accel exited unexpectedly ($status)" >&2
        cat "$log" >&2
        return 1
    fi
    if ! grep -q 'SeaBIOS' "$log" || ! grep -q 'No bootable device' "$log"; then
        echo "FAIL $arch/$accel did not complete the BIOS boot attempt" >&2
        cat "$log" >&2
        return 1
    fi
    echo "PASS $arch/$accel SeaBIOS execution"
}

run_bios x86_64 tcg
run_bios i386 tcg
if [ -e /dev/kvm ] && [ -r /dev/kvm ] && [ -w /dev/kvm ]; then
    run_bios x86_64 kvm
else
    echo "SKIP x86_64/kvm: /dev/kvm is unavailable or inaccessible"
fi

image="$build_dir/exr-smoke-image.exrimg"
img_config="$build_dir/exr-smoke-image.json"
trap 'rm -f "$image" "$img_config"' EXIT HUP INT TERM
rm -f "$image"
cat >"$img_config" <<EOF
{"schema-version":1,"engine":{"architecture":"x86_64","acceleration":"tcg","memory":"32M","display":"none"},"storage":{"path":"$image","format":"exrimg","interface":"virtio","size":"16M"}}
EOF
"$build_dir/exr-x86" create "$img_config" >/dev/null
"$build_dir/exr-x86" resize "$img_config" 32M >/dev/null
"$build_dir/exr-img" check -q "$image"
echo "PASS exrimg create/resize/check"
