# ExR Frontend Panel Integration

This directory is reserved for the ExR Android gaming emulator frontend panel.

## Integrating with `engine/`

- **C ABI Header**: `../engine/include/exr/api.h`
- **Shared Library**: `../engine/build/libexr-x86.so`
- **Headless Engine Binary**: `../engine/build/exr-x86`
- **VM Profile Format (`config.exr`)**: `../examples/config.exr`

## Key Capabilities for Frontend Panel

- **Config Persistence**: `exr_config_load("config.exr", &cfg)` / `exr_config_save(cfg, "config.exr")`
- **Memory & Storage Adjustment**:
  - `exr_config_set_memory(cfg, "768M")`, `exr_config_set_balloon(cfg, true)`
  - Live runtime RAM adjustment: `exr_machine_set_balloon_target(machine, bytes)`
  - Disk resize: `exr-x86 resize config.exr 32G`
- **GPU (Low-End & High-End)**:
  - `exr_config_set_display(cfg, "auto")` (uses `virtio-vga-gl` if virgl+OpenGL built, falls back to `virtio-vga` 2D on low-end)
  - `exr_config_set_vgamem(cfg, 32)` (VRAM cap in MiB)
  - `exr_machine_capture_screen(machine, "frame.ppm")`
- **Inputs**: `exr_config_set_inputs(cfg, true)`, `exr_machine_send_key()`, `exr_machine_send_pointer_abs()`, `exr_machine_send_pointer_btn()`
- **ADB Bridge**: `exr_config_set_adb_port(cfg, 5555)` -> `adb connect 127.0.0.1:5555`
- **Audio & Shared Folder**: `exr_config_set_audio(cfg, "auto", NULL)`, `exr_config_set_shared_folder(cfg, "./shared", true)`
- **Live Telemetry**: `exr_machine_get_stats(machine, &stats)` (`state`, `pid`, `rss_kib`)
