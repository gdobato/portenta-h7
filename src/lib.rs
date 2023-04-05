#![no_std]

pub mod panic;
pub mod sys;

pub use cortex_m_rt::entry;
#[allow(unused_imports)]
pub use rtt_target::rprintln as log;

use hal::{
    gpio::{self, *},
    pac,
    prelude::*,
    rcc,
};
use rtt_target::rtt_init_print as log_init;
use stm32h7xx_hal as hal;

pub type RedUserLed = gpio::gpiok::PK5<Output<PushPull>>;
pub type GreenUserLed = gpio::gpiok::PK6<Output<PushPull>>;
pub type BlueUserLed = gpio::gpiok::PK7<Output<PushPull>>;

pub struct UserLeds {
    pub red: RedUserLed,
    pub green: GreenUserLed,
    pub blue: BlueUserLed,
}

pub struct Board {
    pub user_leds: UserLeds,
}

impl Board {
    pub fn init() -> Option<Self> {
        static mut INITIALIZED: bool = false;
        if unsafe { INITIALIZED } {
            None
        } else {
            unsafe {
                INITIALIZED = true;
            }
            Self::inner_init()
        }
    }

    fn inner_init() -> Option<Self> {
        log_init!();
        #[cfg(debug_assertions)]
        log!("Board init");

        let dp = pac::Peripherals::take().unwrap();
        let sysclk = sys::Clk::take().unwrap();

        // Reset previous configuration and enable external oscillator
        sysclk.reset().enable_ext_clock();

        // Configure power domains and clock tree
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

        // Configure User LEDs
        let gpiok = dp.GPIOK.split(ccdr.peripheral.GPIOK);
        let (red_led, green_led, blue_led) = (
            gpiok.pk5.into_push_pull_output_in_state(PinState::High),
            gpiok.pk6.into_push_pull_output_in_state(PinState::High),
            gpiok.pk7.into_push_pull_output_in_state(PinState::High),
        );

        Some(Board {
            user_leds: UserLeds {
                red: red_led,
                green: green_led,
                blue: blue_led,
            },
        })
    }
}
