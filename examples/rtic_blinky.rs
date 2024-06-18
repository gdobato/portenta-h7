//! Example of blinky
//!
//! Toggles the 3 user LEDs with a different frequency
//!

#![no_std]
#![no_main]

use defmt::info;
use portenta_h7::board::{
    self,
    non_async_impl::{Board, LedBlue, LedGreen, LedRed},
};
use rtic::app;
use rtic_monotonics::systick::prelude::*;

systick_monotonic!(Mono, 1000);

#[app(device = portenta_h7::hal::pac, peripherals = false, dispatchers = [SPI1])]
mod app {
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        info!("Init");

        Mono::start(cx.core.SYST, board::CORE_FREQUENCY.raw());

        // Get board resources
        let Board {
            led_red,
            led_green,
            led_blue,
            ..
        } = Board::take();

        #[cfg(debug_assertions)]
        info!("spawning tasks");
        let _ = blink_led_red::spawn(led_red);
        let _ = blink_led_green::spawn(led_green);
        let _ = blink_led_blue::spawn(led_blue);

        (Shared {}, Local {})
    }

    #[task]
    async fn blink_led_red(_cx: blink_led_red::Context, mut led: LedRed) {
        loop {
            led.toggle();
            Mono::delay(500.millis()).await;
        }
    }

    #[task]
    async fn blink_led_green(_cx: blink_led_green::Context, mut led: LedGreen) {
        loop {
            led.toggle();
            Mono::delay(1000.millis()).await;
        }
    }

    #[task]
    async fn blink_led_blue(_cx: blink_led_blue::Context, mut led: LedBlue) {
        loop {
            led.toggle();
            Mono::delay(2000.millis()).await;
        }
    }
}
