/**
 * @file feedtui.h
 * @brief C/C++ interface for feedtui terminal dashboard
 *
 * This header provides a C-compatible interface for integrating feedtui
 * into C/C++ applications. The interface follows a lifecycle pattern:
 *
 * 1. Initialize with feedtui_init() or feedtui_init_with_config()
 * 2. Run the TUI with feedtui_run()
 * 3. Clean up with feedtui_shutdown()
 *
 * @example
 * ```c
 * #include "feedtui.h"
 *
 * int main(void) {
 *     FeedtuiHandle* handle = feedtui_init(NULL);
 *     if (!handle) {
 *         fprintf(stderr, "Failed to initialize feedtui\n");
 *         return 1;
 *     }
 *
 *     int result = feedtui_run(handle);
 *     feedtui_shutdown(handle);
 *
 *     return result;
 * }
 * ```
 *
 * @copyright MIT License
 */

#ifndef FEEDTUI_H
#define FEEDTUI_H

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief Opaque handle to a feedtui instance.
 *
 * Users should not attempt to access the internal structure.
 * Use the provided functions to interact with the handle.
 */
typedef struct FeedtuiHandle FeedtuiHandle;

/**
 * @brief Result codes returned by feedtui functions.
 */
typedef enum FeedtuiResult {
    /** Operation completed successfully */
    FEEDTUI_SUCCESS = 0,
    /** Invalid or null handle provided */
    FEEDTUI_INVALID_HANDLE = 1,
    /** Invalid or null config path */
    FEEDTUI_INVALID_CONFIG_PATH = 2,
    /** Failed to load configuration */
    FEEDTUI_CONFIG_LOAD_ERROR = 3,
    /** Failed to initialize runtime */
    FEEDTUI_RUNTIME_ERROR = 4,
    /** Application error during execution */
    FEEDTUI_APP_ERROR = 5,
    /** Panic occurred (check feedtui_get_last_error for details) */
    FEEDTUI_PANIC = 6
} FeedtuiResult;

/**
 * @brief Initialize a new feedtui instance.
 *
 * @param config_path Path to the TOML configuration file (UTF-8 encoded, null-terminated).
 *                    If NULL, uses the default configuration.
 *
 * @return A pointer to a FeedtuiHandle on success, or NULL on failure.
 *         The caller is responsible for calling feedtui_shutdown() to free the handle.
 *
 * @note The handle must not be used after calling feedtui_shutdown().
 *
 * @example
 * ```c
 * // Use default config
 * FeedtuiHandle* handle = feedtui_init(NULL);
 *
 * // Use custom config file
 * FeedtuiHandle* handle = feedtui_init("/home/user/.feedtui/config.toml");
 * ```
 */
FeedtuiHandle* feedtui_init(const char* config_path);

/**
 * @brief Initialize feedtui with a configuration string.
 *
 * This function allows passing the TOML configuration directly as a string,
 * which is useful for embedded configurations or testing.
 *
 * @param config_toml TOML configuration content as a UTF-8 null-terminated string.
 *                    Must not be NULL.
 *
 * @return A pointer to a FeedtuiHandle on success, or NULL on failure.
 *
 * @example
 * ```c
 * const char* config =
 *     "[general]\n"
 *     "refresh_interval_secs = 60\n"
 *     "\n"
 *     "[[widgets]]\n"
 *     "type = \"hackernews\"\n"
 *     "title = \"HN\"\n"
 *     "story_count = 10\n"
 *     "story_type = \"top\"\n"
 *     "position = { row = 0, col = 0 }\n";
 *
 * FeedtuiHandle* handle = feedtui_init_with_config(config);
 * ```
 */
FeedtuiHandle* feedtui_init_with_config(const char* config_toml);

/**
 * @brief Run the feedtui application.
 *
 * This function blocks until the user quits the application (e.g., by pressing 'q').
 * The terminal will be taken over for the TUI display.
 *
 * @param handle A valid handle obtained from feedtui_init() or feedtui_init_with_config().
 *
 * @return FEEDTUI_SUCCESS (0) on successful completion, or an error code on failure.
 *
 * @note This function must not be called concurrently from multiple threads.
 * @note On error, call feedtui_get_last_error() to get the error message.
 */
int feedtui_run(FeedtuiHandle* handle);

/**
 * @brief Shutdown and free the feedtui instance.
 *
 * This function frees all resources associated with the handle.
 * After calling this function, the handle is invalid and must not be used.
 *
 * @param handle A valid handle obtained from feedtui_init() or feedtui_init_with_config().
 *               If NULL, this function does nothing.
 *
 * @note It is safe to call this function with a NULL handle.
 * @note After calling this function, the handle must not be used again.
 */
void feedtui_shutdown(FeedtuiHandle* handle);

/**
 * @brief Get the last error message.
 *
 * @param handle A valid handle obtained from feedtui_init() or feedtui_init_with_config().
 *
 * @return A pointer to a null-terminated UTF-8 string containing the last error message,
 *         or NULL if no error has occurred or if the handle is invalid.
 *
 * @note The returned string is owned by the handle and remains valid until:
 *       - The next FFI function call on this handle
 *       - feedtui_shutdown() is called
 *
 * @note Do not free the returned pointer.
 */
const char* feedtui_get_last_error(const FeedtuiHandle* handle);

/**
 * @brief Get the version string of feedtui.
 *
 * @return A pointer to a null-terminated UTF-8 string containing the version.
 *         The returned string is statically allocated and valid for the program's lifetime.
 *
 * @note Do not free the returned pointer.
 */
const char* feedtui_version(void);

/**
 * @brief Check if feedtui was compiled with a specific feature.
 *
 * @param feature The feature name to check (null-terminated UTF-8 string).
 *
 * @return 1 if the feature is enabled, 0 if not, -1 if the feature name is invalid.
 *
 * @example
 * ```c
 * if (feedtui_has_feature("ffi") == 1) {
 *     printf("FFI support is enabled\n");
 * }
 * ```
 */
int feedtui_has_feature(const char* feature);

#ifdef __cplusplus
} /* extern "C" */
#endif

#endif /* FEEDTUI_H */
