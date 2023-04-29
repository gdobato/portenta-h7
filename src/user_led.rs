//! led

pub use crate::hal::gpio::{gpiok::*, Output, Pin, PinState, PushPull};

type DigitalOutputPin<const P: char, const N: u8> = Pin<P, N, Output<PushPull>>;
pub type Red = DigitalOutputPin<'K', 5>;
pub type Green = DigitalOutputPin<'K', 6>;
pub type Blue = DigitalOutputPin<'K', 7>;

pub trait BoardLed {}

impl BoardLed for Red {}
impl BoardLed for Green {}
impl BoardLed for Blue {}

pub trait UserLed {
    fn set_on(&mut self);
    fn set_off(&mut self);
    fn toggle(&mut self);
}

impl<const P: char, const N: u8> UserLed for DigitalOutputPin<P, N>
where
    DigitalOutputPin<P, N>: BoardLed,
{
    #[inline(always)]
    fn set_on(&mut self) {
        self.set_low();
    }

    #[inline(always)]
    fn set_off(&mut self) {
        self.set_high();
    }

    #[inline(always)]
    fn toggle(&mut self) {
        self.toggle();
    }
}
