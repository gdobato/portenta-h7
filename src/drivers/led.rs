//! led

pub struct Led<PIN> {
    pin: PIN,
}

impl<PIN> Led<PIN> {
    pub fn new(pin: PIN) -> Self {
        Self { pin }
    }
}

impl<PIN: embedded_hal_v0::digital::v2::OutputPin> Led<PIN> {
    pub fn on(&mut self) {
        let _ = self.pin.set_low();
    }

    pub fn off(&mut self) {
        let _ = self.pin.set_high();
    }
}

impl<PIN: embedded_hal_v0::digital::v2::ToggleableOutputPin> Led<PIN> {
    pub fn toggle(&mut self) {
        let _ = self.pin.toggle();
    }
}
