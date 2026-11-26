# Technical decisions

## D001 — Fork a stable upstream release

**Decision:** Start from official QEMU tag `v11.1.3`, commit
`005c61c0ef6f6ae615283440b4bb7be51b2c6ddd`.

**Why:** The tag was the newest non-release-candidate semantic-version tag
returned by the official repository on 2026-11-26. A stable baseline makes
runtime regressions and upstream synchronization attributable.

## D002 — Keep upstream execution layout and internal names

**Decision:** Rebrand public artifacts and interfaces, not every internal QEMU
identifier. Do not relocate execution sources into a cosmetic new tree.

**Why:** This preserves proven behavior, licensing context, module relationships
and tractable upstream merges. New product-only code can live under `exr/`.

## D003 — Establish a real minimal build before platform expansion

**Decision:** Build i386 and x86_64 system emulators on Linux first, disabling
only host frontends and optional dependencies. Test TCG before KVM.

**Why:** A working BIOS-level vertical slice distinguishes later integration
failures from baseline build failures. Windows and macOS cannot be runtime
verified on this Linux host and remain explicitly unverified.

## D004 — No synthesized accelerator abstraction

**Decision:** Use upstream `AccelClass` and accelerator operations. ExR will add
selection policy, not an assertion that KVM, WHPX and HVF expose identical
capabilities.

**Why:** CPU state, interrupt, memory and feature semantics differ. Concealing
those differences would risk correctness and misleading support claims.

## D005 — Do not disable security for memory targets

**Decision:** Preserve upstream isolation, validation and cleanup mechanisms.
The 2 GiB target will be addressed through measured allocation and cache policy,
not by weakening memory safety or trust-boundary checks.
