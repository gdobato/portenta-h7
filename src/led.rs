//! led

pub use crate::hal::gpio::{gpiok::*, Output, Pin, PinState, PushPull};
type DigitalOutputPin<const P: char, const N: u8> = Pin<P, N, Output<PushPull>>;

pub trait Led {
    fn on(&mut self);
    fn off(&mut self);
    fn toggle(&mut self);
}

pub mod user {
    use super::*;

    pub type Red = DigitalOutputPin<'K', 5>;
    pub type Green = DigitalOutputPin<'K', 6>;
    pub type Blue = DigitalOutputPin<'K', 7>;

    // Marker trait
    trait BoardLed {}
    impl BoardLed for Red {}
    impl BoardLed for Green {}
    impl BoardLed for Blue {}

    impl<const P: char, const N: u8> Led for DigitalOutputPin<P, N>
    where
        DigitalOutputPin<P, N>: BoardLed,
    {
        #[inline]
        fn on(&mut self) {
            self.set_low();
        }

        #[inline]
        fn off(&mut self) {
            self.set_high();
        }

        #[inline]
        fn toggle(&mut self) {
            self.toggle();
        }
    }
}
