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
    usb_hs::USB1_ULPI,
};
pub use hal::{
    interrupt,
    usb_hs::{UsbBus, USB1_ULPI as USB},
};

#[allow(unused_imports)]
pub use rtt_target::rprintln as log;
pub use rtt_target::rtt_init_print as log_init;
use stm32h7xx_hal as hal;

pub type CorePeripherals = cortex_m::Peripherals;
pub type RedUserLed = PK5<Output<PushPull>>;
pub type GreenUserLed = PK6<Output<PushPull>>;
pub type BlueUserLed = PK7<Output<PushPull>>;

pub trait InterruptEnabler {
    fn enable_interrupt(&self);
    fn disable_interrupt(&self);
    fn set_interrupt_priority(&self, cp: &mut CorePeripherals, prio: u8);
}

impl InterruptEnabler for USB1_ULPI {
    fn enable_interrupt(&self) {
        unsafe { cortex_m::peripheral::NVIC::unmask(hal::pac::Interrupt::OTG_HS) };
    }

    fn disable_interrupt(&self) {
        cortex_m::peripheral::NVIC::mask(hal::pac::Interrupt::OTG_HS);
    }

    fn set_interrupt_priority(&self, cp: &mut CorePeripherals, prio: u8) {
        unsafe {
            cp.NVIC.set_priority(hal::pac::Interrupt::OTG_HS, prio);
        }
    }
}

pub struct Board {
    pub cp: CorePeripherals,
    pub led_red: RedUserLed,
    pub led_green: GreenUserLed,
    pub led_blue: BlueUserLed,
    pub usb: USB1_ULPI,
}

impl Board {
    pub fn take() -> Self {
        static TAKEN: AtomicBool = AtomicBool::new(false);
        debug_assert!(!TAKEN.swap(true, Ordering::SeqCst));
        Self::setup()
    }

    fn setup() -> Self {
        #[cfg(debug_assertions)]
        log!("Board init");

        // Reset previous configuration and enable external oscillator as HSE source (25 MHz)
        sys::Clk::new().reset().enable_ext_clock();
        let cp = cortex_m::Peripherals::take().unwrap();
        let dp = pac::Peripherals::take().unwrap();

        // Configure power domains and clock tree
        let pwrcfg = dp.PWR.constrain().vos0(&dp.SYSCFG).freeze();
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
            cp,
            led_red,
            led_green,
            led_blue,
            usb,
        }
    }
}
