//! Example LED blinky

#![no_std]
#![no_main]

use core::cell::RefCell;
use cortex_m_rt::entry;
use hal::{gpio::*, pac, prelude::*, rcc};
use non_preemptive_scheduler::{resources::UnShared, EventMask, Scheduler, Task};
use non_preemptive_scheduler_macros as scheduler;
#[allow(unused_imports)]
use rtt_target::{rprintln as log, rtt_init_print as log_init};
use stm32h7xx_hal as hal;

// Static and interior mutable entities
static RED_LED: UnShared<RefCell<Option<PK5<Output<PushPull>>>>> =
    UnShared::new(RefCell::new(None));
static GREEN_LED: UnShared<RefCell<Option<PK6<Output<PushPull>>>>> =
    UnShared::new(RefCell::new(None));
static BLUE_LED: UnShared<RefCell<Option<PK7<Output<PushPull>>>>> =
    UnShared::new(RefCell::new(None));

// Create scheduler
#[scheduler::new(task_count = 3, core_freq = 480_000_000)]
struct NonPreemptiveScheduler;

// Functions which are bound to task runnables
fn red_led_blinky(_: EventMask) {
    if let Some(red_led) = RED_LED.borrow().borrow_mut().as_mut() {
        red_led.toggle();
    }
}

fn green_led_blinky(_: EventMask) {
    if let Some(green_led) = GREEN_LED.borrow().borrow_mut().as_mut() {
        green_led.toggle();
    }
}

fn blue_led_blinky(_: EventMask) {
    if let Some(blue_led) = BLUE_LED.borrow().borrow_mut().as_mut() {
        blue_led.toggle();
    }
}

fn bsp_init() {
    let sysclk = portenta_rs::sys::Clk::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    sysclk.reset().enable_ext_clock();

    let pwr = dp.PWR.constrain();
    let pwrcfg = pwr.vos0(&dp.SYSCFG).freeze();

    let ccdr = dp
        .RCC
        .constrain()
        .use_hse(27.MHz())
        .bypass_hse()
        .sys_ck(480.MHz())
        .hclk(240.MHz())
        .pll1_strategy(rcc::PllConfigStrategy::Iterative)
        .pll1_q_ck(240.MHz())
        .pll2_strategy(rcc::PllConfigStrategy::Iterative)
        .pll2_p_ck(100.MHz())
        .pll3_strategy(rcc::PllConfigStrategy::Iterative)
        .pll3_p_ck(100.MHz())
        .pll3_r_ck(100.MHz())
        .freeze(pwrcfg, &dp.SYSCFG);

    // User LEDs
    let gpiok = dp.GPIOK.split(ccdr.peripheral.GPIOK);
    RED_LED.borrow().replace(Some(
        gpiok.pk5.into_push_pull_output_in_state(PinState::High),
    ));
    GREEN_LED.borrow().replace(Some(
        gpiok.pk6.into_push_pull_output_in_state(PinState::High),
    ));
    BLUE_LED.borrow().replace(Some(
        gpiok.pk7.into_push_pull_output_in_state(PinState::High),
    ));
}

#[entry]
fn main() -> ! {
    log_init!();

    bsp_init();

    // Add tasks
    scheduler::add_task!(
        "red_led_blinky",
        None,
        Some(red_led_blinky),
        Some(250),
        Some(3)
    );

    scheduler::add_task!(
        "green_led_blinky",
        None,
        Some(green_led_blinky),
        Some(500),
        Some(281)
    );

    scheduler::add_task!(
        "blue_led_blinky",
        None,
        Some(blue_led_blinky),
        Some(1_000),
        Some(523)
    );

    scheduler::launch!();

    loop {
        panic!("Not expected execution");
    }
}
