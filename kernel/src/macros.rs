macro_rules! print {
    ($($arg:tt)*) => {{
        write!(console(), "{}", format_args!($($arg)*)).unwrap();
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

macro_rules! r#mod {
    ($($arg:tt),+) => {
        $(
            mod $arg;
        )+
    };
}
