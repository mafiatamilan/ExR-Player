/*
 * ExR x86 public command interface.
 * Copyright (C) 2026 ExR x86 contributors
 *
 * This work is licensed under the GNU GPL version 2 or later.
 */

#include "qemu/osdep.h"
#include "exr/cli.h"

#include "block/block-global-state.h"
#include "system/block-backend.h"
#include "crypto/init.h"
#include "qapi/error.h"
#include "qapi/qapi-types-exr.h"
#include "qapi/qapi-types-sockets.h"
#include "qapi/qapi-visit-exr.h"
#include "qapi/qobject-input-visitor.h"
#include "qobject/qdict.h"
#include "qobject/qstring.h"
#include "qobject/qjson.h"
#include "qobject/qobject.h"
#include "qemu-version.h"
#include "qemu/cutils.h"
#include "qemu/error-report.h"
#include "qemu/help-texts.h"
#include "qemu/main-loop.h"
#include "qemu/module.h"
#include "qemu/sockets.h"
#include "qemu/target-info-qom.h"
#include "system/cpus.h"
#include "qobject/qjson.h"

#define EXR_VERSION "0.1.0-dev"
#define EXR_CONFIG_MAX_BYTES (1024 * 1024)
#define EXR_CTRL_MAX_BYTES (16 * 1024 * 1024)

static const char *exr_storage_driver(ExRStorageFormat format)
{
    switch (format) {
    case EX_R_STORAGE_FORMAT_EXRIMG:
        return "qcow2";
    case EX_R_STORAGE_FORMAT_RAW:
        return "raw";
    default:
        return "qcow2";
    }
}

static void exr_print_help(void)
{
    exr_print_version();
    printf("\n"
           "Usage: exr-x86 COMMAND [ARGUMENTS]\n\n"
           "  create CONFIG          create the configured disk without overwriting\n"
           "  resize CONFIG SIZE     resize the configured disk (e.g. 32G)\n"
           "  run CONFIG             run a machine in the foreground\n"
           "  stop CONFIG            request guest shutdown through control socket\n"
           "  inspect CONFIG         print current machine status as JSON\n"
           "  snapshot CONFIG NAME   save an internal VM and disk snapshot\n"
           "  devices                list emulated devices\n"
           "  version                print ExR x86 version\n"
           "  help                   print this help\n\n"
           "CONFIG is a JSON configuration file.\n");
}

void exr_print_version(void)
{
    printf("ExR x86 " EXR_VERSION " (x86/x86_64 virtualization & emulation engine)\n"
           "Copyright (C) 2026 ExR x86 contributors\n");
}

static int exr_fail(Error *err)
{
    error_report_err(err);
    return EXIT_FAILURE;
}

static char *exr_resolve_path(const char *config_path, const char *path)
{
    g_autofree char *directory = NULL;

    if (g_path_is_absolute(path)) {
        return g_strdup(path);
    }
    directory = g_path_get_dirname(config_path);
    return g_build_filename(directory, path, NULL);
}

static char *exr_resolve_socket(const char *config_path, const char *socket)
{
    const char prefix[] = "unix:";

    if (!g_str_has_prefix(socket, prefix) ||
        g_path_is_absolute(socket + sizeof(prefix) - 1)) {
        return g_strdup(socket);
    }
    g_autofree char *path = exr_resolve_path(config_path,
                                             socket + sizeof(prefix) - 1);
    return g_strdup_printf("%s%s", prefix, path);
}

static ExRConfig *exr_read_config(const char *path, Error **errp)
{
    g_autofree char *json = NULL;
    gsize length;
    GError *gerror = NULL;
    QObject *object;
    Visitor *visitor;
    ExRConfig *config = NULL;

    if (!g_file_get_contents(path, &json, &length, &gerror)) {
        error_setg(errp, "cannot read configuration '%s': %s",
                   path, gerror->message);
        g_error_free(gerror);
        return NULL;
    }
    if (length > EXR_CONFIG_MAX_BYTES) {
        error_setg(errp, "configuration '%s' exceeds 1 MiB", path);
        return NULL;
    }

    object = qobject_from_json(json, errp);
    if (!object) {
        error_prepend(errp, "invalid configuration '%s': ", path);
        return NULL;
    }
    visitor = qobject_input_visitor_new(object);
    visit_type_ExRConfig(visitor, path, &config, errp);
    visit_free(visitor);
    qobject_unref(object);
    if (!config) {
        return NULL;
    }
    if (config->schema_version != 1) {
        error_setg(errp, "unsupported configuration schema-version %" PRId64,
                   config->schema_version);
        qapi_free_ExRConfig(config);
        return NULL;
    }
    return config;
}

static bool exr_validate_size(const char *value, const char *field,
                              Error **errp)
{
    uint64_t size;

    if (qemu_strtosz(value, NULL, &size) < 0 || size == 0) {
        error_setg(errp, "%s must be a positive byte size, not '%s'",
                   field, value);
        return false;
    }
    return true;
}

static void exr_add_arg(GPtrArray *args, const char *value)
{
    g_ptr_array_add(args, g_strdup(value));
}

static void exr_take_arg(GPtrArray *args, char *value)
{
    g_ptr_array_add(args, value);
}

static void exr_add_blockdev(GPtrArray *args, const char *config_path,
                             ExRStorageConfig *st, const char *node_name,
                             bool low_mem)
{
    QDict *blockdev = qdict_new();
    QDict *file = qdict_new();
    g_autofree char *path = exr_resolve_path(config_path, st->path);
    GString *json;

    qdict_put_str(blockdev, "driver", exr_storage_driver(st->format));
    qdict_put_str(blockdev, "node-name", node_name);
    qdict_put_str(blockdev, "discard", "unmap");
    qdict_put_str(blockdev, "detect-zeroes", "unmap");
    if (st->format == EX_R_STORAGE_FORMAT_EXRIMG) {
        qdict_put_int(blockdev, "l2-cache-size", low_mem ? 1048576 : 4194304);
    }
    qdict_put_str(file, "driver", "file");
    qdict_put_str(file, "filename", path);
    qdict_put_str(file, "aio", "threads");
    qdict_put_obj(blockdev, "file", QOBJECT(file));
    json = qobject_to_json(QOBJECT(blockdev));
    exr_add_arg(args, "-blockdev");
    exr_take_arg(args, g_string_free(json, false));
    qobject_unref(blockdev);
    exr_add_arg(args, "-device");
    exr_take_arg(args, g_strdup_printf("virtio-blk-pci,drive=%s,iothread=exr-iothread0", node_name));
}

static int exr_prepare_run(const char *program, const char *config_path,
                           int *argc, char ***argv)
{
    Error *err = NULL;
    ExRConfig *config = exr_read_config(config_path, &err);
    GPtrArray *args;
    const char *acceleration;
    const char *machine = "pc";
    const char *serial = "stdio";

    if (!config) {
        return exr_fail(err);
    }
    if (strcmp(config->engine->architecture, "x86_64") &&
        strcmp(config->engine->architecture, "i386")) {
        error_setg(&err, "engine.architecture must be 'x86_64' or 'i386'");
        qapi_free_ExRConfig(config);
        return exr_fail(err);
    }
    if (!exr_validate_size(config->engine->memory, "engine.memory", &err)) {
        qapi_free_ExRConfig(config);
        return exr_fail(err);
    }
    if (config->engine->acceleration == EX_R_ACCELERATION_AUTO &&
        config->engine->cpu && !strcmp(config->engine->cpu, "host")) {
        error_setg(&err, "cpu=host requires an explicit hardware accelerator; "
                   "it cannot fall back to TCG with acceleration=auto");
        qapi_free_ExRConfig(config);
        return exr_fail(err);
    }

    const char *tb_size = config->engine->tb_size;
    if (!tb_size && config->engine->has_profile) {
        if (config->engine->profile == EX_R_MEMORY_PROFILE_LOW_MEM_2GB) {
            tb_size = "32";
        } else if (config->engine->profile == EX_R_MEMORY_PROFILE_MINIMAL) {
            tb_size = "16";
        }
    }

    if (config->engine->has_machine) {
        machine = ExRMachine_str(config->engine->machine);
    }

    bool has_gpu = config->engine->display != EX_R_DISPLAY_NONE;

    args = g_ptr_array_new_with_free_func(g_free);
    exr_add_arg(args, program);

    {
        g_autofree char *exe_dir = g_path_get_dirname(program);
        g_autofree char *bios_dir = g_build_filename(exe_dir, "..", "pc-bios", NULL);
        g_autofree char *bios_probe = g_build_filename(bios_dir, "bios-256k.bin", NULL);
        if (g_file_test(bios_probe, G_FILE_TEST_EXISTS)) {
            exr_add_arg(args, "-L");
            exr_add_arg(args, bios_dir);
        }
    }

    exr_add_arg(args, "-machine");
    if (has_gpu) {
        exr_add_arg(args, machine);
    } else {
        exr_take_arg(args, g_strdup_printf("%s,graphics=off", machine));
    }

    switch (config->engine->acceleration) {
    case EX_R_ACCELERATION_AUTO:
#if defined(CONFIG_LINUX)
        exr_add_arg(args, "-accel");
        exr_add_arg(args, "kvm");
#elif defined(CONFIG_WIN32)
        exr_add_arg(args, "-accel");
        exr_add_arg(args, "whpx");
#elif defined(CONFIG_DARWIN) && defined(HOST_X86_64)
        exr_add_arg(args, "-accel");
        exr_add_arg(args, "hvf");
#endif
        exr_add_arg(args, "-accel");
        if (tb_size) {
            exr_take_arg(args, g_strdup_printf("tcg,tb-size=%s", tb_size));
        } else {
            exr_add_arg(args, "tcg");
        }
        break;
    case EX_R_ACCELERATION_TCG:
        exr_add_arg(args, "-accel");
        if (tb_size) {
            exr_take_arg(args, g_strdup_printf("tcg,tb-size=%s", tb_size));
        } else {
            exr_add_arg(args, "tcg");
        }
        break;
    default:
        acceleration = ExRAcceleration_str(config->engine->acceleration);
        exr_add_arg(args, "-accel");
        exr_add_arg(args, acceleration);
        break;
    }

    exr_add_arg(args, "-m");
    exr_add_arg(args, config->engine->memory);
    exr_add_arg(args, "-cpu");
    exr_add_arg(args, config->engine->cpu ? config->engine->cpu : "max");
    exr_add_arg(args, "-smp");
    exr_take_arg(args, g_strdup_printf("%u", config->engine->has_smp
                                          ? config->engine->smp : 1));
    if (has_gpu) {
        const char *gpu_dev = "virtio-vga";
        if (config->engine->display == EX_R_DISPLAY_VIRTIO_GPU_GL) {
#if defined(CONFIG_VIRGL) && defined(CONFIG_OPENGL)
            gpu_dev = "virtio-vga-gl";
#else
            error_setg(&err, "display 'virtio-gpu-gl' requires virglrenderer + OpenGL support");
            qapi_free_ExRConfig(config);
            g_ptr_array_free(args, true);
            return exr_fail(err);
#endif
        } else if (config->engine->display == EX_R_DISPLAY_AUTO) {
#if defined(CONFIG_VIRGL) && defined(CONFIG_OPENGL)
            gpu_dev = "virtio-vga-gl";
#else
            gpu_dev = "virtio-vga";
#endif
        }
        exr_add_arg(args, "-display");
        exr_add_arg(args, "none");
        exr_add_arg(args, "-device");
        if (config->engine->has_vgamem_mb && config->engine->vgamem_mb > 0) {
            exr_take_arg(args, g_strdup_printf("%s,max_hostmem=%llu",
                                               gpu_dev,
                                               (unsigned long long)config->engine->vgamem_mb * 1024ULL * 1024ULL));
        } else {
            exr_add_arg(args, gpu_dev);
        }
    } else {
        exr_add_arg(args, "-nographic");
    }
    exr_add_arg(args, "-monitor");
    exr_add_arg(args, "none");

    if (config->engine->has_inputs && config->engine->inputs) {
        exr_add_arg(args, "-device");
        exr_add_arg(args, "virtio-tablet-pci");
        exr_add_arg(args, "-device");
        exr_add_arg(args, "virtio-keyboard-pci");
        exr_add_arg(args, "-device");
        exr_add_arg(args, "virtio-mouse-pci");
    }

    if (config->engine->has_balloon && config->engine->balloon) {
        exr_add_arg(args, "-device");
        exr_add_arg(args, "virtio-balloon-pci");
    }

    if (config->boot) {
        if (config->boot->kernel) {
            exr_add_arg(args, "-kernel");
            exr_take_arg(args, exr_resolve_path(config_path, config->boot->kernel));
        }
        if (config->boot->initrd) {
            exr_add_arg(args, "-initrd");
            exr_take_arg(args, exr_resolve_path(config_path, config->boot->initrd));
        }
        if (config->boot->cmdline) {
            exr_add_arg(args, "-append");
            exr_add_arg(args, config->boot->cmdline);
        }
    }

    if (config->storage || config->has_extra_storage) {
        bool low_mem = config->engine->has_profile &&
                       (config->engine->profile == EX_R_MEMORY_PROFILE_LOW_MEM_2GB ||
                        config->engine->profile == EX_R_MEMORY_PROFILE_MINIMAL);
        exr_add_arg(args, "-object");
        exr_add_arg(args, "iothread,id=exr-iothread0");

        if (config->storage) {
            exr_add_blockdev(args, config_path, config->storage, "exr-disk", low_mem);
        }
        if (config->has_extra_storage) {
            size_t idx = 1;
            for (ExRStorageConfigList *it = config->extra_storage; it; it = it->next, idx++) {
                g_autofree char *node = g_strdup_printf("exr-disk%zu", idx);
                exr_add_blockdev(args, config_path, it->value, node, low_mem);
            }
        }
    }

    if (config->shared_folder) {
        g_autofree char *host_dir = exr_resolve_path(config_path, config->shared_folder->path);
        bool ro = config->shared_folder->has_readonly && config->shared_folder->readonly;
        (void)ro;
#ifdef CONFIG_VIRTFS
        exr_add_arg(args, "-fsdev");
        exr_take_arg(args, g_strdup_printf("local,id=exr-fs0,path=%s,security_model=mapped-xattr%s",
                                           host_dir, ro ? ",readonly=on" : ""));
        exr_add_arg(args, "-device");
        exr_add_arg(args, "virtio-9p-pci,fsdev=exr-fs0,mount_tag=exr_shared");
#else
        exr_add_arg(args, "-drive");
        exr_take_arg(args, g_strdup_printf("file=fat:%s,format=raw,readonly=on,if=none,id=exr-shared",
                                           host_dir));
        exr_add_arg(args, "-device");
        exr_add_arg(args, "virtio-blk-pci,drive=exr-shared");
#endif
    }

    if (config->audio && config->audio->backend != EX_R_AUDIO_BACKEND_NONE) {
        if (config->audio->backend == EX_R_AUDIO_BACKEND_WAV && config->audio->wav_path) {
            g_autofree char *wpath = exr_resolve_path(config_path, config->audio->wav_path);
            exr_add_arg(args, "-audiodev");
            exr_take_arg(args, g_strdup_printf("wav,id=exr-audio0,path=%s", wpath));
        } else {
            exr_add_arg(args, "-audiodev");
            exr_add_arg(args, "none,id=exr-audio0");
        }
        exr_add_arg(args, "-device");
        exr_add_arg(args, "virtio-sound-pci,audiodev=exr-audio0");
    }

    if (config->networking &&
        config->networking->backend != EX_R_NETWORK_BACKEND_NONE) {
#ifndef CONFIG_SLIRP
        error_setg(&err, "networking backend '%s' requires libslirp",
                   ExRNetworkBackend_str(config->networking->backend));
        qapi_free_ExRConfig(config);
        g_ptr_array_free(args, true);
        return exr_fail(err);
#else
        exr_add_arg(args, "-nic");
        if (config->networking->has_adb_port && config->networking->adb_port > 0) {
            exr_take_arg(args, g_strdup_printf(
                "user,model=virtio-net-pci,hostfwd=tcp:127.0.0.1:%u-:5555",
                config->networking->adb_port));
        } else {
            exr_add_arg(args, "user,model=virtio-net-pci");
        }
#endif
    } else {
        exr_add_arg(args, "-nic");
        exr_add_arg(args, "none");
    }

    if (config->runtime) {
        if (config->runtime->has_serial) {
            serial = ExRSerial_str(config->runtime->serial);
        }
        if (config->runtime->socket) {
            g_autofree char *socket = exr_resolve_socket(
                config_path, config->runtime->socket);
            exr_add_arg(args, "-qmp");
            exr_take_arg(args, g_strdup_printf("%s,server=on,wait=off", socket));
        }
        if (config->runtime->pidfile) {
            g_autofree char *pidfile = exr_resolve_path(
                config_path, config->runtime->pidfile);
            exr_add_arg(args, "-pidfile");
            exr_add_arg(args, pidfile);
        }
    }
    exr_add_arg(args, "-serial");
    exr_add_arg(args, serial);

    qapi_free_ExRConfig(config);
    *argc = args->len;
    g_ptr_array_add(args, NULL);
    *argv = (char **)g_ptr_array_free(args, false);
    return -1;
}

static int exr_create(const char *program, const char *config_path)
{
    Error *err = NULL;
    ExRConfig *config = exr_read_config(config_path, &err);
    g_autofree char *path = NULL;
    uint64_t size;
    int status = 1;

    if (!config) {
        return exr_fail(err);
    }
    if (!config->storage) {
        error_setg(&err, "create requires a storage section");
        goto out;
    }
    if (!config->storage->size) {
        error_setg(&err, "create requires storage.size");
        goto out;
    }
    if (qemu_strtosz(config->storage->size, NULL, &size) < 0 || size == 0) {
        error_setg(&err, "storage.size must be a positive byte size");
        goto out;
    }
    path = exr_resolve_path(config_path, config->storage->path);
    if (g_file_test(path, G_FILE_TEST_EXISTS)) {
        error_setg(&err, "refusing to overwrite existing disk '%s'", path);
        goto out;
    }

    error_init(program);
    module_call_init(MODULE_INIT_TRACE);
    qemu_init_exec_dir(program);
    module_call_init(MODULE_INIT_TARGET_INFO);
    target_info_qom_set_target();
    if (qemu_init_main_loop(&err) < 0 || qcrypto_init(&err) < 0) {
        goto out;
    }
    module_call_init(MODULE_INIT_QOM);
    qemu_init_cpu_loop();
    bql_lock();
    bdrv_init();
    bdrv_img_create(path, exr_storage_driver(config->storage->format),
                    NULL, NULL, NULL, size, 0, true, &err);
    bql_unlock();
    if (!err) {
        printf("Created %s disk %s (%s)\n",
               ExRStorageFormat_str(config->storage->format), path,
               config->storage->size);
        status = EXIT_SUCCESS;
    }

out:
    if (err) {
        exr_fail(err);
    }
    qapi_free_ExRConfig(config);
    return status;
}

static int exr_resize(const char *program, const char *config_path,
                      const char *size_str)
{
    Error *err = NULL;
    ExRConfig *config = exr_read_config(config_path, &err);
    g_autofree char *path = NULL;
    uint64_t size;
    BlockBackend *blk = NULL;
    QDict *opts = NULL;
    int status = 1;

    if (!config) {
        return exr_fail(err);
    }
    if (!config->storage) {
        error_setg(&err, "resize requires a storage section");
        goto out;
    }
    if (qemu_strtosz(size_str, NULL, &size) < 0 || size == 0) {
        error_setg(&err, "size must be a positive byte size, not '%s'", size_str);
        goto out;
    }
    path = exr_resolve_path(config_path, config->storage->path);
    if (!g_file_test(path, G_FILE_TEST_EXISTS)) {
        error_setg(&err, "disk '%s' does not exist", path);
        goto out;
    }

    error_init(program);
    module_call_init(MODULE_INIT_TRACE);
    qemu_init_exec_dir(program);
    module_call_init(MODULE_INIT_TARGET_INFO);
    target_info_qom_set_target();
    if (qemu_init_main_loop(&err) < 0 || qcrypto_init(&err) < 0) {
        goto out;
    }
    module_call_init(MODULE_INIT_QOM);
    qemu_init_cpu_loop();
    bql_lock();
    bdrv_init();
    opts = qdict_new();
    qdict_put_str(opts, "driver", exr_storage_driver(config->storage->format));
    blk = blk_new_open(path, NULL, opts, BDRV_O_RDWR | BDRV_O_RESIZE, &err);
    if (blk) {
        if (blk_truncate(blk, size, true, PREALLOC_MODE_OFF, 0, &err) == 0) {
            printf("Resized %s disk %s to %s\n",
                   ExRStorageFormat_str(config->storage->format), path, size_str);
            status = EXIT_SUCCESS;
        }
        blk_unref(blk);
    }
    bql_unlock();

out:
    if (err) {
        exr_fail(err);
    }
    qapi_free_ExRConfig(config);
    return status;
}

static bool exr_socket_read_line(int fd, QObject **result, Error **errp)
{
    g_autoptr(GString) line = g_string_new(NULL);

    while (line->len < EXR_CTRL_MAX_BYTES) {
        char byte;
        ssize_t count = recv(fd, &byte, 1, 0);

        if (count == 1) {
            if (byte == '\n') {
                *result = qobject_from_json(line->str, errp);
                return *result != NULL;
            }
            if (byte != '\r') {
                g_string_append_c(line, byte);
            }
            continue;
        }
        if (count == 0) {
            error_setg(errp, "control peer closed the connection");
        } else {
            error_setg_errno(errp, errno, "cannot read control response");
        }
        return false;
    }
    error_setg(errp, "control response exceeds 16 MiB");
    return false;
}

static bool exr_socket_send(int fd, const char *json, Error **errp)
{
    g_autofree char *line = g_strdup_printf("%s\r\n", json);

    if (qemu_send_full(fd, line, strlen(line)) != strlen(line)) {
        error_setg_errno(errp, errno, "cannot send control command");
        return false;
    }
    return true;
}

static bool exr_qmp_wait(int fd, const char *id, QObject **response,
                         Error **errp)
{
    for (;;) {
        QObject *object;

        if (!exr_socket_read_line(fd, &object, errp)) {
            return false;
        }
        QDict *dict = qobject_to(QDict, object);
        if (!dict) {
            qobject_unref(object);
            error_setg(errp, "control response is not an object");
            return false;
        }
        const char *resp_id = qdict_get_try_str(dict, "id");
        if ((!id || !*id) && !resp_id) {
            *response = object;
            return true;
        }
        if (resp_id && id && !g_strcmp0(resp_id, id)) {
            *response = object;
            return true;
        }
        qobject_unref(object); /* asynchronous event or unrelated response */
    }
}

static bool exr_qmp_execute(const char *endpoint, QDict *command,
                            QObject **response, Error **errp)
{
    g_autofree char *uri = g_strdup(endpoint);
    SocketAddress *address = socket_parse(uri, errp);
    QObject *reply = NULL;
    GString *json;
    int fd;
    bool ok = false;

    if (!address) {
        return false;
    }
    socket_init();
    fd = socket_connect(address, errp);
    qapi_free_SocketAddress(address);
    if (fd < 0) {
        return false;
    }
    if (!exr_qmp_wait(fd, "", &reply, errp)) { /* greeting has no id */
        goto out;
    }
    qobject_unref(reply);
    reply = NULL;
    if (!exr_socket_send(fd, "{\"execute\":\"qmp_capabilities\",\"id\":\"exr-cap\"}", errp) ||
        !exr_qmp_wait(fd, "exr-cap", &reply, errp)) {
        goto out;
    }
    if (qdict_get(qobject_to(QDict, reply), "error")) {
        error_setg(errp, "control capability negotiation failed");
        goto out;
    }
    qobject_unref(reply);
    reply = NULL;

    qdict_put_str(command, "id", "exr-command");
    json = qobject_to_json(QOBJECT(command));
    ok = exr_socket_send(fd, json->str, errp);
    g_string_free(json, true);
    if (ok) {
        ok = exr_qmp_wait(fd, "exr-command", response, errp);
    }

out:
    qobject_unref(reply);
    qemu_close(fd);
    return ok;
}

static int exr_control(const char *config_path, QDict *command,
                       bool print_response)
{
    Error *err = NULL;
    ExRConfig *config = exr_read_config(config_path, &err);
    g_autofree char *endpoint = NULL;
    QObject *response = NULL;
    int status = 1;

    if (!config) {
        qobject_unref(command);
        return exr_fail(err);
    }
    if (!config->runtime || !config->runtime->socket) {
        error_setg(&err, "control commands require runtime.socket");
        goto out;
    }
    endpoint = exr_resolve_socket(config_path, config->runtime->socket);
    if (!exr_qmp_execute(endpoint, command, &response, &err)) {
        goto out;
    }
    QDict *dict = qobject_to(QDict, response);
    QObject *error = qdict_get(dict, "error");
    if (error) {
        g_autoptr(GString) text = qobject_to_json_pretty(error, true);
        error_setg(&err, "control command failed: %s", text->str);
        goto out;
    }
    if (print_response) {
        QObject *value = qdict_get(dict, "return");
        g_autoptr(GString) text = qobject_to_json_pretty(value, true);
        puts(text->str);
    }
    status = EXIT_SUCCESS;

out:
    if (err) {
        exr_fail(err);
    }
    qobject_unref(response);
    qobject_unref(command);
    qapi_free_ExRConfig(config);
    return status;
}

static int exr_devices(const char *program, int *argc, char ***argv)
{
    char **args = g_new0(char *, 5);

    args[0] = g_strdup(program);
    args[1] = g_strdup("-display");
    args[2] = g_strdup("none");
    args[3] = g_strdup("-device");
    args[4] = g_strdup("help");
    *argc = 5;
    *argv = args;
    return -1;
}

int exr_cli_prepare(int *argc, char ***argv)
{
    const char *program = (*argv)[0];
    const char *command = *argc > 1 ? (*argv)[1] : "help";
    QDict *request;
    int status;

    if (!strcmp(command, "help") || !strcmp(command, "--help") ||
        !strcmp(command, "-h")) {
        exr_print_help();
        return *argc == 1 ? EXIT_FAILURE : EXIT_SUCCESS;
    }
    if (!strcmp(command, "version") || !strcmp(command, "--version") ||
        !strcmp(command, "-version")) {
        exr_print_version();
        return EXIT_SUCCESS;
    }
    if (!strcmp(command, "devices") && *argc == 2) {
        return exr_devices(program, argc, argv);
    }
    if (!strcmp(command, "run") && *argc == 3) {
        return exr_prepare_run(program, (*argv)[2], argc, argv);
    }
    if (!strcmp(command, "create") && *argc == 3) {
        return exr_create(program, (*argv)[2]);
    }
    if (!strcmp(command, "resize") && *argc == 4) {
        return exr_resize(program, (*argv)[2], (*argv)[3]);
    }
    if (!strcmp(command, "stop") && *argc == 3) {
        request = qdict_new();
        qdict_put_str(request, "execute", "system_powerdown");
        status = exr_control((*argv)[2], request, false);
        if (status == EXIT_SUCCESS) {
            puts("Guest shutdown requested");
        }
        return status;
    }
    if (!strcmp(command, "inspect") && *argc == 3) {
        request = qdict_new();
        qdict_put_str(request, "execute", "query-status");
        return exr_control((*argv)[2], request, true);
    }
    if (!strcmp(command, "snapshot") && *argc == 4) {
        const char *name = (*argv)[3];
        if (!*name || strlen(name) > 128 || strspn(name, "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_.-") != strlen(name)) {
            error_report("snapshot name must be 1-128 characters from [A-Za-z0-9_.-]");
            return EXIT_FAILURE;
        }
        request = qdict_new();
        qdict_put_str(request, "execute", "human-monitor-command");
        QDict *arguments = qdict_new();
        g_autofree char *line = g_strdup_printf("savevm %s", name);
        qdict_put_str(arguments, "command-line", line);
        qdict_put_obj(request, "arguments", QOBJECT(arguments));
        return exr_control((*argv)[2], request, false);
    }

    error_report("invalid command or arguments; run 'exr-x86 help'");
    return EXIT_FAILURE;
}
