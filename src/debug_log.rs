#[macro_export]
macro_rules! debug_log {
    ($($tt:tt)*) => {
        {
            let _ = format_args!($($tt)*);
        }
    };
}

#[macro_export]
macro_rules! debug_err {
    ($($tt:tt)*) => {
        {
            let _ = format_args!($($tt)*);
        }
    };
}
