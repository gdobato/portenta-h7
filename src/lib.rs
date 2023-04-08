#![no_std]

pub mod panic;
mod sys;

use core::sync::atomic::{AtomicBool, Ordering};
pub use cortex_m_rt::entry;
use hal::{
    gpio::{gpiok::*, Output, PinState, PushPull},
    pac,
    prelude::*,
    rcc,
};
#[allow(unused_imports)]
pub use rtt_target::rprintln as log;
use rtt_target::rtt_init_print as log_init;
use stm32h7xx_hal as hal;

pub type RedUserLed = PK5<Output<PushPull>>;
pub type GreenUserLed = PK6<Output<PushPull>>;
pub type BlueUserLed = PK7<Output<PushPull>>;

pub struct Board {
    pub led_red: RedUserLed,
    pub led_green: GreenUserLed,
    pub led_blue: BlueUserLed,
}

impl Board {
    pub fn take() -> Self {
        static TAKEN: AtomicBool = AtomicBool::new(false);
        debug_assert!(!TAKEN.swap(true, Ordering::SeqCst));
        Self::setup()
    }

    fn setup() -> Self {
        log_init!();

        #[cfg(debug_assertions)]
        log!("Board init");

        // Reset previous configuration and enable external oscillator
        sys::Clk::new().reset().enable_ext_clock();

        // Configure power domains and clock tree
        let dp = pac::Peripherals::take().unwrap();
        let pwr = dp.PWR.constrain();
        let pwrcfg = pwr.vos0(&dp.SYSCFG).freeze();
        let ccdr = dp
            .RCC
            .constrain()
            .use_hse(25.MHz())
            .bypass_hse()
            .sys_ck(480.MHz())
            .hclk(240.MHz())
            .pll1_strategy(rcc::PllConfigStrategy::Iterative)
            .freeze(pwrcfg, &dp.SYSCFG);

        debug_assert_eq!(sys::Clk::get_source(), Some(sys::ClkSource::Pll1));
        debug_assert_eq!(sys::Clk::get_pll_source(), sys::PllSourceVariant::Hse);

        // Configure User LEDs
        let gpiok = dp.GPIOK.split(ccdr.peripheral.GPIOK);
        let (led_red, led_green, led_blue) = (
            gpiok.pk5.into_push_pull_output_in_state(PinState::High),
            gpiok.pk6.into_push_pull_output_in_state(PinState::High),
            gpiok.pk7.into_push_pull_output_in_state(PinState::High),
        );

        Board {
            led_red,
            led_green,
            led_blue,
        }
    }
}
