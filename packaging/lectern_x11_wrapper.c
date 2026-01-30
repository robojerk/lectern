/* Force X11: unset WAYLAND_DISPLAY, set DISPLAY if unset, then exec lectern.bin */
#define _GNU_SOURCE
#include <unistd.h>
#include <stdlib.h>
#include <stdio.h>

int main(int argc, char *argv[]) {
    unsetenv("WAYLAND_DISPLAY");
    if (!getenv("DISPLAY"))
        setenv("DISPLAY", ":0", 1);
    char path[4096];
    snprintf(path, sizeof(path), "%s.bin", argv[0]);
    execv(path, argv);
    perror("execv");
    return 127;
}
