use crate::prelude::*;
use base::cell::SingletonCell;
use core::fmt::Write;
use core::panic::PanicInfo;
use owo_colors::OwoColorize;
use rt::backtrace::backtrace;
use rt::thread::current;
use rt::time::Instant;

static HOOK: SingletonCell<fn()> = SingletonCell::new();

pub fn init_global(f: fn()) {
    HOOK.initialize(f);
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    if let Some(f) = HOOK.maybe().cloned() {
        f();
    }

    let mut s = rt::io::stdout().lock();
    writeln!(s).unwrap();

    write!(s, "{}", "Panic".red()).unwrap();
    if let Some(ms) = Instant::maybe_now()
        .map(|x| x - Instant::ZERO)
        .map(|x| x.as_millis())
    {
        write!(s, " [{:#2}.{:#03}]", ms / 1000, ms % 1000).unwrap();
    }
    write!(s, " [CPU {}]", current().id()).unwrap();
    if let Some(location) = info.location() {
        let file = location.file();
        let line = location.line();
        write!(s, " [{}:{}]", file, line).unwrap();
    }
    writeln!(s, " {}", info.message().unwrap()).unwrap();

    writeln!(s, "[Backtrace]").unwrap();
    for stack_frame in unsafe { backtrace() } {
        writeln!(s, "{{ {:?} }}", stack_frame).unwrap();
    }

    rt::process::abort();
}
