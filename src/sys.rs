//! sys
//!
//! Clear up previous clock initialization done in bootloader
//! Enable external oscillator for HSE sourcing (25 MHz)
//!

#![allow(dead_code)]

use stm32h7xx_hal::pac;

pub struct Unreset;
pub struct Reset;

pub struct Clk<State> {
    _state: State,
}

pub type ClkSource = pac::rcc::cfgr::SWS_A;
pub type ClkSourceVariant = Option<ClkSource>;
pub type PllSourceVariant = pac::rcc::pllckselr::PLLSRC_A;

impl Clk<Unreset> {
    pub fn new() -> Clk<Unreset> {
        Clk { _state: Unreset }
    }

    #[inline(always)]
    pub fn get_source() -> ClkSourceVariant {
        unsafe { (*pac::RCC::ptr()).cfgr.read().sws().variant() }
    }

    #[inline(always)]
    pub fn get_pll_source() -> PllSourceVariant {
        unsafe { (*pac::RCC::ptr()).pllckselr.read().pllsrc().variant() }
    }

    pub fn reset(self) -> Clk<Reset> {
        let rcc = unsafe { &(*pac::RCC::ptr()) };

        // Enable HSI and load reset values
        rcc.cr.modify(|_, w| w.hsion().set_bit());
        rcc.hsicfgr.reset();
        // Reset clock configuration and wait for clock switch
        rcc.cfgr.reset();
        while !rcc.cfgr.read().sws().is_hsi() {}
        // Reset CSI, HSE, HSI48 and dividers
        rcc.cr.modify(|_, w| {
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
        while rcc.cr.read().hserdy().bit_is_set() {}
        // Disable PLL1
        rcc.cr.modify(|_, w| w.pll1on().clear_bit());
        while rcc.cr.read().pll1rdy().bit_is_set() {}
        // Disable PLL2
        rcc.cr.modify(|_, w| w.pll2on().clear_bit());
        while rcc.cr.read().pll2rdy().bit_is_set() {}
        // Disable PLL3
        rcc.cr.modify(|_, w| w.pll3on().clear_bit());
        while rcc.cr.read().pll3rdy().bit_is_set() {}
        // Reset domain configurations
        rcc.d1cfgr.reset();
        rcc.d2cfgr.reset();
        rcc.d3cfgr.reset();
        // Reset PLLs configurations
        rcc.pllckselr.reset();
        rcc.pll1divr.reset();
        rcc.pll1fracr.reset();
        rcc.pll2divr.reset();
        rcc.pll2fracr.reset();
        rcc.pll3fracr.reset();
        Clk { _state: Reset }
    }
}

impl Clk<Reset> {
    pub fn enable_ext_clock(self) -> Clk<Reset> {
        let rcc = unsafe { &(*pac::RCC::ptr()) };
        // Enable GPIOH clock
        rcc.ahb4enr.modify(|_, w| w.gpiohen().set_bit());

        // Enable oscilator via push pulled GPIOH_1 output
        let gpioh = unsafe { &(*pac::GPIOH::ptr()) };
        gpioh.bsrr.write(|w| w.bs1().set_bit());
        gpioh.moder.modify(|_, w| w.moder1().output());
        gpioh.otyper.modify(|_, w| w.ot1().push_pull());
        gpioh.ospeedr.modify(|_, w| w.ospeedr1().low_speed());
        gpioh.pupdr.modify(|_, w| w.pupdr1().pull_up());

        // Wait for stabilization. TODO: Use proper delay
        for _ in 0..15_000 {
            unsafe { core::arch::asm!("nop") };
        }
        Clk { _state: Reset }
    }
}
