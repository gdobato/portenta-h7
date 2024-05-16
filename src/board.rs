//! board

use crate::{hal, led, sys};
use core::sync::atomic::{AtomicBool, Ordering};
pub use hal::usb_hs::{UsbBus, USB1_ULPI as USB};
use hal::{gpio::PinState, pac, prelude::*, rcc, time::Hertz, usb_hs::USB1_ULPI};

pub const CORE_FREQUENCY: Hertz = Hertz::from_raw(480_000_000);

pub struct Board {
    pub led_red: led::user::Red,
    pub led_green: led::user::Green,
    pub led_blue: led::user::Blue,
    pub usb: USB1_ULPI,
}

impl Board {
    pub fn take() -> Self {
        static TAKEN: AtomicBool = AtomicBool::new(false);
        debug_assert!(!TAKEN.swap(true, Ordering::SeqCst));
        Self::setup()
    }

    fn setup() -> Self {
        // Reset previous configuration and enable external oscillator as HSE source (25 MHz)
        sys::Clk::new().reset().enable_ext_clock();
        let dp = pac::Peripherals::take().unwrap();

        // Configure power domains and clock tree
        let pwrcfg = dp.PWR.constrain().vos0(&dp.SYSCFG).freeze();
        let ccdr = dp
            .RCC
            .constrain()
            .use_hse(25.MHz())
            .bypass_hse()
            .sys_ck(CORE_FREQUENCY)
            .hclk(240.MHz())
            .pll1_strategy(rcc::PllConfigStrategy::Iterative)
            .freeze(pwrcfg, &dp.SYSCFG);

        debug_assert_eq!(sys::Clk::get_source(), Some(sys::ClkSource::Pll1));
        debug_assert_eq!(sys::Clk::get_pll_source(), sys::PllSourceVariant::Hse);

        // USB
        let (gpioa, gpiob, gpioc, gpioh, gpioi, gpioj) = {
            (
                dp.GPIOA.split(ccdr.peripheral.GPIOA),
                dp.GPIOB.split(ccdr.peripheral.GPIOB),
                dp.GPIOC.split(ccdr.peripheral.GPIOC),
                dp.GPIOH.split_without_reset(ccdr.peripheral.GPIOH), // Do not do a reset since external oscillator is enabled by GPIOH1
                dp.GPIOI.split(ccdr.peripheral.GPIOI),
                dp.GPIOJ.split(ccdr.peripheral.GPIOJ),
            )
        };
        // Enable ULPI transceiver (GPIOJ4)
        let mut ulpi_reset = gpioj.pj4.into_push_pull_output();
        ulpi_reset.set_high();

        let usb = USB1_ULPI::new(
            dp.OTG1_HS_GLOBAL,
            dp.OTG1_HS_DEVICE,
            dp.OTG1_HS_PWRCLK,
            gpioa.pa5.into_alternate(),
            gpioi.pi11.into_alternate(),
            gpioh.ph4.into_alternate(),
            gpioc.pc0.into_alternate(),
            gpioa.pa3.into_alternate(),
            gpiob.pb0.into_alternate(),
            gpiob.pb1.into_alternate(),
            gpiob.pb10.into_alternate(),
            gpiob.pb11.into_alternate(),
            gpiob.pb12.into_alternate(),
            gpiob.pb13.into_alternate(),
            gpiob.pb5.into_alternate(),
            ccdr.peripheral.USB1OTG,
            &ccdr.clocks,
        );

        // User LEDs
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
            usb,
        }
    }
}
