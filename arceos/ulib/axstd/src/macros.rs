//! Standard library macros

/// ANSI color codes
pub mod color {
    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const MAGENTA: &str = "\x1b[35m";
    pub const CYAN: &str = "\x1b[36m";
    pub const WHITE: &str = "\x1b[37m";
    pub const RESET: &str = "\x1b[0m";
}

/// Prints to the standard output.
///
/// Equivalent to the [`println!`] macro except that a newline is not printed at
/// the end of the message.
///
/// [`println!`]: crate::println
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::io::__print_impl(format_args!($($arg)*));
    }
}

/// Prints to the standard output, with a newline (with color).
#[macro_export]
macro_rules! println {
    () => { $crate::print!("\n") };
    ($($arg:tt)*) => {
        $crate::io::__print_impl(format_args!("\x1b[32m{}\x1b[0m\n", format_args!($($arg)*)));
    }
}

/// Prints colored text to the standard output.
#[macro_export]
macro_rules! print_color {
    ($color:expr, $($arg:tt)*) => {
        $crate::io::__print_impl(format_args!("{}{}{}", $color, format_args!($($arg)*), $crate::macros::color::RESET));
    }
}

/// Prints colored text to the standard output, with a newline.
#[macro_export]
macro_rules! println_color {
    ($color:expr, $($arg:tt)*) => {
        $crate::io::__print_impl(format_args!("{}{}{}\n", $color, format_args!($($arg)*), $crate::macros::color::RESET));
    }
}
