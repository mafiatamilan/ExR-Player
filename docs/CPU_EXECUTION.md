# CPU Execution Architecture

ExR x86 retains the upstream QEMU x86/x86_64 execution infrastructure (`target/i386`, `tcg/`, `accel/kvm`, `accel/whpx`, `accel/hvf`).

## Execution Backends

| Host OS | Hardware Acceleration | Software Fallback | `acceleration: "auto"` Order |
|---|---|---|---|
| Linux | KVM (`accel/kvm/`) | TCG (`tcg/`) | `kvm` -> `tcg` |
| Windows | WHPX (`accel/whpx/`) | TCG (`tcg/`) | `whpx` -> `tcg` |
| macOS (Intel) | HVF (`accel/hvf/`) | TCG (`tcg/`) | `hvf` -> `tcg` |
| macOS (Apple Silicon) | None for x86 guests | TCG (`tcg/`) | `tcg` |

Note: `cpu: "host"` is rejected when `acceleration: "auto"` is selected, because `"host"` CPU passthrough cannot fall back to TCG.

## TCG Tuning

- Multi-threaded TCG (MTTCG) is enabled when host/guest memory ordering allows.
- Translation block cache size is controlled via `engine.tb-size` or `engine.profile` (`low-mem-2gb` -> `32 MiB`, `minimal` -> `16 MiB`).
- Dynamic SoftMMU TLB (`accel/tcg/cputlb.c`) scales adaptively based on conflict/use rate and is capped on low-memory hosts.
