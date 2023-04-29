//! interrupt

pub use crate::hal::pac::interrupt;
use crate::hal::{self, usb_hs::USB1_ULPI};
use crate::CorePeripherals;

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
