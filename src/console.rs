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
