//! Example Blinky
//!
//! Toggles the 3 user LEDs with a different frequency
//!

#![no_std]
#![no_main]

use defmt::info;
use portenta_h7::board::{self, Board, LedBlue, LedGreen, LedRed};
use rtic::app;
use rtic_monotonics::systick::prelude::*;

systick_monotonic!(Mono, 1000);

#[app(device = portenta_h7::hal::pac, peripherals = false, dispatchers = [SPI1])]
mod app {
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led_red: LedRed,
        led_green: LedGreen,
        led_blue: LedBlue,
    }

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
        let _ = blink_led_red::spawn();
        let _ = blink_led_green::spawn();
        let _ = blink_led_blue::spawn();

        (
            Shared {},
            Local {
                led_red,
                led_green,
                led_blue,
            },
        )
    }

    #[task(local = [led_red])]
    async fn blink_led_red(cx: blink_led_red::Context) {
        loop {
            cx.local.led_red.toggle();
            Mono::delay(500.millis()).await;
        }
    }

    #[task(local = [led_green])]
    async fn blink_led_green(cx: blink_led_green::Context) {
        loop {
            cx.local.led_green.toggle();
            Mono::delay(1000.millis()).await;
        }
    }

    #[task(local = [led_blue])]
    async fn blink_led_blue(cx: blink_led_blue::Context) {
        loop {
            cx.local.led_blue.toggle();
            Mono::delay(1000.millis()).await;
        }
    }
}
