//! PMIC abstraction

use hal::{i2c::*, prelude::*, stm32::I2C1, *};
use stm32h7xx_hal as hal;

const PMIC_ADDR: u8 = 0x08;
const PMIC_CONFIG_SEQUENCE: &[&[u8]] = &[
    &[0x4F, 0x00], // LDO2 : 1.8V
    &[0x50, 0x0F], // Enable LDO2
    &[0x4C, 0x05], // LDO1 : 1.0V
    &[0x4D, 0x03], // Enable LDO1
    &[0x52, 0x09], // LDO3 : 1.2V
    &[0x53, 0x0F], // Enable LDO3
    &[0x9C, 0x80], // TODO : Value observed but not reference found
    &[0x9E, 0x20], // Disable charger LED
    &[0x42, 0x02], // Current limit : 2A
    &[0x94, 0xA0], // VBUS current limit : 1.5A
    &[0x3B, 0x0F], // SW2 : 3.3V
    &[0x35, 0x0F], // SW1 : 3.0V
];

#[derive(Debug)]
pub enum Error {
    Interface,
    NotResponse,
    Unknown,
}

pub struct Pmic<'a> {
    i2c_bus: &'a mut I2c<I2C1>,
}

impl<'a> Pmic<'a> {
    pub fn bind(i2c_bus: &'a mut I2c<I2C1>) -> Pmic<'a> {
        Pmic { i2c_bus }
    }

    pub fn configure(&mut self) -> Result<(), Error> {
        for config_step in PMIC_CONFIG_SEQUENCE {
            self.i2c_bus.write(PMIC_ADDR, config_step)?;
        }
        Ok(())
    }
}

impl From<i2c::Error> for Error {
    fn from(i2c_error: i2c::Error) -> Self {
        match i2c_error {
            i2c::Error::Bus | i2c::Error::Arbitration => Error::Interface,
            i2c::Error::NotAcknowledge => Error::NotResponse,
            _ => Error::Unknown,
        }
    }
}
