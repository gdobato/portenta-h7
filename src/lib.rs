#![no_std]

pub mod board;
pub mod drivers;
mod sys;
pub use cortex_m_rt::entry;
#[allow(unused)]
use defmt_rtt as _;
use panic_probe as _;
pub use stm32h7xx_hal as hal;
