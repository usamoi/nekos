use crate::println;
use core::panic::PanicInfo;

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message().unwrap(),
        );
    } else {
        println!("Panicked: {}", info.message().unwrap(),);
    }
    loop {}
}
