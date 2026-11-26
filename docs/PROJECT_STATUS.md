# Project status

Last updated: 2026-11-26. "Verified" means exercised on the available host;
source availability alone is never treated as verification.

| Phase | Status | Evidence / remaining work |
|---|---|---|
| 0. Source acquisition | Complete | Official `v11.1.3`, exact commit recorded, independent `main`, read-only upstream push disabled |
| 1. Architecture analysis | Complete | Execution, accelerator, memory, machine, device and build ownership mapped in `ARCHITECTURE.md` |
| 2. Minimal headless build | Complete on Linux x86_64 | `exr-x86`, `exr-img`, `libexr-x86.so` built without GUI frontends |
| 3. Rebranding and CLI | Complete | `exr-x86` (`create`, `run`, `stop`, `inspect`, `snapshot`, `devices`, `version`), `.exrimg` disk format, zero public QEMU branding |
| 4. Linux backend | Verified | TCG/KVM SeaBIOS smoke tests pass; real Linux 6.18.62 kernel boots under KVM and TCG |
| 5. Windows backend | Unverified | No Windows host or WHPX runtime test available |
| 6. macOS backend | Unverified | No Intel or Apple Silicon host test available |
| 7. Acceleration abstraction | Complete | `auto` selects `kvm`/`whpx`/`hvf` with TCG fallback; `cpu=host` guarded against TCG fallback |
| 8. 2 GiB optimization | Implemented & measured | `tcg/region.c` TB cap, `system/physmem.c` `MADV_NOHUGEPAGE`, `accel/tcg/cputlb.c` TLB cap, `low-mem-2gb` profile, `virtio-balloon-pci` |
| 9. Device optimization | Implemented | Dedicated `iothread` for `virtio-blk-pci`, `discard=unmap`, `detect-zeroes=unmap`, tuned `.exrimg` L2 cache |
| 10. Integration API | Complete & tested | Stable C ABI in `include/exr/api.h` + `exr/api.c` (`libexr-x86.so`), tested via `tests/exr/test-api.c` with input injection & live telemetry |
| 11. Automated testing | Verified on Linux | `tests/exr/smoke-bios.sh` and `tests/exr/test-api.c` pass |
| 12. Benchmarking | Complete on Linux | Measured RSS, binary sizes, Linux kernel boot, and `.exrimg` create latency in `BENCHMARKS.md` |
| 13. Packaging | Partial | Meson/Ninja install targets configured (`exr-x86`, `exr-img`, `libexr-x86`, `exr/api.h`) |
| 14. Documentation | Complete | All 13 mandatory documents present |
