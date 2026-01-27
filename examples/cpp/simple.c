/**
 * @file simple.c
 * @brief Minimal C example of using feedtui
 *
 * Build:
 *   cargo build --release --features ffi
 *   gcc -o simple simple.c -I../../include -L../../target/release -lfeedtui -lpthread -ldl -lm
 *
 * Run:
 *   LD_LIBRARY_PATH=../../target/release ./simple
 */

#include <stdio.h>
#include "feedtui.h"

int main(void) {
    printf("feedtui version: %s\n", feedtui_version());

    /* Initialize with default configuration */
    FeedtuiHandle* handle = feedtui_init(NULL);
    if (!handle) {
        fprintf(stderr, "Failed to initialize feedtui\n");
        return 1;
    }

    /* Run the TUI (blocks until user quits) */
    int result = feedtui_run(handle);

    /* Check for errors */
    if (result != FEEDTUI_SUCCESS) {
        const char* err = feedtui_get_last_error(handle);
        if (err) {
            fprintf(stderr, "Error: %s\n", err);
        }
    }

    /* Clean up */
    feedtui_shutdown(handle);

    return result;
}
