/*
 * Standalone ExR ISO -> .exrimg + config.exr converter.
 * Separate from the headless execution engine.
 *
 * Usage: exr-convert <input.iso> <output_dir_or_exrimg> [size]
 */

#define _GNU_SOURCE
#include <errno.h>
#include <libgen.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/wait.h>
#include <unistd.h>

static int run_cmd(const char *const argv[])
{
    pid_t pid = fork();
    if (pid < 0) {
        perror("fork");
        return -1;
    }
    if (pid == 0) {
        execvp(argv[0], (char *const *)argv);
        perror(argv[0]);
        _exit(127);
    }
    int status = 0;
    if (waitpid(pid, &status, 0) < 0) {
        perror("waitpid");
        return -1;
    }
    return WIFEXITED(status) && WEXITSTATUS(status) == 0 ? 0 : -1;
}

static int mkdir_p(const char *path)
{
    char tmp[1024];
    snprintf(tmp, sizeof(tmp), "%s", path);
    size_t len = strlen(tmp);
    if (len == 0) return -1;
    if (tmp[len - 1] == '/') tmp[len - 1] = '\0';
    for (char *p = tmp + 1; *p; p++) {
        if (*p == '/') {
            *p = '\0';
            if (mkdir(tmp, 0755) < 0 && errno != EEXIST) return -1;
            *p = '/';
        }
    }
    if (mkdir(tmp, 0755) < 0 && errno != EEXIST) return -1;
    return 0;
}

static const char *find_exr_img(const char *self)
{
    static char buf[1024];
    char self_copy[1024];
    snprintf(self_copy, sizeof(self_copy), "%s", self);
    const char *dir = dirname(self_copy);

    const char *rels[] = {
        "%s/../engine/bin/exr-img",
        "%s/exr-img",
        "dist/engine/bin/exr-img",
        "engine/build/exr-img",
    };
    for (size_t i = 0; i < sizeof(rels) / sizeof(rels[0]); i++) {
        if (i < 2) snprintf(buf, sizeof(buf), rels[i], dir);
        else snprintf(buf, sizeof(buf), "%s", rels[i]);
        if (access(buf, X_OK) == 0) return buf;
    }
    return "exr-img";
}

int main(int argc, char **argv)
{
    if (argc < 3 || argc > 4) {
        fprintf(stderr, "Usage: %s <input.iso> <output_dir_or_exrimg> [size]\n", argv[0]);
        return 1;
    }
    const char *iso = argv[1];
    const char *out_arg = argv[2];
    const char *size_str = argc == 4 ? argv[3] : NULL;

    struct stat st;
    if (stat(iso, &st) < 0 || !S_ISREG(st.st_mode) || st.st_size <= 0) {
        fprintf(stderr, "error: cannot read ISO '%s'\n", iso);
        return 1;
    }

    char vm_dir[1024], disk_path[1024], cfg_path[1024], disk_base[256];
    size_t out_len = strlen(out_arg);
    if (out_len > 7 && strcmp(out_arg + out_len - 7, ".exrimg") == 0) {
        snprintf(disk_path, sizeof(disk_path), "%s", out_arg);
        char copy1[1024], copy2[1024];
        snprintf(copy1, sizeof(copy1), "%s", out_arg);
        snprintf(copy2, sizeof(copy2), "%s", out_arg);
        snprintf(vm_dir, sizeof(vm_dir), "%s", dirname(copy1));
        snprintf(disk_base, sizeof(disk_base), "%s", basename(copy2));
    } else {
        snprintf(vm_dir, sizeof(vm_dir), "%s", out_arg);
        snprintf(disk_base, sizeof(disk_base), "system.exrimg");
        snprintf(disk_path, sizeof(disk_path), "%s/%s", vm_dir, disk_base);
    }
    snprintf(cfg_path, sizeof(cfg_path), "%s/config.exr", vm_dir);

    if (mkdir_p(vm_dir) < 0) {
        fprintf(stderr, "error: cannot create directory '%s': %s\n", vm_dir, strerror(errno));
        return 1;
    }
    if (access(disk_path, F_OK) == 0) {
        fprintf(stderr, "error: refusing to overwrite '%s'\n", disk_path);
        return 1;
    }

    const char *exr_img = find_exr_img(argv[0]);
    const char *conv_argv[] = { exr_img, "convert", "-f", "raw", "-O", "qcow2", iso, disk_path, NULL };
    if (run_cmd(conv_argv) != 0) {
        fprintf(stderr, "error: ISO conversion failed\n");
        return 1;
    }

    if (size_str) {
        const char *res_argv[] = { exr_img, "resize", "-q", disk_path, size_str, NULL };
        if (run_cmd(res_argv) != 0) {
            fprintf(stderr, "error: resize to '%s' failed\n", size_str);
            return 1;
        }
    }

    if (access(cfg_path, F_OK) != 0) {
        FILE *fp = fopen(cfg_path, "w");
        if (!fp) {
            fprintf(stderr, "error: cannot write '%s': %s\n", cfg_path, strerror(errno));
            return 1;
        }
        fprintf(fp,
                "{\n"
                "  \"schema-version\": 1,\n"
                "  \"engine\": {\n"
                "    \"architecture\": \"x86_64\",\n"
                "    \"acceleration\": \"auto\",\n"
                "    \"memory\": \"768M\",\n"
                "    \"cpu\": \"max\",\n"
                "    \"machine\": \"pc\",\n"
                "    \"smp\": 2,\n"
                "    \"profile\": \"low-mem-2gb\",\n"
                "    \"balloon\": true,\n"
                "    \"inputs\": true,\n"
                "    \"display\": \"auto\",\n"
                "    \"vgamem-mb\": 32\n"
                "  },\n"
                "  \"storage\": {\n"
                "    \"path\": \"%s\",\n"
                "    \"format\": \"exrimg\",\n"
                "    \"interface\": \"virtio\",\n"
                "    \"size\": \"%s\"\n"
                "  },\n"
                "  \"audio\": { \"backend\": \"auto\" },\n"
                "  \"networking\": { \"backend\": \"auto\", \"adb-port\": 5555 },\n"
                "  \"runtime\": { \"socket\": \"unix:exr-x86.sock\", \"pidfile\": \"exr-x86.pid\", \"serial\": \"stdio\" }\n"
                "}\n",
                disk_base, size_str ? size_str : "16G");
        fclose(fp);
    }

    printf("Converted %s -> %s + %s\n", iso, disk_path, cfg_path);
    return 0;
}
