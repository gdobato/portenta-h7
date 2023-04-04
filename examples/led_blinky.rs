//! Example LED blinky

#![no_std]
#![no_main]

use core::cell::RefCell;
use non_preemptive_scheduler::{resources::UnShared, EventMask, Scheduler, Task};
use non_preemptive_scheduler_macros as scheduler;
use portenta_rs::{BlueUserLed, GreenUserLed, RedUserLed, UserLeds};

// Static and interior mutable entities
static RED_LED: UnShared<RefCell<Option<RedUserLed>>> = UnShared::new(RefCell::new(None));
static GREEN_LED: UnShared<RefCell<Option<GreenUserLed>>> = UnShared::new(RefCell::new(None));
static BLUE_LED: UnShared<RefCell<Option<BlueUserLed>>> = UnShared::new(RefCell::new(None));

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
        portenta_rs::log!("blinky");
        blue_led.toggle();
    }
}

#[portenta_rs::entry]
fn main() -> ! {
    let board = portenta_rs::Board::init().unwrap();

    // User LEDs
    let UserLeds { red, green, blue } = board.user_leds;

    RED_LED.borrow().replace(Some(red));
    GREEN_LED.borrow().replace(Some(green));
    BLUE_LED.borrow().replace(Some(blue));

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
