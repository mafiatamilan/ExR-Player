/*
 * ExR x86 Stable C Integration API.
 * Copyright (C) 2026 ExR x86 contributors
 *
 * This work is licensed under the terms of the GNU LGPL, version 2.1 or later.
 */
#ifndef EXR_API_H
#define EXR_API_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    EXR_OK = 0,
    EXR_ERR_INVALID_ARG = -1,
    EXR_ERR_STATE = -2,
    EXR_ERR_IO = -3,
    EXR_ERR_NOT_SUPPORTED = -4,
    EXR_ERR_FAILED = -5,
} exr_status_t;

typedef enum {
    EXR_ACCEL_AUTO = 0,
    EXR_ACCEL_TCG,
    EXR_ACCEL_KVM,
    EXR_ACCEL_WHPX,
    EXR_ACCEL_HVF,
} exr_accel_t;

typedef enum {
    EXR_PROFILE_STANDARD = 0,
    EXR_PROFILE_LOW_MEM_2GB,
    EXR_PROFILE_MINIMAL,
} exr_memory_profile_t;

typedef enum {
    EXR_STATE_STOPPED = 0,
    EXR_STATE_STARTING,
    EXR_STATE_RUNNING,
    EXR_STATE_PAUSED,
    EXR_STATE_ERROR,
} exr_machine_state_t;

typedef void (*exr_event_cb_t)(exr_machine_state_t state,
                               const char *message,
                               void *opaque);

typedef void (*exr_serial_output_cb_t)(const uint8_t *data,
                                       size_t len,
                                       void *opaque);

typedef struct {
    exr_machine_state_t state;
    int32_t pid;
    uint64_t rss_kib;
} exr_machine_stats_t;

typedef enum {
    EXR_MOUSE_LEFT = 0,
    EXR_MOUSE_MIDDLE,
    EXR_MOUSE_RIGHT,
} exr_mouse_button_t;

typedef struct ExRMachine ExRMachine;
typedef struct ExRConfigHandle ExRConfigHandle;

/* Version */
const char *exr_get_version(void);

/* Configuration API */
ExRConfigHandle *exr_config_create(void);
exr_status_t exr_config_load(const char *exr_path, ExRConfigHandle **out_cfg);
exr_status_t exr_config_save(const ExRConfigHandle *cfg, const char *exr_path);
void exr_config_free(ExRConfigHandle *cfg);
exr_status_t exr_config_set_engine_path(ExRConfigHandle *cfg, const char *path);
exr_status_t exr_config_set_arch(ExRConfigHandle *cfg, const char *arch);
exr_status_t exr_config_set_accel(ExRConfigHandle *cfg, exr_accel_t accel);
exr_status_t exr_config_set_memory(ExRConfigHandle *cfg, const char *memory_str);
exr_status_t exr_config_set_profile(ExRConfigHandle *cfg, exr_memory_profile_t profile);
exr_status_t exr_config_set_smp(ExRConfigHandle *cfg, uint16_t smp);
exr_status_t exr_config_set_cpu(ExRConfigHandle *cfg, const char *cpu_model);
exr_status_t exr_config_set_balloon(ExRConfigHandle *cfg, bool enabled);
exr_status_t exr_config_set_display(ExRConfigHandle *cfg, const char *display);
exr_status_t exr_config_set_vgamem(ExRConfigHandle *cfg, uint16_t vgamem_mb);
exr_status_t exr_config_set_inputs(ExRConfigHandle *cfg, bool enabled);
exr_status_t exr_config_set_audio(ExRConfigHandle *cfg,
                                  const char *backend,
                                  const char *wav_path);
exr_status_t exr_config_set_adb_port(ExRConfigHandle *cfg, uint16_t port);
exr_status_t exr_config_set_shared_folder(ExRConfigHandle *cfg,
                                          const char *host_path,
                                          bool readonly);
exr_status_t exr_config_add_extra_disk(ExRConfigHandle *cfg,
                                       const char *path,
                                       const char *format);
exr_status_t exr_config_set_kernel(ExRConfigHandle *cfg,
                                   const char *kernel_path,
                                   const char *initrd_path,
                                   const char *cmdline);
exr_status_t exr_config_set_disk(ExRConfigHandle *cfg,
                                 const char *path,
                                 const char *format,
                                 const char *interface);
exr_status_t exr_config_set_network(ExRConfigHandle *cfg, const char *backend);
exr_status_t exr_config_set_control_socket(ExRConfigHandle *cfg, const char *socket_path);

/* Machine Lifecycle API */
exr_status_t exr_machine_create(const ExRConfigHandle *cfg, ExRMachine **out_machine);
exr_status_t exr_machine_start(ExRMachine *machine);
exr_status_t exr_machine_pause(ExRMachine *machine);
exr_status_t exr_machine_resume(ExRMachine *machine);
exr_status_t exr_machine_reset(ExRMachine *machine);
exr_status_t exr_machine_stop(ExRMachine *machine);
exr_status_t exr_machine_wait(ExRMachine *machine, int timeout_ms);
exr_machine_state_t exr_machine_get_state(ExRMachine *machine);
void exr_machine_destroy(ExRMachine *machine);

/* Snapshot API */
exr_status_t exr_machine_snapshot_save(ExRMachine *machine, const char *name);
exr_status_t exr_machine_snapshot_load(ExRMachine *machine, const char *name);
exr_status_t exr_machine_snapshot_delete(ExRMachine *machine, const char *name);

/* Event & Serial Console API */
exr_status_t exr_machine_set_event_callback(ExRMachine *machine,
                                            exr_event_cb_t cb,
                                            void *opaque);
exr_status_t exr_machine_set_serial_output_callback(ExRMachine *machine,
                                                    exr_serial_output_cb_t cb,
                                                    void *opaque);
exr_status_t exr_machine_serial_input(ExRMachine *machine,
                                      const uint8_t *data,
                                      size_t len);

/* Frontend Panel Input Injection & Live Telemetry API */
exr_status_t exr_machine_send_key(ExRMachine *machine, const char *qcode, bool down);
exr_status_t exr_machine_send_pointer_abs(ExRMachine *machine, int32_t x, int32_t y);
exr_status_t exr_machine_send_pointer_btn(ExRMachine *machine,
                                          exr_mouse_button_t btn,
                                          bool down);
exr_status_t exr_machine_get_stats(ExRMachine *machine, exr_machine_stats_t *out_stats);
exr_status_t exr_machine_set_balloon_target(ExRMachine *machine, uint64_t target_bytes);
exr_status_t exr_machine_capture_screen(ExRMachine *machine, const char *ppm_path);

/* Error Reporting API */
const char *exr_get_last_error(void);

#ifdef __cplusplus
}
#endif

#endif /* EXR_API_H */
