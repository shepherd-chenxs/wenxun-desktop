#[macro_export]
macro_rules! log_info {
    ($module:expr, $($arg:tt)*) => {
        log::info!("[{}] {}", $module, format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_error {
    ($module:expr, $($arg:tt)*) => {
        log::error!("[{}] {}", $module, format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_warn {
    ($module:expr, $($arg:tt)*) => {
        log::warn!("[{}] {}", $module, format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_debug {
    ($module:expr, $($arg:tt)*) => {
        log::debug!("[{}] {}", $module, format!($($arg)*))
    };
}