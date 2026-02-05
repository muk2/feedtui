//! FFI (Foreign Function Interface) module for feedtui
//!
//! This module provides a C-compatible interface for integrating feedtui
//! into C/C++ applications. It exposes lifecycle-style APIs for initializing,
//! running, and shutting down the feedtui TUI application.
//!
//! # Safety
//!
//! All functions in this module that cross the FFI boundary are marked as `unsafe`
//! and must be called with valid parameters. The caller is responsible for ensuring
//! that pointers are valid and that the functions are called in the correct order.
//!
//! # Thread Safety
//!
//! The feedtui application is single-threaded. All FFI functions must be called
//! from the same thread. Calling from multiple threads simultaneously is undefined
//! behavior.
//!
//! # Example (C++)
//!
//! ```cpp
//! #include "feedtui.h"
//!
//! int main() {
//!     // Initialize with default config
//!     FeedtuiHandle* handle = feedtui_init(nullptr);
//!     if (!handle) {
//!         fprintf(stderr, "Failed to initialize feedtui\n");
//!         return 1;
//!     }
//!
//!     // Run the TUI (blocks until user quits)
//!     int result = feedtui_run(handle);
//!
//!     // Clean up
//!     feedtui_shutdown(handle);
//!
//!     return result;
//! }
//! ```

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::panic::{self, AssertUnwindSafe};
use std::path::PathBuf;
use std::ptr;

use crate::app::App;
use crate::config::Config;

/// Opaque handle to the feedtui application instance.
///
/// This struct is opaque to C code - users should only interact with it
/// through the provided functions.
pub struct FeedtuiHandle {
    app: Option<App>,
    config: Config,
    runtime: Option<tokio::runtime::Runtime>,
    last_error: Option<CString>,
}

/// Result codes returned by FFI functions
#[repr(C)]
pub enum FeedtuiResult {
    /// Operation completed successfully
    Success = 0,
    /// Invalid or null handle provided
    InvalidHandle = 1,
    /// Invalid or null config path
    InvalidConfigPath = 2,
    /// Failed to load configuration
    ConfigLoadError = 3,
    /// Failed to initialize runtime
    RuntimeError = 4,
    /// Application error during execution
    AppError = 5,
    /// Panic occurred (check last_error for details)
    Panic = 6,
}

/// Initialize a new feedtui instance.
///
/// # Arguments
///
/// * `config_path` - Path to the TOML configuration file (UTF-8 encoded, null-terminated).
///                   If NULL, uses the default configuration.
///
/// # Returns
///
/// A pointer to a `FeedtuiHandle` on success, or NULL on failure.
/// The caller is responsible for calling `feedtui_shutdown` to free the handle.
///
/// # Safety
///
/// * `config_path` must be NULL or a valid null-terminated UTF-8 string.
/// * The returned handle must not be used after calling `feedtui_shutdown`.
#[no_mangle]
pub unsafe extern "C" fn feedtui_init(config_path: *const c_char) -> *mut FeedtuiHandle {
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        // Load config
        let config = if config_path.is_null() {
            Config::default()
        } else {
            let path_str = match CStr::from_ptr(config_path).to_str() {
                Ok(s) => s,
                Err(_) => return ptr::null_mut(),
            };
            let path = PathBuf::from(path_str);
            match Config::load(&path) {
                Ok(c) => c,
                Err(_) => return ptr::null_mut(),
            }
        };

        // Create tokio runtime
        let runtime = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(_) => return ptr::null_mut(),
        };

        // Create handle
        let handle = Box::new(FeedtuiHandle {
            app: None,
            config,
            runtime: Some(runtime),
            last_error: None,
        });

        Box::into_raw(handle)
    }));

    match result {
        Ok(handle) => handle,
        Err(_) => ptr::null_mut(),
    }
}

/// Initialize feedtui with a configuration string.
///
/// # Arguments
///
/// * `config_toml` - TOML configuration content as a UTF-8 null-terminated string.
///
/// # Returns
///
/// A pointer to a `FeedtuiHandle` on success, or NULL on failure.
///
/// # Safety
///
/// * `config_toml` must be a valid null-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn feedtui_init_with_config(config_toml: *const c_char) -> *mut FeedtuiHandle {
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        if config_toml.is_null() {
            return ptr::null_mut();
        }

        let config_str = match CStr::from_ptr(config_toml).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        };

        let config: Config = match toml::from_str(config_str) {
            Ok(c) => c,
            Err(_) => return ptr::null_mut(),
        };

        let runtime = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(_) => return ptr::null_mut(),
        };

        let handle = Box::new(FeedtuiHandle {
            app: None,
            config,
            runtime: Some(runtime),
            last_error: None,
        });

        Box::into_raw(handle)
    }));

    match result {
        Ok(handle) => handle,
        Err(_) => ptr::null_mut(),
    }
}

/// Run the feedtui application.
///
/// This function blocks until the user quits the application (e.g., by pressing 'q').
///
/// # Arguments
///
/// * `handle` - A valid handle obtained from `feedtui_init` or `feedtui_init_with_config`.
///
/// # Returns
///
/// * `FeedtuiResult::Success` (0) on successful completion.
/// * Other error codes on failure.
///
/// # Safety
///
/// * `handle` must be a valid pointer returned by `feedtui_init` or `feedtui_init_with_config`.
/// * This function must not be called concurrently from multiple threads.
#[no_mangle]
pub unsafe extern "C" fn feedtui_run(handle: *mut FeedtuiHandle) -> c_int {
    if handle.is_null() {
        return FeedtuiResult::InvalidHandle as c_int;
    }

    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        let handle = &mut *handle;

        let runtime = match handle.runtime.as_ref() {
            Some(rt) => rt,
            None => return FeedtuiResult::RuntimeError as c_int,
        };

        // Create and run the app
        let mut app = App::new(handle.config.clone());

        match runtime.block_on(app.run()) {
            Ok(_) => FeedtuiResult::Success as c_int,
            Err(e) => {
                handle.last_error = CString::new(e.to_string()).ok();
                FeedtuiResult::AppError as c_int
            }
        }
    }));

    match result {
        Ok(code) => code,
        Err(_) => FeedtuiResult::Panic as c_int,
    }
}

/// Shutdown and free the feedtui instance.
///
/// # Arguments
///
/// * `handle` - A valid handle obtained from `feedtui_init` or `feedtui_init_with_config`.
///              After this call, the handle is invalid and must not be used.
///
/// # Safety
///
/// * `handle` must be a valid pointer returned by `feedtui_init` or `feedtui_init_with_config`,
///   or NULL (in which case this function does nothing).
/// * After calling this function, `handle` must not be used again.
#[no_mangle]
pub unsafe extern "C" fn feedtui_shutdown(handle: *mut FeedtuiHandle) {
    if handle.is_null() {
        return;
    }

    let _ = panic::catch_unwind(AssertUnwindSafe(|| {
        let _ = Box::from_raw(handle);
    }));
}

/// Get the last error message.
///
/// # Arguments
///
/// * `handle` - A valid handle obtained from `feedtui_init` or `feedtui_init_with_config`.
///
/// # Returns
///
/// A pointer to a null-terminated UTF-8 string containing the last error message,
/// or NULL if no error has occurred or if the handle is invalid.
///
/// The returned string is owned by the handle and remains valid until:
/// - The next FFI function call on this handle
/// - `feedtui_shutdown` is called
///
/// # Safety
///
/// * `handle` must be a valid pointer returned by `feedtui_init` or `feedtui_init_with_config`.
#[no_mangle]
pub unsafe extern "C" fn feedtui_get_last_error(handle: *const FeedtuiHandle) -> *const c_char {
    if handle.is_null() {
        return ptr::null();
    }

    let handle = &*handle;
    match &handle.last_error {
        Some(err) => err.as_ptr(),
        None => ptr::null(),
    }
}

/// Get the version string of feedtui.
///
/// # Returns
///
/// A pointer to a null-terminated UTF-8 string containing the version.
/// The returned string is statically allocated and valid for the program's lifetime.
#[no_mangle]
pub extern "C" fn feedtui_version() -> *const c_char {
    static VERSION: &[u8] = concat!(env!("CARGO_PKG_VERSION"), "\0").as_bytes();
    VERSION.as_ptr() as *const c_char
}

/// Check if feedtui was compiled with a specific feature.
///
/// # Arguments
///
/// * `feature` - The feature name to check (null-terminated UTF-8 string).
///
/// # Returns
///
/// 1 if the feature is enabled, 0 if not, -1 if the feature name is invalid.
///
/// # Safety
///
/// * `feature` must be a valid null-terminated UTF-8 string or NULL.
#[no_mangle]
pub unsafe extern "C" fn feedtui_has_feature(feature: *const c_char) -> c_int {
    if feature.is_null() {
        return -1;
    }

    let feature_str = match CStr::from_ptr(feature).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    match feature_str {
        "ffi" => 1,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        let version = feedtui_version();
        assert!(!version.is_null());
        let version_str = unsafe { CStr::from_ptr(version) };
        assert!(!version_str.to_str().unwrap().is_empty());
    }

    #[test]
    fn test_init_with_null() {
        let handle = unsafe { feedtui_init(ptr::null()) };
        assert!(!handle.is_null());
        unsafe { feedtui_shutdown(handle) };
    }

    #[test]
    fn test_has_feature() {
        assert_eq!(unsafe { feedtui_has_feature(ptr::null()) }, -1);

        let ffi_feature = CString::new("ffi").unwrap();
        assert_eq!(unsafe { feedtui_has_feature(ffi_feature.as_ptr()) }, 1);

        let unknown_feature = CString::new("unknown").unwrap();
        assert_eq!(unsafe { feedtui_has_feature(unknown_feature.as_ptr()) }, 0);
    }

    #[test]
    fn test_shutdown_null() {
        // Should not panic
        unsafe { feedtui_shutdown(ptr::null_mut()) };
    }
}
