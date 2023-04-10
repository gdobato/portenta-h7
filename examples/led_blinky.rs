//! Example LED blinky
//!
//! Toggles the user LEDs every second, using a basic not preemptive scheduler
//!

#![no_std]
#![no_main]

use core::cell::RefCell;
use non_preemptive_scheduler as scheduler;
use portenta_h7::{entry, log_init, BlueUserLed, GreenUserLed, RedUserLed};
use scheduler::{resources::UnShared, EventMask, Scheduler, Task};

// Static and interior mutable entities
static LED_RED: UnShared<RefCell<Option<RedUserLed>>> = UnShared::new(RefCell::new(None));
static LED_GREEN: UnShared<RefCell<Option<GreenUserLed>>> = UnShared::new(RefCell::new(None));
static LED_BLUE: UnShared<RefCell<Option<BlueUserLed>>> = UnShared::new(RefCell::new(None));

// Create scheduler
#[scheduler::new(task_count = 3, core_freq = 480_000_000)]
struct NonPreemptiveScheduler;

// Process runnables
fn led_red_process(_: EventMask) {
    if let Some(led_red) = LED_RED.borrow().borrow_mut().as_mut() {
        led_red.toggle();
    }
}

fn led_green_process(_: EventMask) {
    if let Some(led_green) = LED_GREEN.borrow().borrow_mut().as_mut() {
        led_green.toggle();
    }
}

fn led_blue_process(_: EventMask) {
    if let Some(led_blue) = LED_BLUE.borrow().borrow_mut().as_mut() {
        led_blue.toggle();
    }
}

#[entry]
fn main() -> ! {
    log_init!();

    let portenta_h7::Board {
        led_red,
        led_green,
        led_blue,
        ..
    } = portenta_h7::Board::take();

    LED_RED.borrow().replace(Some(led_red));
    LED_GREEN.borrow().replace(Some(led_green));
    LED_BLUE.borrow().replace(Some(led_blue));

    // Add tasks
    scheduler::add_task!(
        "led_red",
        None,                  // Init runnable
        Some(led_red_process), // Process runnable
        Some(1_000),           // Cycle period
        Some(3)                // Offset from startup
    );

    scheduler::add_task!(
        "led_green",
        None,
        Some(led_green_process),
        Some(1_000),
        Some(281)
    );

    scheduler::add_task!(
        "led_blue",
        None,
        Some(led_blue_process),
        Some(1_000),
        Some(523)
    );

    scheduler::launch!();

    loop {
        panic!("Not expected execution");
    }
}
