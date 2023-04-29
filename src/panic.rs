//! panic

use core::panic::PanicInfo;
#[cfg(debug_assertions)]
use rtt_target::rprintln as log;

#[panic_handler]
#[allow(unused_variables)]
fn panic(info: &PanicInfo) -> ! {
    #[cfg(debug_assertions)]
    log!("{}", info);
    loop {}
}
