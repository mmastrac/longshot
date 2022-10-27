#[macro_export]
macro_rules! trace_packet {
    ($($arg:tt)*) => {{
        eprintln!("{}", std::format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! warning {
    ($($arg:tt)*) => {{
        eprintln!("{}", std::format!($($arg)*));
    }};
}
