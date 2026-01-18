#[macro_export]
macro_rules! print_err {
    () => {};
    ($($arg:tt)*) => {
        if std::io::IsTerminal::is_terminal(&std::io::stderr()) {
            eprint!("\x1b[1;31m");
            eprint!($($arg)*);
            eprint!("\x1b[0m");
        } else {
            eprint!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! println_err {
    () => {};
    ($($arg:tt)*) => {
        if std::io::IsTerminal::is_terminal(&std::io::stderr()) {
            eprint!("\x1b[1;31m");
            eprint!($($arg)*);
            eprintln!("\x1b[0m");
        } else {
            eprintln!($($arg)*);
        }
    };
}
