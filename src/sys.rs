//! Sys
//!
//! Clear up previous clock initilization done in bootloader
//! Enable external oscilator for HSE sourcing (25 MHz)
//!

use stm32h7xx_hal::pac;

pub struct Unreset;
pub struct Reset;
pub struct ExtClockEnabled;
pub struct ExtClockDisabled;

pub struct Clk;

pub struct ClkCfg<State, ExtClock> {
    _state: State,
    _ext_clock: ExtClock,
}

impl Clk {
    pub fn take() -> Option<ClkCfg<Unreset, ExtClockDisabled>> {
        static mut TAKEN: bool = false;
        if unsafe { TAKEN } {
            None
        } else {
            unsafe { TAKEN = true };
            Some(ClkCfg {
                _state: Unreset,
                _ext_clock: ExtClockDisabled,
            })
        }
    }
}

impl<State, ExtClock> ClkCfg<State, ExtClock> {
    pub fn reset(self) -> ClkCfg<Reset, ExtClockDisabled> {
        unsafe {
            let rcc_blck = &(*pac::RCC::PTR);
            // Enable HSI and load reset values
            rcc_blck.cr.modify(|_, w| w.hsion().set_bit());
            rcc_blck.hsicfgr.reset();
            // Reset clock configuration and wait for clock switch
            rcc_blck.cfgr.reset();
            while !rcc_blck.cfgr.read().sws().is_hsi() {}
            // Reset CSI, HSE, HSI48 and dividers
            rcc_blck.cr.modify(|_, w| {
                w.hseon()
                    .off()
                    .hsikeron()
                    .off()
                    .hsidiv()
                    .div1()
                    .hsidivf()
                    .clear_bit()
                    .csion()
                    .off()
                    .csikeron()
                    .off()
                    .hsi48on()
                    .off()
                    .hsecsson()
                    .off()
                    .hsebyp()
                    .clear_bit()
            });
            while rcc_blck.cr.read().hserdy().bit_is_set() {}
            // Disable PLL1
            rcc_blck.cr.modify(|_, w| w.pll1on().clear_bit());
            while rcc_blck.cr.read().pll1rdy().bit_is_set() {}
            // Disable PLL2
            rcc_blck.cr.modify(|_, w| w.pll2on().clear_bit());
            while rcc_blck.cr.read().pll2rdy().bit_is_set() {}
            // Disable PLL3
            rcc_blck.cr.modify(|_, w| w.pll3on().clear_bit());
            while rcc_blck.cr.read().pll3rdy().bit_is_set() {}
            // Reset domain configurations
            rcc_blck.d1cfgr.reset();
            rcc_blck.d2cfgr.reset();
            rcc_blck.d3cfgr.reset();
            // Reset PLLs configurations
            rcc_blck.pllckselr.reset();
            rcc_blck.pll1divr.reset();
            rcc_blck.pll1fracr.reset();
            rcc_blck.pll2divr.reset();
            rcc_blck.pll2fracr.reset();
            rcc_blck.pll3fracr.reset();
        }
        ClkCfg {
            _state: Reset,
            _ext_clock: ExtClockDisabled,
        }
    }
}

impl<ExtClock> ClkCfg<Reset, ExtClock> {
    pub fn enable_ext_clock(self) -> ClkCfg<Reset, ExtClockEnabled> {
        unsafe {
            // Enable GPIOH clock
            let rcc_blck = &(*pac::RCC::PTR);
            rcc_blck.ahb4enr.modify(|_, w| w.gpiohen().set_bit());

            // Enable oscilator via push pulled GPIOH_1 output
            let gpioh_blck = &(*pac::GPIOH::PTR);
            gpioh_blck.bsrr.write(|w| w.bs1().set_bit());
            gpioh_blck.moder.modify(|_, w| w.moder1().output());
            gpioh_blck.otyper.modify(|_, w| w.ot1().push_pull());
            gpioh_blck.ospeedr.modify(|_, w| w.ospeedr1().low_speed());
            gpioh_blck.pupdr.modify(|_, w| w.pupdr1().pull_up());

            // Wait for stabilization. TODO: Use proper delay
            for _ in 0..10_000 {
                core::arch::asm!("nop");
            }
        }
        ClkCfg {
            _state: Reset,
            _ext_clock: ExtClockEnabled,
        }
    }
}
