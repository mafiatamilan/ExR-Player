# Benchmarks

Measured on Linux 6.18.62 (`x86_64`, AMD Ryzen 5 7535HS, 23222 MiB host RAM) via `./benchmarks/bench.sh`.

## Binary Sizes

| Artifact | Size |
|---|---|
| `exr-x86` | 30 MiB |
| `exr-img` | 2.3 MiB |
| `libexr-x86.so` | 35 KiB |

## Memory Footprint (4-Second Window)

| Scenario | Accel | Profile | Guest RAM | Idle RSS | Peak RSS |
|---|---|---|---|---|---|
| SeaBIOS boot | `tcg` | `standard` | 256M | 42784 KiB | 42784 KiB |
| SeaBIOS boot | `tcg` | `low-mem-2gb` | 256M | 43492 KiB | 43492 KiB |
| SeaBIOS boot | `tcg` | `minimal` | 128M | 43212 KiB | 43212 KiB |
| SeaBIOS boot | `kvm` | `standard` | 256M | 39576 KiB | 39576 KiB |
| SeaBIOS boot | `kvm` | `low-mem-2gb` | 256M | 39576 KiB | 39576 KiB |
| Linux 6.18.62 boot | `tcg` | `low-mem-2gb` | 256M | 215476 KiB | 215476 KiB |
| Linux 6.18.62 boot | `kvm` | `low-mem-2gb` | 256M | 236768 KiB | 236768 KiB |

## Disk Image Creation

- `exr-x86 create` (16 GiB sparse `.exrimg`): `0.00s` elapsed, `16584 KiB` max RSS.

## Unverified Platforms

- Windows (WHPX/TCG) and macOS (HVF/TCG) benchmarks are unverified because Windows and macOS hosts are not available in this environment.
- 2 GiB physical host hardware run was not executed directly (host has 23222 MiB RAM); `low-mem-2gb` thresholds in `tcg/region.c`, `system/physmem.c`, and `accel/tcg/cputlb.c` are verified by code inspection and profile execution.
