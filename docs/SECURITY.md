# Security Model

ExR x86 preserves all upstream QEMU memory-safety, hardening, and privilege boundaries:

- **Unprivileged Execution**: Runs as the invoking user by default; root privileges are never required.
- **Compiler Hardening**: Built with `-fPIE`, `-Wl,-z,relro`, `-Wl,-z,now`, `-ftrivial-auto-var-init=zero`, `-fzero-init-padding-bits=all`, and `-fzero-call-used-regs=used-gpr`.
- **Input Validation**:
  - JSON configuration is strictly validated against `qapi/exr.qapi` and capped at `1 MiB` (`EXR_CONFIG_MAX_BYTES`).
  - Control socket responses are capped at `16 MiB` (`EXR_CTRL_MAX_BYTES`).
  - Snapshot names are restricted to `1-128` characters from `[A-Za-z0-9_.-]`.
  - `exr-x86 create` refuses to overwrite an existing `.exrimg` or `raw` disk image.
- **Process & Runtime Isolation**:
  - `libexr-x86` spawns each VM in an isolated process with a private `mkdtemp("/tmp/exr-rt-XXXXXX")` directory and cleans up sockets/configs on `exr_machine_destroy()`.
  - `MADV_DONTFORK` and `MADV_DONTDUMP` are preserved on guest RAM blocks.
