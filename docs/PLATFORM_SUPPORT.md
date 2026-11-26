# Platform support

Only cells marked **Verified** have actually built and run in this project.
“Inherited” describes upstream source and is not an ExR support claim.

| Host | Build | TCG | Hardware acceleration | Notes |
|---|---|---|---|---|
| Linux x86_64 | **Verified** | **Verified:** i386 + x86_64 BIOS | **KVM verified:** x86_64 BIOS | NixOS, Linux 6.18.62, `/dev/kvm` available |
| Linux other architectures | Unverified | Unverified | Unverified | Cross-architecture TCG not tested |
| Windows x86_64 | Unverified | Unverified | WHPX unverified | Native toolchain, feature availability, permissions and packaging untested |
| macOS Intel | Unverified | Unverified | HVF unverified | No test host available |
| macOS Apple Silicon | Unverified | Unverified | Not applicable to native x86 execution | x86 guests require cross-architecture TCG |
| FreeBSD / other BSD | Unverified | Unverified | NVMM and other inherited backends unverified | Future target only |

## Host abstraction policy

ExR retains upstream OS-dependent memory, threading, socket, disk and process
code in `util/`, `system/`, `net/`, `block/` and `os-*.c`. New policy belongs in
thin ExR-facing layers; it must not replace proven platform code with a lowest
common denominator. Backend discovery and `auto` selection must fail clearly
when the requested facility is unavailable.

WHPX and HVF support differs by OS version, host CPU, enabled hypervisor
services, APIs exposed, and guest CPU features. Their presence in upstream
source is not evidence of a working ExR build. Apple Silicon compatibility is
reported separately from Intel Mac compatibility at every stage.
