use std::sync::atomic::AtomicBool;
pub(crate) static TRACE_ENABLED: AtomicBool = AtomicBool::new(false);

pub fn enable_tracing() {
    TRACE_ENABLED.store(true, std::sync::atomic::Ordering::Relaxed);
}

#[macro_export]
macro_rules! trace_packet {
    ($($arg:tt)*) => {{
        if $crate::logging::TRACE_ENABLED.load(std::sync::atomic::Ordering::Relaxed) {
            eprintln!("[TRACE] {}", std::format!($($arg)*));
        }
    }};
}

#[macro_export]
macro_rules! warning {
    ($($arg:tt)*) => {{
        if $crate::logging::TRACE_ENABLED.load(std::sync::atomic::Ordering::Relaxed) {
            eprintln!("[TRACE] {}", std::format!($($arg)*));
        }
    }};
}
