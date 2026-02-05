# feedtui FFI (Foreign Function Interface)

This document describes how to use feedtui from C/C++ applications through its FFI interface.

## Overview

feedtui provides a C-compatible FFI layer that allows C/C++ applications to:
- Initialize the feedtui TUI application
- Run the dashboard (blocking or controlled)
- Cleanly shutdown and free resources

The interface follows a lifecycle pattern with opaque handles, making it safe and easy to integrate.

## Building with FFI Support

### Prerequisites

- Rust toolchain (1.70+)
- C/C++ compiler (GCC, Clang, or MSVC)
- OpenSSL development libraries

### Build the Library

```bash
# Build the shared and static libraries with FFI support
cargo build --release --features ffi

# The libraries will be in target/release/
# - libfeedtui.so (Linux shared)
# - libfeedtui.dylib (macOS shared)
# - libfeedtui.dll (Windows shared)
# - libfeedtui.a (static)
```

### Library Artifacts

After building, you'll find:

| Platform | Shared Library | Static Library |
|----------|---------------|----------------|
| Linux    | `libfeedtui.so` | `libfeedtui.a` |
| macOS    | `libfeedtui.dylib` | `libfeedtui.a` |
| Windows  | `feedtui.dll` | `feedtui.lib` |

## API Reference

### Header File

Include the header in your C/C++ code:

```c
#include "feedtui.h"
```

The header is located at `include/feedtui.h`.

### Types

#### `FeedtuiHandle`

An opaque pointer to the feedtui instance. Do not attempt to access its internal structure.

```c
typedef struct FeedtuiHandle FeedtuiHandle;
```

#### `FeedtuiResult`

Result codes returned by FFI functions:

```c
typedef enum FeedtuiResult {
    FEEDTUI_SUCCESS = 0,           // Operation completed successfully
    FEEDTUI_INVALID_HANDLE = 1,    // Invalid or null handle
    FEEDTUI_INVALID_CONFIG_PATH = 2,
    FEEDTUI_CONFIG_LOAD_ERROR = 3,
    FEEDTUI_RUNTIME_ERROR = 4,
    FEEDTUI_APP_ERROR = 5,
    FEEDTUI_PANIC = 6              // Rust panic (check last_error)
} FeedtuiResult;
```

### Functions

#### `feedtui_init`

Initialize a new feedtui instance with a config file.

```c
FeedtuiHandle* feedtui_init(const char* config_path);
```

**Parameters:**
- `config_path`: Path to TOML config file (UTF-8), or `NULL` for default config

**Returns:** Handle pointer on success, `NULL` on failure

**Example:**
```c
// Use default configuration
FeedtuiHandle* handle = feedtui_init(NULL);

// Use custom config file
FeedtuiHandle* handle = feedtui_init("/home/user/.feedtui/config.toml");
```

#### `feedtui_init_with_config`

Initialize with an inline TOML configuration string.

```c
FeedtuiHandle* feedtui_init_with_config(const char* config_toml);
```

**Parameters:**
- `config_toml`: TOML configuration as a null-terminated string

**Example:**
```c
const char* config =
    "[general]\n"
    "refresh_interval_secs = 60\n"
    "\n"
    "[[widgets]]\n"
    "type = \"hackernews\"\n"
    "title = \"HN\"\n"
    "story_count = 10\n"
    "position = { row = 0, col = 0 }\n";

FeedtuiHandle* handle = feedtui_init_with_config(config);
```

#### `feedtui_run`

Run the feedtui TUI application. This function blocks until the user quits.

```c
int feedtui_run(FeedtuiHandle* handle);
```

**Parameters:**
- `handle`: Valid handle from `feedtui_init` or `feedtui_init_with_config`

**Returns:** `FEEDTUI_SUCCESS` (0) on success, error code otherwise

**Note:** The terminal will be taken over for the TUI. Press 'q' to quit.

#### `feedtui_shutdown`

Free all resources and invalidate the handle.

```c
void feedtui_shutdown(FeedtuiHandle* handle);
```

**Parameters:**
- `handle`: Handle to shutdown, or `NULL` (no-op)

**Note:** After calling this, the handle must not be used again.

#### `feedtui_get_last_error`

Get the last error message.

```c
const char* feedtui_get_last_error(const FeedtuiHandle* handle);
```

**Returns:** Error string or `NULL` if no error. Do not free this pointer.

#### `feedtui_version`

Get the library version string.

```c
const char* feedtui_version(void);
```

**Returns:** Version string (e.g., "0.1.1"). Do not free this pointer.

#### `feedtui_has_feature`

Check if a feature was enabled at compile time.

```c
int feedtui_has_feature(const char* feature);
```

**Returns:** 1 if enabled, 0 if not, -1 if invalid feature name

## Complete Examples

### Minimal C Example

```c
#include <stdio.h>
#include "feedtui.h"

int main(void) {
    FeedtuiHandle* handle = feedtui_init(NULL);
    if (!handle) {
        fprintf(stderr, "Failed to initialize\n");
        return 1;
    }

    int result = feedtui_run(handle);

    if (result != FEEDTUI_SUCCESS) {
        const char* err = feedtui_get_last_error(handle);
        if (err) fprintf(stderr, "Error: %s\n", err);
    }

    feedtui_shutdown(handle);
    return result;
}
```

### C++ Example with Custom Config

```cpp
#include <iostream>
#include <string>
#include "feedtui.h"

int main() {
    // Build a custom configuration
    std::string config = R"(
[general]
refresh_interval_secs = 30
theme = "dark"

[[widgets]]
type = "hackernews"
title = "Hacker News"
story_count = 20
story_type = "top"
position = { row = 0, col = 0 }

[[widgets]]
type = "rss"
title = "Tech News"
feeds = [
    "https://feeds.arstechnica.com/arstechnica/technology-lab"
]
max_items = 15
position = { row = 0, col = 1 }
)";

    std::cout << "feedtui version: " << feedtui_version() << std::endl;

    FeedtuiHandle* handle = feedtui_init_with_config(config.c_str());
    if (!handle) {
        std::cerr << "Failed to initialize feedtui" << std::endl;
        return 1;
    }

    int result = feedtui_run(handle);

    if (result != FEEDTUI_SUCCESS) {
        const char* err = feedtui_get_last_error(handle);
        if (err) {
            std::cerr << "Error: " << err << std::endl;
        }
    }

    feedtui_shutdown(handle);
    return result;
}
```

## Build Instructions

### Linux (GCC)

```bash
# Build the Rust library
cargo build --release --features ffi

# Compile C code
gcc -o myapp myapp.c \
    -I/path/to/feedtui/include \
    -L/path/to/feedtui/target/release \
    -lfeedtui \
    -lpthread -ldl -lm

# Run (set library path)
LD_LIBRARY_PATH=/path/to/feedtui/target/release ./myapp
```

### macOS (Clang)

```bash
# Build the Rust library
cargo build --release --features ffi

# Compile C code
clang -o myapp myapp.c \
    -I/path/to/feedtui/include \
    -L/path/to/feedtui/target/release \
    -lfeedtui \
    -framework Security -framework CoreFoundation

# Run
DYLD_LIBRARY_PATH=/path/to/feedtui/target/release ./myapp
```

### CMake

```cmake
cmake_minimum_required(VERSION 3.15)
project(MyFeedtuiApp)

# Find the feedtui library
set(FEEDTUI_DIR "/path/to/feedtui")
set(FEEDTUI_INCLUDE "${FEEDTUI_DIR}/include")
set(FEEDTUI_LIB "${FEEDTUI_DIR}/target/release")

add_executable(myapp main.cpp)

target_include_directories(myapp PRIVATE ${FEEDTUI_INCLUDE})
target_link_directories(myapp PRIVATE ${FEEDTUI_LIB})
target_link_libraries(myapp feedtui pthread dl m)
```

## Thread Safety

- All FFI functions must be called from the same thread
- Do not call FFI functions concurrently
- The TUI takes exclusive control of the terminal

## Memory Management

- Handles returned by `feedtui_init*` must be freed with `feedtui_shutdown`
- String pointers returned by `feedtui_get_last_error` and `feedtui_version` are owned by the library - do not free them
- Error strings are valid until the next FFI call or shutdown

## Error Handling

1. Check if functions return `NULL` (init functions) or non-zero (run)
2. Use `feedtui_get_last_error` to get detailed error messages
3. Always call `feedtui_shutdown` even after errors

## Configuration Reference

The TOML configuration format is the same as the CLI version. See the main README for full configuration options.

### Minimal Configuration

```toml
[general]
refresh_interval_secs = 60

[[widgets]]
type = "hackernews"
title = "HN"
story_count = 10
story_type = "top"
position = { row = 0, col = 0 }
```

### Available Widget Types

- `hackernews` - Hacker News stories
- `rss` - RSS feed aggregator
- `stocks` - Stock quotes
- `sports` - Sports scores
- `github` - GitHub dashboard
- `youtube` - YouTube videos
- `creature` - Virtual pet companion

## Troubleshooting

### Library not found at runtime

Set the library path:
```bash
# Linux
export LD_LIBRARY_PATH=/path/to/target/release:$LD_LIBRARY_PATH

# macOS
export DYLD_LIBRARY_PATH=/path/to/target/release:$DYLD_LIBRARY_PATH
```

### Linking errors

Ensure you're linking all required system libraries:
- Linux: `-lpthread -ldl -lm`
- macOS: `-framework Security -framework CoreFoundation`

### Terminal not restored after crash

If feedtui crashes without proper cleanup, run:
```bash
reset
```

## License

MIT License - See LICENSE file for details.
