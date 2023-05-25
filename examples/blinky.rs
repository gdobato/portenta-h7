//! Example Blinky
//!
//! Toggles the 3 user LEDs with a different frequency
//!

#![no_std]
#![no_main]

#[allow(unused_imports)]
use portenta_h7::{log, log_init, user_led};
use rtic::app;
use systick_monotonic::{fugit::MillisDurationU64, Systick};

#[app(device = portenta_h7::hal::pac, peripherals = false, dispatchers = [SPI1])]
mod app {
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led_red: user_led::Red,
        led_green: user_led::Green,
        led_blue: user_led::Blue,
    }

    #[monotonic(binds = SysTick, default = true)]
    type MonoTimer = Systick<1_000>;

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        log_init!();

        let mono = Systick::new(cx.core.SYST, portenta_h7::CORE_FREQUENCY.raw());

        // Get boatd resources
        let portenta_h7::Board {
            led_red,
            led_green,
            led_blue,
            ..
        } = portenta_h7::Board::take();

        #[cfg(debug_assertions)]
        log!("spawning tasks");
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
            init::Monotonics(mono),
        )
    }

    #[task(local = [led_red])]
    fn blink_led_red(cx: blink_led_red::Context) {
        #[cfg(debug_assertions)]
        log!("toggling {:?}", cx.local.led_red);
        cx.local.led_red.toggle();
        blink_led_red::spawn_after(MillisDurationU64::from_ticks(500)).unwrap();
    }

    #[task(local = [led_green])]
    fn blink_led_green(cx: blink_led_green::Context) {
        #[cfg(debug_assertions)]
        log!("toggling {:?}", cx.local.led_green);
        cx.local.led_green.toggle();
        blink_led_green::spawn_after(MillisDurationU64::from_ticks(1_000)).unwrap();
    }

    #[task(local = [led_blue])]
    fn blink_led_blue(cx: blink_led_blue::Context) {
        #[cfg(debug_assertions)]
        log!("toggling {:?}", cx.local.led_blue);
        cx.local.led_blue.toggle();
        blink_led_blue::spawn_after(MillisDurationU64::from_ticks(2_000)).unwrap();
    }
}
