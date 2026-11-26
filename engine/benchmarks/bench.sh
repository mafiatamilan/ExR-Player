#!/bin/sh
# ExR x86 benchmark suite: measures RSS, startup/boot latency, and binary size.
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
build_dir=${BUILD_DIR:-"$root/build"}
engine="$build_dir/exr-x86"
img_tool="$build_dir/exr-img"
kernel="/nix/store/scmf8v3nc10jbdawqmf69f2wbqbcv81d-linux-6.18.62/bzImage"

echo "=== ExR x86 Benchmark Suite ==="
echo "Host: $(uname -srm)"
echo "CPU:  $(grep -m1 'model name' /proc/cpuinfo | cut -d: -f2- | xargs)"
echo "RAM:  $(free -m | awk '/Mem:/ {print $2 " MiB total, " $7 " MiB available"}')"
echo ""

echo "--- Binary Sizes ---"
ls -lh "$engine" "$img_tool" "$build_dir/libexr-x86.so" | awk '{print $9 ": " $5}'
echo ""

measure_vm() {
    name=$1
    accel=$2
    profile=$3
    mem=$4
    extra_boot=$5
    cfg="$build_dir/bench-$name.json"
    log="$build_dir/bench-$name.log"

    if [ -n "$extra_boot" ]; then
        cat >"$cfg" <<EOF
{"schema-version":1,"engine":{"architecture":"x86_64","acceleration":"$accel","memory":"$mem","profile":"$profile","display":"none"},"boot":{"kernel":"$extra_boot","cmdline":"console=ttyS0 panic=-1"},"runtime":{"serial":"stdio"}}
EOF
    else
        cat >"$cfg" <<EOF
{"schema-version":1,"engine":{"architecture":"x86_64","acceleration":"$accel","memory":"$mem","profile":"$profile","display":"none"},"runtime":{"serial":"stdio"}}
EOF
    fi

    start_ns=$(date +%s%N)
    "$engine" run "$cfg" < /dev/null > "$log" 2>&1 &
    pid=$!

    peak_rss=0
    idle_rss=0
    elapsed_ms=0
    while kill -0 "$pid" 2>/dev/null; do
        if [ -r "/proc/$pid/status" ]; then
            rss=$(awk '/VmRSS:/ {print $2}' "/proc/$pid/status" 2>/dev/null || echo 0)
            hwm=$(awk '/VmHWM:/ {print $2}' "/proc/$pid/status" 2>/dev/null || echo 0)
            [ "${rss:-0}" -gt "$idle_rss" ] && idle_rss=$rss
            [ "${hwm:-0}" -gt "$peak_rss" ] && peak_rss=$hwm
        fi
        now_ns=$(date +%s%N)
        elapsed_ms=$(( (now_ns - start_ns) / 1000000 ))
        if [ "$elapsed_ms" -ge 4000 ]; then
            break
        fi
        sleep 0.1
    done

    kill -TERM "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true

    printf "%-28s | accel=%-3s | profile=%-11s | mem=%-4s | RSS=%6d KiB | Peak=%6d KiB\n" \
        "$name" "$accel" "$profile" "$mem" "$idle_rss" "$peak_rss"
    rm -f "$cfg" "$log"
}

echo "--- Memory Footprint (Idle / Peak RSS over 4s window) ---"
measure_vm "bios-tcg-standard" "tcg" "standard" "256M" ""
measure_vm "bios-tcg-lowmem2gb" "tcg" "low-mem-2gb" "256M" ""
measure_vm "bios-tcg-minimal" "tcg" "minimal" "128M" ""

if [ -e /dev/kvm ] && [ -r /dev/kvm ] && [ -w /dev/kvm ]; then
    measure_vm "bios-kvm-standard" "kvm" "standard" "256M" ""
    measure_vm "bios-kvm-lowmem2gb" "kvm" "low-mem-2gb" "256M" ""
fi

if [ -f "$kernel" ]; then
    echo ""
    echo "--- Real Linux 6.18.62 Kernel Boot RSS ---"
    measure_vm "linux-tcg-lowmem2gb" "tcg" "low-mem-2gb" "256M" "$kernel"
    if [ -e /dev/kvm ] && [ -r /dev/kvm ] && [ -w /dev/kvm ]; then
        measure_vm "linux-kvm-lowmem2gb" "kvm" "low-mem-2gb" "256M" "$kernel"
    fi
fi

echo ""
echo "--- Disk Image Creation Latency (16G sparse .exrimg) ---"
dcfg="$build_dir/bench-disk.json"
dimg="$build_dir/bench-disk.exrimg"
rm -f "$dimg"
cat >"$dcfg" <<EOF
{"schema-version":1,"engine":{"architecture":"x86_64","acceleration":"tcg","memory":"64M","display":"none"},"storage":{"path":"$dimg","format":"exrimg","interface":"virtio","size":"16G"}}
EOF
/run/current-system/sw/bin/time -f "exr-x86 create 16G .exrimg: elapsed=%es maxrss=%M KiB" "$engine" create "$dcfg" >/dev/null
rm -f "$dcfg" "$dimg"
