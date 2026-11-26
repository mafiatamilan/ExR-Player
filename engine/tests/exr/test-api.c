#include "exr/api.h"

#include <assert.h>
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

static pthread_mutex_t out_lock = PTHREAD_MUTEX_INITIALIZER;
static char serial_buf[16384];
static size_t serial_len = 0;

static void on_serial(const uint8_t *data, size_t len, void *opaque)
{
    (void)opaque;
    pthread_mutex_lock(&out_lock);
    size_t copy = len;
    if (serial_len + copy >= sizeof(serial_buf)) {
        copy = sizeof(serial_buf) - 1 - serial_len;
    }
    if (copy > 0) {
        memcpy(serial_buf + serial_len, data, copy);
        serial_len += copy;
        serial_buf[serial_len] = '\0';
    }
    pthread_mutex_unlock(&out_lock);
}

static void on_event(exr_machine_state_t state, const char *msg, void *opaque)
{
    (void)opaque;
    printf("[EVENT] state=%d msg=%s\n", (int)state, msg ? msg : "");
}

int main(int argc, char **argv)
{
    const char *engine = argc > 1 ? argv[1] : "./build/exr-x86";
    ExRConfigHandle *cfg = exr_config_create();
    assert(cfg != NULL);

    assert(exr_config_set_engine_path(cfg, engine) == EXR_OK);
    assert(exr_config_set_arch(cfg, "x86_64") == EXR_OK);
    assert(exr_config_set_accel(cfg, EXR_ACCEL_TCG) == EXR_OK);
    assert(exr_config_set_memory(cfg, "64M") == EXR_OK);
    assert(exr_config_set_profile(cfg, EXR_PROFILE_LOW_MEM_2GB) == EXR_OK);
    assert(exr_config_set_balloon(cfg, true) == EXR_OK);
    assert(exr_config_set_display(cfg, "auto") == EXR_OK);
    assert(exr_config_set_vgamem(cfg, 32) == EXR_OK);
    assert(exr_config_set_inputs(cfg, true) == EXR_OK);
    assert(exr_config_set_audio(cfg, "none", NULL) == EXR_OK);
    assert(exr_config_set_network(cfg, "user") == EXR_OK);
    assert(exr_config_set_adb_port(cfg, 15555) == EXR_OK);
    mkdir("/tmp/exr-shared-test", 0755);
    assert(exr_config_set_shared_folder(cfg, "/tmp/exr-shared-test", true) == EXR_OK);

    const char *exr_cfg_path = "/tmp/test-config.exr";
    assert(exr_config_save(cfg, exr_cfg_path) == EXR_OK);
    ExRConfigHandle *loaded = NULL;
    assert(exr_config_load(exr_cfg_path, &loaded) == EXR_OK);
    assert(exr_config_set_engine_path(loaded, engine) == EXR_OK);
    unlink(exr_cfg_path);

    ExRMachine *m = NULL;
    assert(exr_machine_create(loaded, &m) == EXR_OK);
    exr_config_free(loaded);
    exr_config_free(cfg);

    assert(exr_machine_set_event_callback(m, on_event, NULL) == EXR_OK);
    assert(exr_machine_set_serial_output_callback(m, on_serial, NULL) == EXR_OK);

    if (exr_machine_start(m) != EXR_OK) {
        fprintf(stderr, "Start failed: %s\n", exr_get_last_error());
        exr_machine_destroy(m);
        return 1;
    }
    assert(exr_machine_get_state(m) == EXR_STATE_RUNNING);

    /* Wait briefly for SeaBIOS to initialize virtio-vga display */
    usleep(1000000);
    const char *ppm = "/tmp/exr-test-screen.ppm";
    unlink(ppm);
    assert(exr_machine_capture_screen(m, ppm) == EXR_OK);
    assert(access(ppm, R_OK) == 0);
    unlink(ppm);
    printf("PASS C API headless display capture (virtio-vga screendump)\n");

    exr_machine_stats_t stats;
    assert(exr_machine_get_stats(m, &stats) == EXR_OK);
    assert(stats.state == EXR_STATE_RUNNING);
    assert(stats.pid > 0);
    assert(stats.rss_kib > 0);
    printf("PASS C API telemetry (pid=%d, rss=%llu KiB)\n",
           stats.pid, (unsigned long long)stats.rss_kib);

    assert(exr_machine_send_key(m, "ret", true) == EXR_OK);
    assert(exr_machine_send_key(m, "ret", false) == EXR_OK);
    assert(exr_machine_send_pointer_abs(m, 16384, 16384) == EXR_OK);
    assert(exr_machine_send_pointer_btn(m, EXR_MOUSE_LEFT, true) == EXR_OK);
    assert(exr_machine_send_pointer_btn(m, EXR_MOUSE_LEFT, false) == EXR_OK);
    printf("PASS C API input injection (keyboard/abs-touch/mouse)\n");

    assert(exr_machine_set_balloon_target(m, 48ULL * 1024 * 1024) == EXR_OK);
    printf("PASS C API live memory balloon target + config.exr save/load\n");

    assert(exr_machine_pause(m) == EXR_OK);
    assert(exr_machine_get_state(m) == EXR_STATE_PAUSED);
    assert(exr_machine_resume(m) == EXR_OK);
    assert(exr_machine_get_state(m) == EXR_STATE_RUNNING);
    assert(exr_machine_reset(m) == EXR_OK);
    assert(exr_machine_stop(m) == EXR_OK);
    assert(exr_machine_get_state(m) == EXR_STATE_STOPPED);

    exr_machine_destroy(m);
    printf("PASS C API lifecycle (create/start/pause/resume/reset/stop/destroy)\n");
    return 0;
}
