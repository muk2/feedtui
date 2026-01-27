//! feedtui - A configurable terminal dashboard for stocks, news, sports, and social feeds
//!
//! This crate provides a terminal-based dashboard (TUI) built with ratatui that displays
//! various feeds including Hacker News, RSS feeds, stock prices, sports scores, and more.
//!
//! # Features
//!
//! - **Hacker News**: View top, new, and best stories
//! - **RSS Feeds**: Aggregate multiple RSS feeds
//! - **Stocks**: Real-time stock quotes (requires API)
//! - **Sports**: Live scores and schedules
//! - **GitHub**: Notifications, PRs, and commits
//! - **YouTube**: Latest videos from channels
//! - **Virtual Pet**: Interactive creature companion
//!
//! # FFI Support
//!
//! When compiled with the `ffi` feature, this crate provides a C-compatible interface
//! for embedding feedtui in C/C++ applications. See the [`ffi`] module for details.
//!
//! # Example (Rust)
//!
//! ```no_run
//! use feedtui::config::Config;
//! use feedtui::app::App;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = Config::default();
//!     let mut app = App::new(config);
//!     app.run().await
//! }
//! ```

pub mod app;
pub mod config;
pub mod creature;
pub mod event;
pub mod feeds;
pub mod ui;

#[cfg(feature = "ffi")]
pub mod ffi;

// Re-export FFI functions at crate root for easier linking
#[cfg(feature = "ffi")]
pub use ffi::*;
