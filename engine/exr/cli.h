/* ExR product entry points; execution internals remain upstream QEMU. */
#ifndef EXR_CLI_H
#define EXR_CLI_H

void exr_print_version(void);

/* Return -1 to enter the engine with transformed argv, otherwise exit status. */
int exr_cli_prepare(int *argc, char ***argv);

#endif
