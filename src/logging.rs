//! Logging utilities.

use std::sync::atomic::AtomicBool;
pub(crate) static TRACE_ENABLED: AtomicBool = AtomicBool::new(false);

/// Enable tracing display to standard error.
pub fn enable_tracing() {
    TRACE_ENABLED.store(true, std::sync::atomic::Ordering::Relaxed);
}

/// Writes a trace of the given communication packet or event if [`enable_tracing`] has been called.
#[macro_export]
macro_rules! trace_packet {
    ($($arg:tt)*) => {{
        if $crate::logging::TRACE_ENABLED.load(std::sync::atomic::Ordering::Relaxed) {
            $crate::display::log($crate::display::LogLevel::Trace, &format!("{}", std::format!($($arg)*)));
        }
    }};
}

/// Writes a trace of the given shutdown event if [`enable_tracing`] has been called.
#[macro_export]
macro_rules! trace_shutdown {
    ($arg:literal) => {{
        if $crate::logging::TRACE_ENABLED.load(std::sync::atomic::Ordering::Relaxed) {
            $crate::display::log(
                $crate::display::LogLevel::Trace,
                &format!("[SHUTDOWN] {}", $arg),
            );
        }
    }};
}

/// Writes a warning of the given event if [`enable_tracing`] has been called.
#[macro_export]
macro_rules! warning {
    ($($arg:tt)*) => {{
        if $crate::logging::TRACE_ENABLED.load(std::sync::atomic::Ordering::Relaxed) {
            $crate::display::log($crate::display::LogLevel::Warning, &std::format!($($arg)*));
        }
    }};
}

/// Writes info text for the given event.
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        $crate::display::log($crate::display::LogLevel::Info, &std::format!($($arg)*));
    }};
}
