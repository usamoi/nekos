use crate::prelude::*;

pub macro print {
    () => (),
    ($($arg:tt)*) => {
        {
            let mut s = rt::io::stdout().lock();
            core::fmt::Write::write_fmt(&mut s, core::format_args!($($arg)*)).unwrap();
        }
    }
}

pub macro println {
    () => {
        {
            let mut s = rt::io::stdout().lock();
            core::fmt::Write::write_str(&mut s, "\n").unwrap();
        }
    },
    ($($arg:tt)*) => {
        {
            let mut s = rt::io::stdout().lock();
            core::fmt::Write::write_fmt(&mut s, core::format_args!($($arg)*)).unwrap();
            core::fmt::Write::write_str(&mut s, "\n").unwrap();
        }
    }
}

pub macro dbg {
    () => {
        debug!("[{}:{}]", file!(), line!())
    },
    ($val:expr $(,)?) => {
        match $val {
            tmp => {
                debug!("[{}:{}] {} = {:#?}", file!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    },
    ($($val:expr),+ $(,)?) => {
        ($(dbg!($val)),+,)
    }
}
