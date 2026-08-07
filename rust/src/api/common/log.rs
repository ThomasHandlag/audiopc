// This module provides a simple logging interface that can be easily redirected to different backends.
// By default, it just prints to stderr, but it can be configured to use the `log` crate for more advanced logging. 

pub const ERROR_COLOR: &str = "\x1b[31m"; // Red
pub const WARN_COLOR: &str = "\x1b[33m"; // Yellow
pub const INFO_COLOR: &str = "\x1b[32m"; // Green
pub const DEBUG_COLOR: &str = "\x1b[34m"; // Blue
pub const RESET_COLOR: &str = "\x1b[0m";

#[macro_export]
macro_rules! error {
    () => {
        log::info!();
    };    ($($arg:tt)*) => {
        log::info!("{}[ERROR] {}{}", $crate::api::common::log::ERROR_COLOR, format!($($arg)*), $crate::api::common::log::RESET_COLOR)
    };
}


#[macro_export]
macro_rules! warn {
    () => {
        log::info!("\n");
    };
    ($($arg:tt)*) => {
        log::info!("{}[WARN] {}{}", $crate::api::common::log::WARN_COLOR, format!($($arg)*), $crate::api::common::log::RESET_COLOR)
    };
}

#[macro_export]
macro_rules! info {
    () => {
        log::info!("\n");
    };
    ($($arg:tt)*) => {
        log::info!("{}[INFO] {}{}", $crate::api::common::log::INFO_COLOR, format!($($arg)*), $crate::api::common::log::RESET_COLOR)
    };
}

#[macro_export]
macro_rules! debug {
    () => {
        log::info!("\n");
    };
    ($($arg:tt)*) => {
        log::info!("{}[DEBUG] {}{}", $crate::api::common::log::DEBUG_COLOR, format!($($arg)*), $crate::api::common::log::RESET_COLOR)
    };
}