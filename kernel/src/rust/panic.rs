use crate::prelude::*;
use core::fmt::Write;
use core::panic::PanicInfo;
use owo_colors::OwoColorize;
use rt::backtrace::backtrace;
use rt::thread::current;
use rt::time::Instant;
use spin::Once;

static HOOK: Once<fn()> = Once::new();

pub fn hook_set_hook(f: fn()) {
    HOOK.call_once(|| f);
}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    if let Some(f) = HOOK.get().cloned() {
        f();
    }

    let mut s = rt::io::stdout().lock();
    writeln!(s).unwrap();

    write!(s, "{}", "Panic".red()).unwrap();
    if let Some(ms) = Instant::now()
        .maybe_duration_since(Instant::ZERO)
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
