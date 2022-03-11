pub trait IntExt: Sized {
    fn to_bytes(self) -> [u8; core::mem::size_of::<Self>()];
}

macro impl_intext_for_int($t:tt) {
    impl IntExt for $t {
        fn to_bytes(self) -> [u8; core::mem::size_of::<Self>()] {
            cfg_if::cfg_if! {
                if #[cfg(target_endian = "little")] {
                    self.to_le_bytes()
                } else if #[cfg(target_endian = "big")] {
                    self.to_ge_bytes()
                } else {
                    compile_error!("unknown endian");
                }
            }
        }
    }
}

impl_intext_for_int!(u8);
impl_intext_for_int!(u16);
impl_intext_for_int!(u32);
impl_intext_for_int!(u64);
impl_intext_for_int!(u128);
impl_intext_for_int!(usize);
impl_intext_for_int!(i8);
impl_intext_for_int!(i16);
impl_intext_for_int!(i32);
impl_intext_for_int!(i64);
impl_intext_for_int!(i128);
impl_intext_for_int!(isize);

pub trait UintExt {
    #[must_use]
    fn lowbit(self) -> Self;
}

macro impl_uintext_for_int($t: ty) {
    impl const UintExt for $t {
        fn lowbit(self) -> Self {
            self & self.wrapping_neg()
        }
    }
}

impl_uintext_for_int!(u8);
impl_uintext_for_int!(u16);
impl_uintext_for_int!(u32);
impl_uintext_for_int!(u64);
impl_uintext_for_int!(u128);
impl_uintext_for_int!(usize);

pub trait ConstResultExt {
    type T;
    fn const_ok(self) -> Option<Self::T>;
    fn const_unwrap(self) -> Self::T;
}

impl<T, U> const ConstResultExt for Result<T, U> {
    type T = T;
    fn const_ok(self) -> Option<T>
    where
        Self: ~const Drop,
    {
        match self {
            Ok(t) => Some(t),
            Err(_) => None,
        }
    }
    fn const_unwrap(self) -> T
    where
        Self: ~const Drop,
    {
        match self {
            Ok(t) => t,
            Err(_) => panic!("called `Result::unwrap()` on an `Err` value"),
        }
    }
}

pub trait ConstEq {
    fn const_eq(&self, other: &Self) -> bool;
}

impl const ConstEq for str {
    fn const_eq(&self, other: &Self) -> bool {
        const fn for_each(x: &[u8], y: &[u8]) -> bool {
            match (x, y) {
                ([], []) => true,
                ([x, xs @ ..], [y, ys @ ..]) => *x == *y && for_each(xs, ys),
                _ => false,
            }
        }
        for_each(self.as_bytes(), other.as_bytes())
    }
}

pub trait ConstFromStr<'s>: Copy + 's {
    fn const_from_str(s: &'s str) -> Self;
}

impl<'s> const ConstFromStr<'s> for &'s str {
    fn const_from_str(s: &'s str) -> Self {
        s
    }
}

impl const ConstFromStr<'_> for bool {
    fn const_from_str(s: &str) -> Self {
        match s.as_bytes() {
            [0x74, 0x72, 0x75, 0x65] => true,
            [0x66, 0x61, 0x6c, 0x73, 0x65] => false,
            _ => panic!("parse boolean error"),
        }
    }
}

macro impl_cfs_for_int($t: ty) {
    impl const ConstFromStr<'_> for $t {
        fn const_from_str(s: &str) -> Self {
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
                [0x30, 0x62, bytes @ ..] if bytes.len() > 0 => digits::<2>(bytes),
                [0x2B, 0x30, 0x62, bytes @ ..] if bytes.len() > 0 => 0 + digits::<2>(bytes),
                [0x2D, 0x30, 0x62, bytes @ ..] if bytes.len() > 0 => 0 - digits::<2>(bytes),
                [0x30, 0x6F, bytes @ ..] if bytes.len() > 0 => digits::<8>(bytes),
                [0x2B, 0x30, 0x6F, bytes @ ..] if bytes.len() > 0 => 0 + digits::<8>(bytes),
                [0x2D, 0x30, 0x6F, bytes @ ..] if bytes.len() > 0 => 0 - digits::<8>(bytes),
                [0x30, 0x78, bytes @ ..] if bytes.len() > 0 => digits::<16>(bytes),
                [0x2B, 0x30, 0x78, bytes @ ..] if bytes.len() > 0 => 0 + digits::<16>(bytes),
                [0x2D, 0x30, 0x78, bytes @ ..] if bytes.len() > 0 => 0 - digits::<16>(bytes),
                [0x2B, bytes @ ..] if bytes.len() > 0 => 0 + digits::<10>(bytes),
                [0x2D, bytes @ ..] if bytes.len() > 0 => 0 - digits::<10>(bytes),
                [bytes @ ..]  if bytes.len() > 0 => digits::<10>(bytes),
                _ => panic!("parse integer error")
            }
        }
    }
}

impl_cfs_for_int!(u8);
impl_cfs_for_int!(u16);
impl_cfs_for_int!(u32);
impl_cfs_for_int!(u64);
impl_cfs_for_int!(u128);
impl_cfs_for_int!(usize);
impl_cfs_for_int!(i8);
impl_cfs_for_int!(i16);
impl_cfs_for_int!(i32);
impl_cfs_for_int!(i64);
impl_cfs_for_int!(i128);
impl_cfs_for_int!(isize);
