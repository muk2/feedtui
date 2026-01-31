/**
 * @file main.cpp
 * @brief Example of using feedtui from C++
 *
 * This example demonstrates how to initialize, run, and shutdown
 * the feedtui terminal dashboard from a C++ application.
 *
 * Build instructions:
 *   # First build the Rust library with FFI support
 *   cargo build --release --features ffi
 *
 *   # Then compile this C++ example
 *   g++ -o feedtui_example main.cpp \
 *       -I../../include \
 *       -L../../target/release \
 *       -lfeedtui \
 *       -lpthread -ldl -lm
 *
 *   # Run (may need to set library path)
 *   LD_LIBRARY_PATH=../../target/release ./feedtui_example
 */

#include <cstdio>
#include <cstdlib>
#include <cstring>

#include "feedtui.h"

// Example TOML configuration with a simple HN widget
const char* DEFAULT_CONFIG = R"(
[general]
refresh_interval_secs = 60
theme = "dark"

[[widgets]]
type = "hackernews"
title = "Hacker News"
story_count = 15
story_type = "top"
position = { row = 0, col = 0 }
)";

void print_usage(const char* program_name) {
    printf("Usage: %s [options]\n", program_name);
    printf("\n");
    printf("Options:\n");
    printf("  -c, --config <path>   Path to TOML config file\n");
    printf("  -v, --version         Print version and exit\n");
    printf("  -h, --help            Print this help message\n");
    printf("\n");
    printf("If no config file is specified, a default configuration with\n");
    printf("Hacker News widget will be used.\n");
}

int main(int argc, char* argv[]) {
    const char* config_path = nullptr;
    bool use_embedded_config = true;

    // Parse command line arguments
    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-h") == 0 || strcmp(argv[i], "--help") == 0) {
            print_usage(argv[0]);
            return 0;
        }
        else if (strcmp(argv[i], "-v") == 0 || strcmp(argv[i], "--version") == 0) {
            printf("feedtui version: %s\n", feedtui_version());
            printf("FFI support: %s\n", feedtui_has_feature("ffi") == 1 ? "yes" : "no");
            return 0;
        }
        else if ((strcmp(argv[i], "-c") == 0 || strcmp(argv[i], "--config") == 0) && i + 1 < argc) {
            config_path = argv[++i];
            use_embedded_config = false;
        }
        else {
            fprintf(stderr, "Unknown option: %s\n", argv[i]);
            print_usage(argv[0]);
            return 1;
        }
    }

    // Print version info
    printf("feedtui C++ Example\n");
    printf("Library version: %s\n", feedtui_version());
    printf("\n");

    // Initialize feedtui
    FeedtuiHandle* handle = nullptr;

    if (use_embedded_config) {
        printf("Using embedded default configuration...\n");
        handle = feedtui_init_with_config(DEFAULT_CONFIG);
    } else {
        printf("Loading config from: %s\n", config_path);
        handle = feedtui_init(config_path);
    }

    if (!handle) {
        fprintf(stderr, "Error: Failed to initialize feedtui\n");
        return 1;
    }

    printf("Starting feedtui... (press 'q' to quit)\n");
    printf("\n");

    // Run the TUI (this blocks until user quits)
    int result = feedtui_run(handle);

    // Check for errors
    if (result != FEEDTUI_SUCCESS) {
        const char* error = feedtui_get_last_error(handle);
        if (error) {
            fprintf(stderr, "Error: %s\n", error);
        } else {
            fprintf(stderr, "Error: Unknown error (code %d)\n", result);
        }
    }

    // Clean up
    feedtui_shutdown(handle);

    printf("\nfeedtui terminated with code: %d\n", result);
    return result;
}
