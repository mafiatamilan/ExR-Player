# Testing

## Current host smoke tests

From the configured source checkout:

```sh
./tests/exr/smoke-bios.sh
```

On the verified Linux x86_64 host this tests:

- x86_64 BIOS execution with TCG;
- i386 BIOS execution with TCG;
- x86_64 BIOS execution with KVM when `/dev/kvm` is accessible;
- qcow2 image creation and integrity checking.

The test deliberately waits for a timeout because an otherwise idle VM keeps
running. It requires SeaBIOS to reach the expected “No bootable device” state.
That proves firmware and CPU execution, **not** guest operating-system boot.
Logs are retained in `build/exr-smoke-*.log` for diagnosis.

## Verification tiers

Tests that require actual acceleration must report `SKIP` when facilities are
absent and must never convert an unavailable backend into a pass. Future suites
will distinguish software-only tests from KVM, WHPX and HVF hardware tests.

The following acceptance tests remain unimplemented or unverified: 32-bit and
64-bit real guest boot, UEFI, Windows guests, disk read/write inside a guest,
virtio networking, snapshots, serial API, memory-pressure behavior, controlled
shutdown/restart, cross-host configuration compatibility, and host CI.

## Initial results

On 2026-11-26, all three BIOS smoke cases passed and `qemu-img check` reported
no errors. The host is NixOS with Linux 6.18.62. This is a functional baseline,
not a benchmark result.
