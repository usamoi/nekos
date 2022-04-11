use crate::prelude::*;
use core::time::Duration;

pub trait ConstEq {
    fn const_eq(&self, other: &Self) -> bool;
}

impl const ConstEq for str {
    fn const_eq(&self, other: &Self) -> bool {
        const fn f(x: &[u8], y: &[u8]) -> bool {
            match (x, y) {
                ([], []) => true,
                ([x, xs @ ..], [y, ys @ ..]) => *x == *y && f(xs, ys),
                _ => false,
            }
        }
        f(self.as_bytes(), other.as_bytes())
    }
}

pub trait ConstParse<'s>: Copy + 's {
    fn const_parse(s: &'s str) -> Self;
}

impl<'s> const ConstParse<'s> for &'s str {
    fn const_parse(s: &'s str) -> Self {
        s
    }
}

impl const ConstParse<'_> for bool {
    fn const_parse(s: &str) -> Self {
        match s.as_bytes() {
            [0x74, 0x72, 0x75, 0x65] => true,
            [0x66, 0x61, 0x6c, 0x73, 0x65] => false,
            _ => panic!("parse boolean error"),
        }
    }
}

macro impl_cp_for_int($t: ty) {
    impl const ConstParse<'_> for $t {
        fn const_parse(s: &str) -> Self {
            const fn digit<const RADIX: u8>(c: char) -> u8 {
                let x = match c {
                    x @ '0'..='9' => x as u8 - '0' as u8,
                    x @ 'a'..='z' => x as u8 - 'a' as u8 + 10,
                    x @ 'A'..='Z' => x as u8 - 'A' as u8 + 10,
                    _ => panic!("parse integer error"),
                };
                if x < RADIX {
                    x
                } else {
                    panic!("parse integer error")
                }
            }
            const fn digits<const RADIX: u8>(s: &[u8]) -> $t {
                match s {
                    [] => 0,
                    [x @ .., 0x5F /* ascii char '_' */] => digits::<RADIX>(x),
                    [x @ .., c] => {
                        digits::<RADIX>(x) * RADIX as $t + digit::<RADIX>(*c as char) as $t
                    }
                }
            }
            match s.as_bytes() {
                [b'0', b'b', bytes @ ..] if bytes.len() > 0 => digits::<2>(bytes),
                [b'+', b'0', b'b', bytes @ ..] if bytes.len() > 0 => 0 + digits::<2>(bytes),
                [b'-', b'0', b'b', bytes @ ..] if bytes.len() > 0 => 0 - digits::<2>(bytes),
                [b'0', b'o', bytes @ ..] if bytes.len() > 0 => digits::<8>(bytes),
                [b'+', b'0', b'o', bytes @ ..] if bytes.len() > 0 => 0 + digits::<8>(bytes),
                [b'-', b'0', b'o', bytes @ ..] if bytes.len() > 0 => 0 - digits::<8>(bytes),
                [b'0', b'x', bytes @ ..] if bytes.len() > 0 => digits::<16>(bytes),
                [b'+', b'0', b'x', bytes @ ..] if bytes.len() > 0 => 0 + digits::<16>(bytes),
                [b'-', b'0', b'x', bytes @ ..] if bytes.len() > 0 => 0 - digits::<16>(bytes),
                [b'+', bytes @ ..] if bytes.len() > 0 => 0 + digits::<10>(bytes),
                [b'-', bytes @ ..] if bytes.len() > 0 => 0 - digits::<10>(bytes),
                [bytes @ ..]  if bytes.len() > 0 => digits::<10>(bytes),
                _ => panic!("parse integer error")
            }
        }
    }
}

impl_cp_for_int!(u8);
impl_cp_for_int!(u16);
impl_cp_for_int!(u32);
impl_cp_for_int!(u64);
impl_cp_for_int!(u128);
impl_cp_for_int!(usize);
impl_cp_for_int!(i8);
impl_cp_for_int!(i16);
impl_cp_for_int!(i32);
impl_cp_for_int!(i64);
impl_cp_for_int!(i128);
impl_cp_for_int!(isize);

macro env_match(match env!($x: literal) { $($y: literal => Some($z: expr)),+ $(,)? _ => None }) {{
    $( if env!($x).const_eq($y) { Some($z) } else )+
    { None }
}}

macro env_cast($x: literal) {{
    ConstParse::const_parse(env!($x))
}}

// kernel
pub static KERNEL_ADDRESS: PAddr = PAddr::new(env_cast!("kernel_address"));

// logging
pub const LOGGING_LEVEL: log::Level = env_match! {
    match env!("logging_level") {
        "trace" => Some(log::Level::Trace),
        "debug" => Some(log::Level::Debug),
        "info" => Some(log::Level::Info),
        "warn" => Some(log::Level::Warn),
        "error" => Some(log::Level::Error),
        _ => None
    }
}
.unwrap();

// backtrace

pub const BACKTRACE_LIMIT: usize = env_cast!("backtrace_limit");

// memory
pub const FALLBACK_HEAP_SIZE: usize = 16 * 1024 * 1024;
pub const STACK_SIZE: usize = 2 * 1024 * 1024;
pub const FAULT_STACK_SIZE: usize = 8 * 1024;

// process
pub const PROCESS_RESERVE_HANDLES: usize = 65536;
pub const THREAD_STACK_LAYOUT: MapLayout = MapLayout::new(16 * 1024, 4096).unwrap();

// schedule
pub const SCHEDULE_TIMESLICE: Duration = Duration::from_millis(10);
