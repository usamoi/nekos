use crate::prelude::*;
use arch::cpu::checked_local;
use arch::power::POWER;
use arch::stdout::STDOUT;
use arch::time::SystemTime;
use arch::trampoline::fault;
use core::fmt::Write;
use core::panic::PanicInfo;
use owo_colors::OwoColorize;

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    unsafe {
        fault::panic_handler();
    }

    let s = &mut *STDOUT.write.lock();
    writeln!(s).unwrap();

    write!(s, "{}", "Panic".red()).unwrap();
    if let Some(ms) = SystemTime::now()
        .checked_duration_since(SystemTime::ZERO)
        .map(|x| x.as_millis())
    {
        write!(s, " [{:#2}.{:#03}]", ms / 1000, ms % 1000).unwrap();
    }
    if let Some(id) = checked_local().and_then(|local| local.get_id()) {
        write!(s, " [CPU {}]", id).unwrap();
    }
    if let Some(location) = info.location() {
        let file = location.file();
        let line = location.line();
        write!(s, " [{}:{}]", file, line).unwrap();
    }
    writeln!(s, " {}", info.message().unwrap()).unwrap();

    writeln!(s, "[Backtrace]").unwrap();
    for stack_frame in arch::common::backtrace::backtrace!() {
        writeln!(s, "{{ {:?} }}", stack_frame).unwrap();
    }

    POWER.shutdown();
}
