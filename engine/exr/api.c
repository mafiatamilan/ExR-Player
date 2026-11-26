/*
 * ExR x86 Stable C Integration API Implementation.
 * Copyright (C) 2026 ExR x86 contributors
 *
 * This work is licensed under the terms of the GNU LGPL, version 2.1 or later.
 */

#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif
#include "exr/api.h"

#include <errno.h>
#include <fcntl.h>
#include <pthread.h>
#include <signal.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/un.h>
#include <sys/wait.h>
#include <unistd.h>

#define EXR_API_VERSION "0.1.0-dev"

static __thread char exr_last_error[512];

static void exr_set_error(const char *fmt, ...)
{
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(exr_last_error, sizeof(exr_last_error), fmt, ap);
    va_end(ap);
}

const char *exr_get_last_error(void)
{
    return exr_last_error;
}

const char *exr_get_version(void)
{
    return EXR_API_VERSION;
}

typedef struct {
    char *path;
    char *format;
} ExRExtraDisk;

struct ExRConfigHandle {
    char *engine_path;
    char *arch;
    exr_accel_t accel;
    char *memory;
    exr_memory_profile_t profile;
    uint16_t smp;
    char *cpu;
    bool balloon;
    char *display;
    uint16_t vgamem_mb;
    bool inputs;

    char *audio_backend;
    char *audio_wav_path;

    char *shared_folder_path;
    bool shared_folder_ro;

    char *kernel;
    char *initrd;
    char *cmdline;

    char *disk_path;
    char *disk_format;
    char *disk_interface;

    ExRExtraDisk *extra_disks;
    size_t extra_disk_count;

    char *network_backend;
    uint16_t adb_port;
    char *control_socket;
};

struct ExRMachine {
    ExRConfigHandle *cfg;
    exr_machine_state_t state;
    pthread_mutex_t lock;

    pid_t pid;
    int stdin_fd;
    int stdout_fd;
    pthread_t io_thread;
    bool io_thread_started;

    char *runtime_dir;
    char *config_path;
    char *socket_path;
    bool owns_socket;

    exr_event_cb_t event_cb;
    void *event_opaque;
    exr_serial_output_cb_t serial_cb;
    void *serial_opaque;
};

static char *exr_strdup_safe(const char *s)
{
    return s ? strdup(s) : NULL;
}

ExRConfigHandle *exr_config_create(void)
{
    ExRConfigHandle *cfg = calloc(1, sizeof(*cfg));
    if (!cfg) {
        exr_set_error("out of memory");
        return NULL;
    }
    cfg->arch = strdup("x86_64");
    cfg->accel = EXR_ACCEL_AUTO;
    cfg->memory = strdup("512M");
    cfg->profile = EXR_PROFILE_LOW_MEM_2GB;
    cfg->smp = 1;
    cfg->cpu = strdup("max");
    cfg->display = strdup("none");
    cfg->network_backend = strdup("none");
    return cfg;
}

void exr_config_free(ExRConfigHandle *cfg)
{
    if (!cfg) {
        return;
    }
    free(cfg->engine_path);
    free(cfg->arch);
    free(cfg->memory);
    free(cfg->cpu);
    free(cfg->display);
    free(cfg->audio_backend);
    free(cfg->audio_wav_path);
    free(cfg->shared_folder_path);
    free(cfg->kernel);
    free(cfg->initrd);
    free(cfg->cmdline);
    free(cfg->disk_path);
    free(cfg->disk_format);
    free(cfg->disk_interface);
    for (size_t i = 0; i < cfg->extra_disk_count; i++) {
        free(cfg->extra_disks[i].path);
        free(cfg->extra_disks[i].format);
    }
    free(cfg->extra_disks);
    free(cfg->network_backend);
    free(cfg->control_socket);
    free(cfg);
}

static ExRConfigHandle *exr_config_clone(const ExRConfigHandle *src)
{
    ExRConfigHandle *dst = calloc(1, sizeof(*dst));
    if (!dst) {
        return NULL;
    }
    dst->engine_path = exr_strdup_safe(src->engine_path);
    dst->arch = exr_strdup_safe(src->arch);
    dst->accel = src->accel;
    dst->memory = exr_strdup_safe(src->memory);
    dst->profile = src->profile;
    dst->smp = src->smp;
    dst->cpu = exr_strdup_safe(src->cpu);
    dst->balloon = src->balloon;
    dst->display = exr_strdup_safe(src->display);
    dst->vgamem_mb = src->vgamem_mb;
    dst->inputs = src->inputs;
    dst->audio_backend = exr_strdup_safe(src->audio_backend);
    dst->audio_wav_path = exr_strdup_safe(src->audio_wav_path);
    dst->shared_folder_path = exr_strdup_safe(src->shared_folder_path);
    dst->shared_folder_ro = src->shared_folder_ro;
    dst->kernel = exr_strdup_safe(src->kernel);
    dst->initrd = exr_strdup_safe(src->initrd);
    dst->cmdline = exr_strdup_safe(src->cmdline);
    dst->disk_path = exr_strdup_safe(src->disk_path);
    dst->disk_format = exr_strdup_safe(src->disk_format);
    dst->disk_interface = exr_strdup_safe(src->disk_interface);
    if (src->extra_disk_count > 0) {
        dst->extra_disks = calloc(src->extra_disk_count, sizeof(ExRExtraDisk));
        if (dst->extra_disks) {
            dst->extra_disk_count = src->extra_disk_count;
            for (size_t i = 0; i < src->extra_disk_count; i++) {
                dst->extra_disks[i].path = exr_strdup_safe(src->extra_disks[i].path);
                dst->extra_disks[i].format = exr_strdup_safe(src->extra_disks[i].format);
            }
        }
    }
    dst->network_backend = exr_strdup_safe(src->network_backend);
    dst->adb_port = src->adb_port;
    dst->control_socket = exr_strdup_safe(src->control_socket);
    return dst;
}

exr_status_t exr_config_set_engine_path(ExRConfigHandle *cfg, const char *path)
{
    if (!cfg || !path) {
        exr_set_error("invalid argument");
        return EXR_ERR_INVALID_ARG;
    }
    free(cfg->engine_path);
    cfg->engine_path = strdup(path);
    return EXR_OK;
}

exr_status_t exr_config_set_arch(ExRConfigHandle *cfg, const char *arch)
{
    if (!cfg || !arch || (strcmp(arch, "x86_64") && strcmp(arch, "i386"))) {
        exr_set_error("architecture must be 'x86_64' or 'i386'");
        return EXR_ERR_INVALID_ARG;
    }
    free(cfg->arch);
    cfg->arch = strdup(arch);
    return EXR_OK;
}

exr_status_t exr_config_set_accel(ExRConfigHandle *cfg, exr_accel_t accel)
{
    if (!cfg || accel < EXR_ACCEL_AUTO || accel > EXR_ACCEL_HVF) {
        exr_set_error("invalid accelerator");
        return EXR_ERR_INVALID_ARG;
    }
    cfg->accel = accel;
    return EXR_OK;
}

exr_status_t exr_config_set_memory(ExRConfigHandle *cfg, const char *memory_str)
{
    if (!cfg || !memory_str || !*memory_str) {
        exr_set_error("invalid memory string");
        return EXR_ERR_INVALID_ARG;
    }
    free(cfg->memory);
    cfg->memory = strdup(memory_str);
    return EXR_OK;
}

exr_status_t exr_config_set_profile(ExRConfigHandle *cfg, exr_memory_profile_t profile)
{
    if (!cfg || profile < EXR_PROFILE_STANDARD || profile > EXR_PROFILE_MINIMAL) {
        exr_set_error("invalid memory profile");
        return EXR_ERR_INVALID_ARG;
    }
    cfg->profile = profile;
    return EXR_OK;
}

exr_status_t exr_config_set_smp(ExRConfigHandle *cfg, uint16_t smp)
{
    if (!cfg || smp == 0) {
        exr_set_error("smp must be >= 1");
        return EXR_ERR_INVALID_ARG;
    }
    cfg->smp = smp;
    return EXR_OK;
}

exr_status_t exr_config_set_cpu(ExRConfigHandle *cfg, const char *cpu_model)
{
    if (!cfg || !cpu_model || !*cpu_model) {
        exr_set_error("invalid cpu model");
        return EXR_ERR_INVALID_ARG;
    }
    free(cfg->cpu);
    cfg->cpu = strdup(cpu_model);
    return EXR_OK;
}

exr_status_t exr_config_set_balloon(ExRConfigHandle *cfg, bool enabled)
{
    if (!cfg) {
        return EXR_ERR_INVALID_ARG;
    }
    cfg->balloon = enabled;
    return EXR_OK;
}

exr_status_t exr_config_set_display(ExRConfigHandle *cfg, const char *display)
{
    if (!cfg || !display ||
        (strcmp(display, "none") && strcmp(display, "auto") &&
         strcmp(display, "virtio-gpu") && strcmp(display, "virtio-gpu-gl"))) {
        exr_set_error("display must be 'none', 'auto', 'virtio-gpu', or 'virtio-gpu-gl'");
        return EXR_ERR_INVALID_ARG;
    }
    free(cfg->display);
    cfg->display = strdup(display);
    return EXR_OK;
}

exr_status_t exr_config_set_vgamem(ExRConfigHandle *cfg, uint16_t vgamem_mb)
{
    if (!cfg) {
        return EXR_ERR_INVALID_ARG;
    }
    cfg->vgamem_mb = vgamem_mb;
    return EXR_OK;
}

exr_status_t exr_config_set_inputs(ExRConfigHandle *cfg, bool enabled)
{
    if (!cfg) {
        return EXR_ERR_INVALID_ARG;
    }
    cfg->inputs = enabled;
    return EXR_OK;
}

exr_status_t exr_config_set_audio(ExRConfigHandle *cfg,
                                  const char *backend,
                                  const char *wav_path)
{
    if (!cfg || !backend ||
        (strcmp(backend, "none") && strcmp(backend, "wav") && strcmp(backend, "auto"))) {
        exr_set_error("audio backend must be 'none', 'wav', or 'auto'");
        return EXR_ERR_INVALID_ARG;
    }
    free(cfg->audio_backend);
    free(cfg->audio_wav_path);
    cfg->audio_backend = strdup(backend);
    cfg->audio_wav_path = exr_strdup_safe(wav_path);
    return EXR_OK;
}

exr_status_t exr_config_set_adb_port(ExRConfigHandle *cfg, uint16_t port)
{
    if (!cfg) {
        return EXR_ERR_INVALID_ARG;
    }
    cfg->adb_port = port;
    return EXR_OK;
}

exr_status_t exr_config_set_shared_folder(ExRConfigHandle *cfg,
                                          const char *host_path,
                                          bool readonly)
{
    if (!cfg || !host_path || !*host_path) {
        exr_set_error("shared folder path is required");
        return EXR_ERR_INVALID_ARG;
    }
    free(cfg->shared_folder_path);
    cfg->shared_folder_path = strdup(host_path);
    cfg->shared_folder_ro = readonly;
    return EXR_OK;
}

exr_status_t exr_config_add_extra_disk(ExRConfigHandle *cfg,
                                       const char *path,
                                       const char *format)
{
    if (!cfg || !path || !*path) {
        exr_set_error("extra disk path is required");
        return EXR_ERR_INVALID_ARG;
    }
    ExRExtraDisk *next = realloc(cfg->extra_disks,
                                 (cfg->extra_disk_count + 1) * sizeof(ExRExtraDisk));
    if (!next) {
        exr_set_error("out of memory");
        return EXR_ERR_FAILED;
    }
    cfg->extra_disks = next;
    cfg->extra_disks[cfg->extra_disk_count].path = strdup(path);
    cfg->extra_disks[cfg->extra_disk_count].format = strdup(format ? format : "exrimg");
    cfg->extra_disk_count++;
    return EXR_OK;
}

exr_status_t exr_config_set_kernel(ExRConfigHandle *cfg,
                                   const char *kernel_path,
                                   const char *initrd_path,
                                   const char *cmdline)
{
    if (!cfg || !kernel_path) {
        exr_set_error("kernel_path is required");
        return EXR_ERR_INVALID_ARG;
    }
    free(cfg->kernel);
    free(cfg->initrd);
    free(cfg->cmdline);
    cfg->kernel = strdup(kernel_path);
    cfg->initrd = exr_strdup_safe(initrd_path);
    cfg->cmdline = exr_strdup_safe(cmdline);
    return EXR_OK;
}

exr_status_t exr_config_set_disk(ExRConfigHandle *cfg,
                                 const char *path,
                                 const char *format,
                                 const char *interface)
{
    if (!cfg || !path) {
        exr_set_error("disk path is required");
        return EXR_ERR_INVALID_ARG;
    }
    free(cfg->disk_path);
    free(cfg->disk_format);
    free(cfg->disk_interface);
    cfg->disk_path = strdup(path);
    cfg->disk_format = strdup(format ? format : "exrimg");
    cfg->disk_interface = strdup(interface ? interface : "virtio");
    return EXR_OK;
}

exr_status_t exr_config_set_network(ExRConfigHandle *cfg, const char *backend)
{
    if (!cfg || !backend) {
        return EXR_ERR_INVALID_ARG;
    }
    free(cfg->network_backend);
    cfg->network_backend = strdup(backend);
    return EXR_OK;
}

exr_status_t exr_config_set_control_socket(ExRConfigHandle *cfg, const char *socket_path)
{
    if (!cfg || !socket_path) {
        return EXR_ERR_INVALID_ARG;
    }
    free(cfg->control_socket);
    cfg->control_socket = strdup(socket_path);
    return EXR_OK;
}

static void exr_write_json_str(FILE *fp, const char *s)
{
    fputc('"', fp);
    for (; *s; s++) {
        if (*s == '"' || *s == '\\') {
            fputc('\\', fp);
        }
        fputc(*s, fp);
    }
    fputc('"', fp);
}

static const char *exr_accel_str(exr_accel_t accel)
{
    switch (accel) {
    case EXR_ACCEL_TCG: return "tcg";
    case EXR_ACCEL_KVM: return "kvm";
    case EXR_ACCEL_WHPX: return "whpx";
    case EXR_ACCEL_HVF: return "hvf";
    case EXR_ACCEL_AUTO:
    default: return "auto";
    }
}

static const char *exr_profile_str(exr_memory_profile_t p)
{
    switch (p) {
    case EXR_PROFILE_LOW_MEM_2GB: return "low-mem-2gb";
    case EXR_PROFILE_MINIMAL: return "minimal";
    case EXR_PROFILE_STANDARD:
    default: return "standard";
    }
}

static bool exr_write_config_file(const ExRConfigHandle *cfg,
                                  const char *config_path,
                                  const char *socket_path)
{
    FILE *fp = fopen(config_path, "w");
    if (!fp) {
        exr_set_error("cannot write config '%s': %s", config_path, strerror(errno));
        return false;
    }
    fprintf(fp, "{\n");
    fprintf(fp, "  \"schema-version\": 1,\n");
    fprintf(fp, "  \"engine\": {\n");
    fprintf(fp, "    \"architecture\": "); exr_write_json_str(fp, cfg->arch); fprintf(fp, ",\n");
    fprintf(fp, "    \"acceleration\": "); exr_write_json_str(fp, exr_accel_str(cfg->accel)); fprintf(fp, ",\n");
    fprintf(fp, "    \"memory\": "); exr_write_json_str(fp, cfg->memory); fprintf(fp, ",\n");
    fprintf(fp, "    \"cpu\": "); exr_write_json_str(fp, cfg->cpu); fprintf(fp, ",\n");
    fprintf(fp, "    \"smp\": %u,\n", cfg->smp);
    fprintf(fp, "    \"profile\": "); exr_write_json_str(fp, exr_profile_str(cfg->profile)); fprintf(fp, ",\n");
    fprintf(fp, "    \"balloon\": %s,\n", cfg->balloon ? "true" : "false");
    fprintf(fp, "    \"inputs\": %s,\n", cfg->inputs ? "true" : "false");
    if (cfg->vgamem_mb > 0) {
        fprintf(fp, "    \"vgamem-mb\": %u,\n", cfg->vgamem_mb);
    }
    fprintf(fp, "    \"display\": "); exr_write_json_str(fp, cfg->display ? cfg->display : "none"); fprintf(fp, "\n");
    fprintf(fp, "  }");

    if (cfg->disk_path) {
        fprintf(fp, ",\n  \"storage\": {\n");
        fprintf(fp, "    \"path\": "); exr_write_json_str(fp, cfg->disk_path); fprintf(fp, ",\n");
        fprintf(fp, "    \"format\": "); exr_write_json_str(fp, cfg->disk_format); fprintf(fp, ",\n");
        fprintf(fp, "    \"interface\": "); exr_write_json_str(fp, cfg->disk_interface); fprintf(fp, "\n");
        fprintf(fp, "  }");
    }

    if (cfg->extra_disk_count > 0) {
        fprintf(fp, ",\n  \"extra-storage\": [\n");
        for (size_t i = 0; i < cfg->extra_disk_count; i++) {
            fprintf(fp, "    {\"path\": ");
            exr_write_json_str(fp, cfg->extra_disks[i].path);
            fprintf(fp, ", \"format\": ");
            exr_write_json_str(fp, cfg->extra_disks[i].format);
            fprintf(fp, ", \"interface\": \"virtio\"}%s\n",
                    (i + 1 < cfg->extra_disk_count) ? "," : "");
        }
        fprintf(fp, "  ]");
    }

    if (cfg->shared_folder_path) {
        fprintf(fp, ",\n  \"shared-folder\": {\n");
        fprintf(fp, "    \"path\": "); exr_write_json_str(fp, cfg->shared_folder_path); fprintf(fp, ",\n");
        fprintf(fp, "    \"readonly\": %s\n", cfg->shared_folder_ro ? "true" : "false");
        fprintf(fp, "  }");
    }

    if (cfg->audio_backend) {
        fprintf(fp, ",\n  \"audio\": {\n");
        fprintf(fp, "    \"backend\": "); exr_write_json_str(fp, cfg->audio_backend);
        if (cfg->audio_wav_path) {
            fprintf(fp, ",\n    \"wav-path\": "); exr_write_json_str(fp, cfg->audio_wav_path);
        }
        fprintf(fp, "\n  }");
    }

    if (cfg->kernel) {
        fprintf(fp, ",\n  \"boot\": {\n");
        fprintf(fp, "    \"kernel\": "); exr_write_json_str(fp, cfg->kernel);
        if (cfg->initrd) {
            fprintf(fp, ",\n    \"initrd\": "); exr_write_json_str(fp, cfg->initrd);
        }
        if (cfg->cmdline) {
            fprintf(fp, ",\n    \"cmdline\": "); exr_write_json_str(fp, cfg->cmdline);
        }
        fprintf(fp, "\n  }");
    }

    if (cfg->network_backend) {
        fprintf(fp, ",\n  \"networking\": {\n");
        fprintf(fp, "    \"backend\": "); exr_write_json_str(fp, cfg->network_backend);
        if (cfg->adb_port > 0) {
            fprintf(fp, ",\n    \"adb-port\": %u", cfg->adb_port);
        }
        fprintf(fp, "\n  }");
    }

    fprintf(fp, ",\n  \"runtime\": {\n");
    fprintf(fp, "    \"serial\": \"stdio\"");
    if (socket_path) {
        fprintf(fp, ",\n    \"socket\": \"unix:");
        for (const char *s = socket_path; *s; s++) {
            if (*s == '"' || *s == '\\') {
                fputc('\\', fp);
            }
            fputc(*s, fp);
        }
        fprintf(fp, "\"");
    }
    fprintf(fp, "\n  }\n}\n");

    fclose(fp);
    return true;
}

exr_status_t exr_config_save(const ExRConfigHandle *cfg, const char *exr_path)
{
    if (!cfg || !exr_path || !*exr_path) {
        exr_set_error("invalid argument");
        return EXR_ERR_INVALID_ARG;
    }
    return exr_write_config_file(cfg, exr_path, cfg->control_socket) ? EXR_OK : EXR_ERR_IO;
}

static char *exr_json_extract_str(const char *json, const char *key)
{
    char pattern[128];
    snprintf(pattern, sizeof(pattern), "\"%s\"", key);
    const char *p = strstr(json, pattern);
    if (!p) return NULL;
    p += strlen(pattern);
    while (*p == ' ' || *p == '\t' || *p == ':') p++;
    if (*p != '"') return NULL;
    p++;
    const char *end = strchr(p, '"');
    if (!end) return NULL;
    return strndup(p, (size_t)(end - p));
}

static bool exr_json_extract_int(const char *json, const char *key, long *out_val)
{
    char pattern[128];
    snprintf(pattern, sizeof(pattern), "\"%s\"", key);
    const char *p = strstr(json, pattern);
    if (!p) return false;
    p += strlen(pattern);
    while (*p == ' ' || *p == '\t' || *p == ':') p++;
    char *end = NULL;
    long v = strtol(p, &end, 10);
    if (end == p) return false;
    *out_val = v;
    return true;
}

static bool exr_json_extract_bool(const char *json, const char *key, bool *out_val)
{
    char pattern[128];
    snprintf(pattern, sizeof(pattern), "\"%s\"", key);
    const char *p = strstr(json, pattern);
    if (!p) return false;
    p += strlen(pattern);
    while (*p == ' ' || *p == '\t' || *p == ':') p++;
    if (strncmp(p, "true", 4) == 0) { *out_val = true; return true; }
    if (strncmp(p, "false", 5) == 0) { *out_val = false; return true; }
    return false;
}

exr_status_t exr_config_load(const char *exr_path, ExRConfigHandle **out_cfg)
{
    if (!exr_path || !out_cfg) {
        exr_set_error("invalid argument");
        return EXR_ERR_INVALID_ARG;
    }
    FILE *fp = fopen(exr_path, "r");
    if (!fp) {
        exr_set_error("cannot open '%s': %s", exr_path, strerror(errno));
        return EXR_ERR_IO;
    }
    fseek(fp, 0, SEEK_END);
    long sz = ftell(fp);
    fseek(fp, 0, SEEK_SET);
    if (sz <= 0 || sz > 1024 * 1024) {
        fclose(fp);
        exr_set_error("invalid config.exr size");
        return EXR_ERR_INVALID_ARG;
    }
    char *buf = malloc((size_t)sz + 1);
    if (!buf) {
        fclose(fp);
        return EXR_ERR_FAILED;
    }
    size_t n = fread(buf, 1, (size_t)sz, fp);
    buf[n] = '\0';
    fclose(fp);

    ExRConfigHandle *cfg = exr_config_create();
    if (!cfg) {
        free(buf);
        return EXR_ERR_FAILED;
    }
    char *val = NULL;
    long lval = 0;
    bool bval = false;

    if ((val = exr_json_extract_str(buf, "architecture"))) { exr_config_set_arch(cfg, val); free(val); }
    if ((val = exr_json_extract_str(buf, "memory"))) { exr_config_set_memory(cfg, val); free(val); }
    if ((val = exr_json_extract_str(buf, "cpu"))) { exr_config_set_cpu(cfg, val); free(val); }
    if ((val = exr_json_extract_str(buf, "display"))) { exr_config_set_display(cfg, val); free(val); }
    if (exr_json_extract_int(buf, "smp", &lval) && lval > 0) exr_config_set_smp(cfg, (uint16_t)lval);
    if (exr_json_extract_int(buf, "vgamem-mb", &lval) && lval > 0) exr_config_set_vgamem(cfg, (uint16_t)lval);
    if (exr_json_extract_int(buf, "adb-port", &lval) && lval > 0) exr_config_set_adb_port(cfg, (uint16_t)lval);
    if (exr_json_extract_bool(buf, "balloon", &bval)) exr_config_set_balloon(cfg, bval);
    if (exr_json_extract_bool(buf, "inputs", &bval)) exr_config_set_inputs(cfg, bval);
    if ((val = exr_json_extract_str(buf, "backend"))) { exr_config_set_network(cfg, val); free(val); }
    if ((val = exr_json_extract_str(buf, "acceleration"))) {
        if (strcmp(val, "tcg") == 0) cfg->accel = EXR_ACCEL_TCG;
        else if (strcmp(val, "kvm") == 0) cfg->accel = EXR_ACCEL_KVM;
        else if (strcmp(val, "whpx") == 0) cfg->accel = EXR_ACCEL_WHPX;
        else if (strcmp(val, "hvf") == 0) cfg->accel = EXR_ACCEL_HVF;
        else cfg->accel = EXR_ACCEL_AUTO;
        free(val);
    }

    free(buf);
    *out_cfg = cfg;
    return EXR_OK;
}

static void exr_emit_event(ExRMachine *m, exr_machine_state_t state, const char *msg)
{
    m->state = state;
    if (m->event_cb) {
        m->event_cb(state, msg, m->event_opaque);
    }
}

static void *exr_serial_reader(void *arg)
{
    ExRMachine *m = arg;
    uint8_t buf[4096];

    for (;;) {
        ssize_t n = read(m->stdout_fd, buf, sizeof(buf));
        if (n > 0) {
            if (m->serial_cb) {
                m->serial_cb(buf, (size_t)n, m->serial_opaque);
            }
            continue;
        }
        if (n == 0 || (errno != EINTR && errno != EAGAIN)) {
            break;
        }
    }
    return NULL;
}

static bool exr_ctrl_send_cmd(const char *socket_path, const char *cmd_json)
{
    struct sockaddr_un addr;
    int fd;
    char buf[4096];
    ssize_t n;

    if (!socket_path) {
        exr_set_error("no control socket configured");
        return false;
    }
    if (strlen(socket_path) >= sizeof(addr.sun_path)) {
        exr_set_error("control socket path too long");
        return false;
    }

    fd = socket(AF_UNIX, SOCK_STREAM, 0);
    if (fd < 0) {
        exr_set_error("socket() failed: %s", strerror(errno));
        return false;
    }

    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    strcpy(addr.sun_path, socket_path);

    if (connect(fd, (struct sockaddr *)&addr, sizeof(addr)) < 0) {
        exr_set_error("connect(%s) failed: %s", socket_path, strerror(errno));
        close(fd);
        return false;
    }

    /* Drain greeting */
    n = read(fd, buf, sizeof(buf) - 1);
    if (n <= 0) {
        exr_set_error("failed reading control greeting");
        close(fd);
        return false;
    }

    /* Negotiate capabilities */
    const char *cap = "{\"execute\":\"qmp_capabilities\"}\r\n";
    if (write(fd, cap, strlen(cap)) < 0) {
        exr_set_error("failed sending capabilities");
        close(fd);
        return false;
    }
    n = read(fd, buf, sizeof(buf) - 1);
    if (n <= 0) {
        exr_set_error("failed reading capabilities response");
        close(fd);
        return false;
    }

    /* Send command */
    if (write(fd, cmd_json, strlen(cmd_json)) < 0 ||
        write(fd, "\r\n", 2) < 0) {
        exr_set_error("failed sending control command");
        close(fd);
        return false;
    }

    n = read(fd, buf, sizeof(buf) - 1);
    close(fd);
    if (n <= 0) {
        exr_set_error("no response from control socket");
        return false;
    }
    buf[n] = '\0';
    if (strstr(buf, "\"error\"") != NULL) {
        exr_set_error("control command returned error: %s", buf);
        return false;
    }
    return true;
}

exr_status_t exr_machine_create(const ExRConfigHandle *cfg, ExRMachine **out_machine)
{
    if (!cfg || !out_machine) {
        exr_set_error("invalid argument");
        return EXR_ERR_INVALID_ARG;
    }

    ExRMachine *m = calloc(1, sizeof(*m));
    if (!m) {
        exr_set_error("out of memory");
        return EXR_ERR_FAILED;
    }
    pthread_mutex_init(&m->lock, NULL);
    m->cfg = exr_config_clone(cfg);
    m->state = EXR_STATE_STOPPED;
    m->pid = -1;
    m->stdin_fd = -1;
    m->stdout_fd = -1;

    char tmpl[] = "/tmp/exr-rt-XXXXXX";
    if (!mkdtemp(tmpl)) {
        exr_set_error("mkdtemp failed: %s", strerror(errno));
        exr_machine_destroy(m);
        return EXR_ERR_IO;
    }
    m->runtime_dir = strdup(tmpl);
    if (asprintf(&m->config_path, "%s/config.json", m->runtime_dir) < 0) {
        exr_machine_destroy(m);
        return EXR_ERR_FAILED;
    }

    if (cfg->control_socket) {
        m->socket_path = strdup(cfg->control_socket);
        m->owns_socket = false;
    } else {
        if (asprintf(&m->socket_path, "%s/ctrl.sock", m->runtime_dir) < 0) {
            exr_machine_destroy(m);
            return EXR_ERR_FAILED;
        }
        m->owns_socket = true;
    }

    *out_machine = m;
    return EXR_OK;
}

exr_status_t exr_machine_start(ExRMachine *m)
{
    if (!m) {
        return EXR_ERR_INVALID_ARG;
    }
    pthread_mutex_lock(&m->lock);
    if (m->state != EXR_STATE_STOPPED) {
        pthread_mutex_unlock(&m->lock);
        exr_set_error("machine is not stopped");
        return EXR_ERR_STATE;
    }

    if (!exr_write_config_file(m->cfg, m->config_path, m->socket_path)) {
        pthread_mutex_unlock(&m->lock);
        return EXR_ERR_IO;
    }

    int in_pipe[2], out_pipe[2];
    if (pipe(in_pipe) < 0 || pipe(out_pipe) < 0) {
        exr_set_error("pipe() failed: %s", strerror(errno));
        pthread_mutex_unlock(&m->lock);
        return EXR_ERR_IO;
    }

    exr_emit_event(m, EXR_STATE_STARTING, "starting engine");

    const char *bin = m->cfg->engine_path ? m->cfg->engine_path : "exr-x86";

    pid_t pid = fork();
    if (pid < 0) {
        close(in_pipe[0]); close(in_pipe[1]);
        close(out_pipe[0]); close(out_pipe[1]);
        exr_emit_event(m, EXR_STATE_ERROR, "fork failed");
        pthread_mutex_unlock(&m->lock);
        return EXR_ERR_FAILED;
    }

    if (pid == 0) {
        dup2(in_pipe[0], STDIN_FILENO);
        dup2(out_pipe[1], STDOUT_FILENO);
        dup2(out_pipe[1], STDERR_FILENO);
        close(in_pipe[0]); close(in_pipe[1]);
        close(out_pipe[0]); close(out_pipe[1]);
        execlp(bin, bin, "run", m->config_path, (char *)NULL);
        _exit(127);
    }

    close(in_pipe[0]);
    close(out_pipe[1]);
    m->pid = pid;
    m->stdin_fd = in_pipe[1];
    m->stdout_fd = out_pipe[0];

    if (pthread_create(&m->io_thread, NULL, exr_serial_reader, m) == 0) {
        m->io_thread_started = true;
    }

    /* Wait briefly for control socket to become ready */
    bool ready = false;
    for (int i = 0; i < 50; i++) {
        struct stat st;
        if (stat(m->socket_path, &st) == 0) {
            ready = true;
            break;
        }
        int st_code = 0;
        if (waitpid(m->pid, &st_code, WNOHANG) == m->pid) {
            m->pid = -1;
            exr_emit_event(m, EXR_STATE_ERROR, "engine exited prematurely");
            pthread_mutex_unlock(&m->lock);
            return EXR_ERR_FAILED;
        }
        usleep(50000);
    }

    if (!ready) {
        exr_emit_event(m, EXR_STATE_ERROR, "control socket timeout");
        pthread_mutex_unlock(&m->lock);
        return EXR_ERR_FAILED;
    }

    exr_emit_event(m, EXR_STATE_RUNNING, "running");
    pthread_mutex_unlock(&m->lock);
    return EXR_OK;
}

exr_status_t exr_machine_pause(ExRMachine *m)
{
    if (!m) return EXR_ERR_INVALID_ARG;
    pthread_mutex_lock(&m->lock);
    if (m->state != EXR_STATE_RUNNING) {
        pthread_mutex_unlock(&m->lock);
        return EXR_ERR_STATE;
    }
    bool ok = exr_ctrl_send_cmd(m->socket_path, "{\"execute\":\"stop\"}");
    if (ok) {
        exr_emit_event(m, EXR_STATE_PAUSED, "paused");
    }
    pthread_mutex_unlock(&m->lock);
    return ok ? EXR_OK : EXR_ERR_FAILED;
}

exr_status_t exr_machine_resume(ExRMachine *m)
{
    if (!m) return EXR_ERR_INVALID_ARG;
    pthread_mutex_lock(&m->lock);
    if (m->state != EXR_STATE_PAUSED) {
        pthread_mutex_unlock(&m->lock);
        return EXR_ERR_STATE;
    }
    bool ok = exr_ctrl_send_cmd(m->socket_path, "{\"execute\":\"cont\"}");
    if (ok) {
        exr_emit_event(m, EXR_STATE_RUNNING, "resumed");
    }
    pthread_mutex_unlock(&m->lock);
    return ok ? EXR_OK : EXR_ERR_FAILED;
}

exr_status_t exr_machine_reset(ExRMachine *m)
{
    if (!m) return EXR_ERR_INVALID_ARG;
    pthread_mutex_lock(&m->lock);
    bool ok = exr_ctrl_send_cmd(m->socket_path, "{\"execute\":\"system_reset\"}");
    pthread_mutex_unlock(&m->lock);
    return ok ? EXR_OK : EXR_ERR_FAILED;
}

exr_status_t exr_machine_stop(ExRMachine *m)
{
    if (!m) return EXR_ERR_INVALID_ARG;
    pthread_mutex_lock(&m->lock);
    if (m->pid <= 0) {
        m->state = EXR_STATE_STOPPED;
        pthread_mutex_unlock(&m->lock);
        return EXR_OK;
    }
    exr_ctrl_send_cmd(m->socket_path, "{\"execute\":\"quit\"}");
    int status = 0;
    for (int i = 0; i < 40; i++) {
        if (waitpid(m->pid, &status, WNOHANG) == m->pid) {
            m->pid = -1;
            break;
        }
        usleep(50000);
    }
    if (m->pid > 0) {
        kill(m->pid, SIGTERM);
        waitpid(m->pid, &status, 0);
        m->pid = -1;
    }
    exr_emit_event(m, EXR_STATE_STOPPED, "stopped");
    pthread_mutex_unlock(&m->lock);
    return EXR_OK;
}

exr_status_t exr_machine_wait(ExRMachine *m, int timeout_ms)
{
    if (!m) return EXR_ERR_INVALID_ARG;
    if (m->pid <= 0) return EXR_OK;

    int elapsed = 0;
    int status = 0;
    while (timeout_ms < 0 || elapsed < timeout_ms) {
        pid_t r = waitpid(m->pid, &status, WNOHANG);
        if (r == m->pid) {
            pthread_mutex_lock(&m->lock);
            m->pid = -1;
            exr_emit_event(m, EXR_STATE_STOPPED, "exited");
            pthread_mutex_unlock(&m->lock);
            return EXR_OK;
        }
        usleep(20000);
        elapsed += 20;
    }
    return EXR_ERR_STATE;
}

exr_machine_state_t exr_machine_get_state(ExRMachine *m)
{
    if (!m) return EXR_STATE_ERROR;
    return m->state;
}

exr_status_t exr_machine_snapshot_save(ExRMachine *m, const char *name)
{
    if (!m || !name || !*name) return EXR_ERR_INVALID_ARG;
    char *cmd = NULL;
    if (asprintf(&cmd, "{\"execute\":\"human-monitor-command\",\"arguments\":{\"command-line\":\"savevm %s\"}}", name) < 0) {
        return EXR_ERR_FAILED;
    }
    pthread_mutex_lock(&m->lock);
    bool ok = exr_ctrl_send_cmd(m->socket_path, cmd);
    pthread_mutex_unlock(&m->lock);
    free(cmd);
    return ok ? EXR_OK : EXR_ERR_FAILED;
}

exr_status_t exr_machine_snapshot_load(ExRMachine *m, const char *name)
{
    if (!m || !name || !*name) return EXR_ERR_INVALID_ARG;
    char *cmd = NULL;
    if (asprintf(&cmd, "{\"execute\":\"human-monitor-command\",\"arguments\":{\"command-line\":\"loadvm %s\"}}", name) < 0) {
        return EXR_ERR_FAILED;
    }
    pthread_mutex_lock(&m->lock);
    bool ok = exr_ctrl_send_cmd(m->socket_path, cmd);
    pthread_mutex_unlock(&m->lock);
    free(cmd);
    return ok ? EXR_OK : EXR_ERR_FAILED;
}

exr_status_t exr_machine_snapshot_delete(ExRMachine *m, const char *name)
{
    if (!m || !name || !*name) return EXR_ERR_INVALID_ARG;
    char *cmd = NULL;
    if (asprintf(&cmd, "{\"execute\":\"human-monitor-command\",\"arguments\":{\"command-line\":\"delvm %s\"}}", name) < 0) {
        return EXR_ERR_FAILED;
    }
    pthread_mutex_lock(&m->lock);
    bool ok = exr_ctrl_send_cmd(m->socket_path, cmd);
    pthread_mutex_unlock(&m->lock);
    free(cmd);
    return ok ? EXR_OK : EXR_ERR_FAILED;
}

exr_status_t exr_machine_set_event_callback(ExRMachine *m, exr_event_cb_t cb, void *opaque)
{
    if (!m) return EXR_ERR_INVALID_ARG;
    m->event_cb = cb;
    m->event_opaque = opaque;
    return EXR_OK;
}

exr_status_t exr_machine_set_serial_output_callback(ExRMachine *m, exr_serial_output_cb_t cb, void *opaque)
{
    if (!m) return EXR_ERR_INVALID_ARG;
    m->serial_cb = cb;
    m->serial_opaque = opaque;
    return EXR_OK;
}

exr_status_t exr_machine_serial_input(ExRMachine *m, const uint8_t *data, size_t len)
{
    if (!m || !data || m->stdin_fd < 0) return EXR_ERR_INVALID_ARG;
    ssize_t written = write(m->stdin_fd, data, len);
    return (written == (ssize_t)len) ? EXR_OK : EXR_ERR_IO;
}

exr_status_t exr_machine_send_key(ExRMachine *m, const char *qcode, bool down)
{
    if (!m || !qcode || !*qcode) return EXR_ERR_INVALID_ARG;
    char *cmd = NULL;
    if (asprintf(&cmd,
                 "{\"execute\":\"input-send-event\",\"arguments\":{\"events\":[{\"type\":\"key\",\"data\":{\"down\":%s,\"key\":{\"type\":\"qcode\",\"data\":\"%s\"}}}]}}",
                 down ? "true" : "false", qcode) < 0) {
        return EXR_ERR_FAILED;
    }
    pthread_mutex_lock(&m->lock);
    bool ok = exr_ctrl_send_cmd(m->socket_path, cmd);
    pthread_mutex_unlock(&m->lock);
    free(cmd);
    return ok ? EXR_OK : EXR_ERR_FAILED;
}

exr_status_t exr_machine_send_pointer_abs(ExRMachine *m, int32_t x, int32_t y)
{
    if (!m) return EXR_ERR_INVALID_ARG;
    if (x < 0) x = 0;
    if (x > 32767) x = 32767;
    if (y < 0) y = 0;
    if (y > 32767) y = 32767;
    char *cmd = NULL;
    if (asprintf(&cmd,
                 "{\"execute\":\"input-send-event\",\"arguments\":{\"events\":[{\"type\":\"abs\",\"data\":{\"axis\":\"x\",\"value\":%d}},{\"type\":\"abs\",\"data\":{\"axis\":\"y\",\"value\":%d}}]}}",
                 x, y) < 0) {
        return EXR_ERR_FAILED;
    }
    pthread_mutex_lock(&m->lock);
    bool ok = exr_ctrl_send_cmd(m->socket_path, cmd);
    pthread_mutex_unlock(&m->lock);
    free(cmd);
    return ok ? EXR_OK : EXR_ERR_FAILED;
}

exr_status_t exr_machine_send_pointer_btn(ExRMachine *m, exr_mouse_button_t btn, bool down)
{
    if (!m) return EXR_ERR_INVALID_ARG;
    const char *btn_str = "left";
    if (btn == EXR_MOUSE_MIDDLE) btn_str = "middle";
    else if (btn == EXR_MOUSE_RIGHT) btn_str = "right";

    char *cmd = NULL;
    if (asprintf(&cmd,
                 "{\"execute\":\"input-send-event\",\"arguments\":{\"events\":[{\"type\":\"btn\",\"data\":{\"down\":%s,\"button\":\"%s\"}}]}}",
                 down ? "true" : "false", btn_str) < 0) {
        return EXR_ERR_FAILED;
    }
    pthread_mutex_lock(&m->lock);
    bool ok = exr_ctrl_send_cmd(m->socket_path, cmd);
    pthread_mutex_unlock(&m->lock);
    free(cmd);
    return ok ? EXR_OK : EXR_ERR_FAILED;
}

exr_status_t exr_machine_get_stats(ExRMachine *m, exr_machine_stats_t *out_stats)
{
    if (!m || !out_stats) return EXR_ERR_INVALID_ARG;
    memset(out_stats, 0, sizeof(*out_stats));
    pthread_mutex_lock(&m->lock);
    out_stats->state = m->state;
    out_stats->pid = (int32_t)m->pid;
    if (m->pid > 0) {
        char path[64];
        snprintf(path, sizeof(path), "/proc/%d/status", (int)m->pid);
        FILE *fp = fopen(path, "r");
        if (fp) {
            char line[256];
            while (fgets(line, sizeof(line), fp)) {
                unsigned long kb = 0;
                if (sscanf(line, "VmRSS: %lu kB", &kb) == 1) {
                    out_stats->rss_kib = kb;
                    break;
                }
            }
            fclose(fp);
        }
    }
    pthread_mutex_unlock(&m->lock);
    return EXR_OK;
}

exr_status_t exr_machine_set_balloon_target(ExRMachine *m, uint64_t target_bytes)
{
    if (!m || target_bytes == 0) return EXR_ERR_INVALID_ARG;
    char *cmd = NULL;
    if (asprintf(&cmd,
                 "{\"execute\":\"balloon\",\"arguments\":{\"value\":%llu}}",
                 (unsigned long long)target_bytes) < 0) {
        return EXR_ERR_FAILED;
    }
    pthread_mutex_lock(&m->lock);
    bool ok = exr_ctrl_send_cmd(m->socket_path, cmd);
    pthread_mutex_unlock(&m->lock);
    free(cmd);
    return ok ? EXR_OK : EXR_ERR_FAILED;
}

exr_status_t exr_machine_capture_screen(ExRMachine *m, const char *ppm_path)
{
    if (!m || !ppm_path || !*ppm_path) return EXR_ERR_INVALID_ARG;
    char *cmd = NULL;
    if (asprintf(&cmd,
                 "{\"execute\":\"screendump\",\"arguments\":{\"filename\":\"%s\"}}",
                 ppm_path) < 0) {
        return EXR_ERR_FAILED;
    }
    pthread_mutex_lock(&m->lock);
    bool ok = exr_ctrl_send_cmd(m->socket_path, cmd);
    pthread_mutex_unlock(&m->lock);
    free(cmd);
    return ok ? EXR_OK : EXR_ERR_FAILED;
}

void exr_machine_destroy(ExRMachine *m)
{
    if (!m) return;
    if (m->pid > 0) {
        exr_machine_stop(m);
    }
    if (m->stdin_fd >= 0) {
        close(m->stdin_fd);
    }
    if (m->stdout_fd >= 0) {
        close(m->stdout_fd);
    }
    if (m->io_thread_started) {
        pthread_join(m->io_thread, NULL);
    }
    if (m->config_path) {
        unlink(m->config_path);
        free(m->config_path);
    }
    if (m->socket_path) {
        if (m->owns_socket) {
            unlink(m->socket_path);
        }
        free(m->socket_path);
    }
    if (m->runtime_dir) {
        rmdir(m->runtime_dir);
        free(m->runtime_dir);
    }
    exr_config_free(m->cfg);
    pthread_mutex_destroy(&m->lock);
    free(m);
}
