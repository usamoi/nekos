pub struct _Stdout;

impl core::fmt::Write for _Stdout {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        crate::syscalls::debug_write(s).unwrap();
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    () => ();
    ($($arg:tt)*) => ({
        core::fmt::Write::write_fmt(&mut $crate::macros::_Stdout, core::format_args!($($arg)*)).unwrap();
    });
}

#[macro_export]
macro_rules! println {
    () => ({
        core::fmt::Write::write_fmt(&mut $crate::macros::_Stdout, core::format_args_nl!("")).unwrap();
    });
    ($($arg:tt)*) => ({
        core::fmt::Write::write_fmt(&mut $crate::macros::_Stdout, core::format_args_nl!($($arg)*)).unwrap();
    })
}
