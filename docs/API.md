# ExR x86 Integration API

`libexr-x86.so` (`include/exr/api.h`) exposes a stable C11 ABI for embedding the headless `exr-x86` engine into external frontends, including the ExR Android gaming emulator panel.

## Architecture Principles

- **Headless & Decoupled**: Zero dependency on GTK, SDL, Cocoa, VNC, SPICE, or QEMU internal headers.
- **Process Isolation**: `libexr-x86` manages the `exr-x86` engine as an isolated child process with a dedicated control socket (`unix:...`) and serial pipes. A guest fault cannot crash the host frontend panel.

## Core API Groups

1. **Configuration (`ExRConfigHandle`)**
   - `exr_config_create()`, `exr_config_free()`
   - `exr_config_set_arch()`, `exr_config_set_accel()`, `exr_config_set_memory()`, `exr_config_set_profile()`
   - `exr_config_set_smp()`, `exr_config_set_cpu()`, `exr_config_set_balloon()`
   - `exr_config_set_kernel()`, `exr_config_set_disk()`, `exr_config_set_network()`
   - `exr_config_set_engine_path()`, `exr_config_set_control_socket()`

2. **Machine Lifecycle (`ExRMachine`)**
   - `exr_machine_create()`, `exr_machine_start()`, `exr_machine_pause()`, `exr_machine_resume()`
   - `exr_machine_reset()`, `exr_machine_stop()`, `exr_machine_wait()`, `exr_machine_get_state()`, `exr_machine_destroy()`

3. **Snapshots**
   - `exr_machine_snapshot_save()`, `exr_machine_snapshot_load()`, `exr_machine_snapshot_delete()`

4. **Serial Console & Events**
   - `exr_machine_set_event_callback()` — state change notifications (`STOPPED`, `STARTING`, `RUNNING`, `PAUSED`, `ERROR`)
   - `exr_machine_set_serial_output_callback()`, `exr_machine_serial_input()`

5. **Frontend Gaming Panel Input & Telemetry**
   - `exr_machine_send_key()` — inject keyboard/gamepad-mapped qcode events
   - `exr_machine_send_pointer_abs()` — inject absolute touch/pointer coordinates (`0..32767`)
   - `exr_machine_send_pointer_btn()` — inject mouse/touch button press/release
   - `exr_machine_get_stats()` — live `exr_machine_stats_t` (`state`, `pid`, `rss_kib`)

6. **Error Reporting**
   - `exr_get_last_error()` — thread-local human-readable diagnostic message.
