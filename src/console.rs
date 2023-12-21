#[macro_export]
macro_rules! bold {
    ($arg:expr) => {
        format_args!("\x1b[1m{}\x1b[0m", $arg)
    };
}

#[macro_export]
macro_rules! dim {
    ($arg:expr) => {
        format_args!("\x1b[2m{}\x1b[0m", $arg)
    };
}

/// Clears the entire line and moves the cursor to beginning.
#[macro_export]
macro_rules! clear_line {
    ($stdout:expr) => {{
        $stdout.write_fmt(format_args!("\x1b[2K"))?;
        cursor_beg!($stdout)
    }};
}

/// Moves cursor to beginning of the line.
#[macro_export]
macro_rules! cursor_beg {
    ($stdout:expr) => {
        $stdout.write_fmt(format_args!("\r"))
    };
}

/// Moves cursor to beginning of the previous line.
#[macro_export]
macro_rules! cursor_up {
    ($stdout:expr) => {
        $stdout.write_fmt(format_args!("\x1b[F"))
    };
}
