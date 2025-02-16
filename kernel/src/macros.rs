use crate::{FontWriter, WRITER};
use core::fmt::Write;

macro_rules! print {
    ($($arg:tt)*) => {{
        if let Some(writer) = unsafe { WRITER.as_mut() } {
            write!(writer, "{}", format_args!($($arg)*)).unwrap();
        }
    }}
}
