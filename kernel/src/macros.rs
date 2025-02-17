macro_rules! print {
    ($($arg:tt)*) => {{
        #[allow(static_mut_refs)]
        if let Some(writer) = unsafe { CONSOLE.as_mut() } {
            write!(writer, "{}", format_args!($($arg)*)).unwrap();
        }
    }}
}

macro_rules! println {
    () => {{
        print!("\n");
    }};
    ($($arg:tt)*) => {{
        print!("{}\n", format_args!($($arg)*));
    }};
}

macro_rules! dbg {
    () => {{
        println!("[{}:{}]", file!(), line!());
    }};
    ($arg:expr $(,)?) => {{
        println!(
            "[{}:{}] {} = {:#?}",
            file!(),
            line!(),
            stringify!($arg),
            $arg
        );
    }};
    ($($val:expr),+ $(,)?) => {
        ($(dbg!($val)),+,)
    };
}
