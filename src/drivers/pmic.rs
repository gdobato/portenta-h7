//! Pmic WIP

const PMIC_ADDR: u8 = 0x08;

#[derive(Clone, Copy, Debug)]
pub enum Reg {
    DeviceId = 0x00,
}

impl Reg {
    pub const fn as_u8(&self) -> u8 {
        *self as u8
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Error {
    I2cError,
}

pub struct Pmic<I2C> {
    i2c: I2C,
}

impl<I2C> Pmic<I2C> {
    pub fn new(i2c: I2C) -> Self {
        Self { i2c }
    }
}

impl<I2C: embedded_hal_v0::blocking::i2c::WriteRead> Pmic<I2C> {
    pub fn device_id(&mut self) -> Result<u8, Error> {
        let mut data = [0u8];
        self.i2c
            .write_read(PMIC_ADDR, &[Reg::DeviceId.as_u8()], &mut data)
            .map_err(|_| Error::I2cError)?;
        Ok(data[0])
    }
}
