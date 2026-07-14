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
        eprintln!();
    };    ($($arg:tt)*) => {
        eprintln!("{}[ERROR] {}{}", $crate::log::ERROR_COLOR, format!($($arg)*), $crate::log::RESET_COLOR);
    };   
}


#[macro_export]
macro_rules! warn {
    () => {
        print("\n");
    };
    ($($arg:tt)*) => {
        eprintln!("{}[WARN] {}{}", $crate::log::WARN_COLOR, format!($($arg)*), $crate::log::RESET_COLOR);
    };
}

#[macro_export]
macro_rules! info {
    () => {
        print("\n");
    };
    ($($arg:tt)*) => {
        eprintln!("{}[INFO] {}{}", $crate::log::INFO_COLOR, format!($($arg)*), $crate::log::RESET_COLOR);
    };
}

#[macro_export]
macro_rules! debug {
    () => {
        print("\n");
    };
    ($($arg:tt)*) => {
        eprintln!("{}[DEBUG] {}{}", $crate::log::DEBUG_COLOR, format!($($arg)*), $crate::log::RESET_COLOR);
    };
}