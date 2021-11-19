//! Macros used on ouch.

/// Macro that prints \[INFO\] messages, wraps [`println`].
#[macro_export]
macro_rules! info {
    // Show info message even in ACCESSIBLE mode
    (a11y_show, $($arg:tt)*) => {
        // if in ACCESSIBLE mode, suppress the "[INFO]" and just print the message
        if (!$crate::cli::ACCESSIBLE.get().unwrap()) {
            $crate::macros::_info_helper(&mut ::std::io::stdout());
        }
        println!($($arg)*);
    };
    (@$handle: expr, $($arg:tt)*) => {
        let handle = &mut $handle;
        $crate::macros::_info_helper(handle);
        write!(handle, $($arg)*).unwrap();
        ::std::io::Write::flush(handle).unwrap();
    };
    ($($arg:tt)*) => {
        $crate::macros::_info_helper(&mut ::std::io::stdout());
        println!($($arg)*);
    };
}

/// Helper to display "\[INFO\]", colored yellow
pub fn _info_helper(handle: &mut impl std::io::Write) {
    use crate::utils::colors::{RESET, YELLOW};

    write!(handle, "{}[INFO]{} ", *YELLOW, *RESET).unwrap();
}

/// Macro that prints \[WARNING\] messages, wraps [`println`].
#[macro_export]
macro_rules! warning {
    (@$handle: expr, $($arg:tt)*) => {
        let handle = &mut $handle;
        $crate::macros::_warning_helper(handle);
        write!(handle, $($arg)*).unwrap();
        ::std::io::Write::flush(handle).unwrap();
    };
    ($($arg:tt)*) => {
        $crate::macros::_warning_helper(&mut ::std::io::stdout());
        println!($($arg)*);
    };
}

/// Helper to display "\[WARNING\]", colored orange
pub fn _warning_helper(handle: &mut impl std::io::Write) {
    use crate::utils::colors::{ORANGE, RESET};

    if !crate::cli::ACCESSIBLE.get().unwrap() {
        write!(handle, "{}Warning:{} ", *ORANGE, *RESET).unwrap();
    } else {
        write!(handle, "{}[WARNING]{} ", *ORANGE, *RESET).unwrap();
    }
}
