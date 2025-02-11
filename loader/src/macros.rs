macro_rules! print {
    ($($args:tt)*) => {{
        #[allow(static_mut_refs)]
        if let Some(writer) = unsafe { WRITER.as_mut() } {
            write!(writer, "{}", format_args!($($args)*)).unwrap();
        }
    }};
}

macro_rules! println {
    ($($args:tt)*) => {{
        print!("{}\n", format_args!($($args)*));
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
