# Architecture

## Fork boundary

ExR keeps QEMU's Meson/Ninja source layout. Moving execution files into a new
cosmetic directory tree would obscure upstream diffs without improving the
engine. New product-facing management and API code may use `exr/`; internal
execution symbols remain upstream-compatible.

## Execution path

`system/main.c` enters `qemu_init()` in `system/vl.c`, then the main loop.
Machine and device models descend from QOM/QDevice. The x86 PC machine is under
`hw/i386/`; CPU object state, instruction helpers and machine migration state
are under `target/i386/`.

- **TCG:** x86 decoding/translation is owned by `target/i386/tcg/`, with common
  translation and code generation in `tcg/` and execution, translation-block,
  TLB and CPU-loop management in `accel/tcg/` and `system/cpus.c`.
- **KVM:** common acceleration is in `accel/kvm/`; x86 CPUID, MSRs, interrupt
  state and virtual-CPU entry/exit are in `target/i386/kvm/`.
- **WHPX:** `accel/whpx/` contains common Windows Hypervisor Platform support;
  target-independent x86 handling and platform API capabilities must be checked
  before advertising feature parity with KVM.
- **HVF:** common framework code is in `accel/hvf/`, with x86 support in
  `target/i386/hvf/`. HVF is an x86-host path, not a mechanism for executing an
  x86 guest natively on Apple Silicon; that configuration requires TCG.
- **Abstraction:** `AccelClass` and accelerator ops select backend behavior.
  ExR will expose policy (`auto`, `tcg`, `kvm`, `whpx`, `hvf`) without claiming
  equivalent API capabilities across hosts.

## Memory and devices

`system/memory.c`, `system/physmem.c`, `memory.c` and `util/` implement address
spaces, RAM blocks, mappings and allocation support. TCG adds the software TLB
in `accel/tcg/cputlb.c`; hardware accelerators own their respective guest
physical-memory synchronization.

The block layer is in `block/`; qcow2, raw, throttling and asynchronous I/O are
shared with device models. Virtio is in `hw/virtio/`, PCI in `hw/pci/`, serial
in `hw/char/` and `chardev/`, and networking in `net/` and `hw/net/`. These are
required for real operating systems and therefore are not grouped with GUI
code for removal.

Control passes through QMP/HMP and QAPI (`qapi/`, `qobject/`, `qmp/`, `monitor/`).
The future ExR C ABI will delegate lifecycle and control to these mechanisms,
not maintain a second machine implementation.

## Headless build boundary

The minimal x86 system-emulator targets remain enabled. GTK, SDL, Cocoa, VNC,
SPICE, OpenGL, virglrenderer, audio drivers and guest-agent utilities are
excluded by configuration. Display-independent console and graphics device
emulation must not be confused with host GUI frontends.

## Build dependencies

The required native baseline here is a C compiler, Python, Meson, Ninja,
pkg-config, GLib, Pixman and zlib; libslirp supplies user-mode networking.
Optional upstream features are disabled unless a product requirement enables
them. Upstream's configure/Meson probes remain authoritative on each host.
