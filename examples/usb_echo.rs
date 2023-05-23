//! Example USB echo
//!
//! Uses a Usb CDC class to echo the data received by the host
//!

#![no_std]
#![no_main]

#[allow(unused_imports)]
use core::{cell::RefCell, str::from_utf8};
use cortex_m::singleton;
use non_preemptive_scheduler as scheduler;
use portenta_h7::{
    entry,
    interrupt::{interrupt, InterruptEnabler},
    log, log_init,
    user_led::{self, UserLed},
    UsbBus, USB,
};
use scheduler::{resources::UnShared, EventMask, Scheduler, Task};
use usb_device::{class_prelude::*, prelude::*};
use usbd_serial::CdcAcmClass;

// Events
const EVENT_USB_ENUMERATION_COMPLETED: EventMask = 0x00000001;
const EVENT_USB_ENUMERATION_LOST: EventMask = 0x00000002;

// Static interior mutable entities
static LED_BLUE: UnShared<RefCell<Option<user_led::Blue>>> = UnShared::new(RefCell::new(None));
static USB_SERIAL_PORT: UnShared<RefCell<Option<CdcAcmClass<UsbBus<USB>>>>> =
    UnShared::new(RefCell::new(None));
static USB_DEV: UnShared<RefCell<Option<UsbDevice<UsbBus<USB>>>>> =
    UnShared::new(RefCell::new(None));
// Static mutable entities
const USB_BUS_BUFFER_SIZE: usize = 1024;
static mut USB_BUS_BUFFER: [u32; USB_BUS_BUFFER_SIZE] = [0u32; USB_BUS_BUFFER_SIZE];
const USB_HS_MAX_PACKET_SIZE: usize = 512;
static mut USB_APP_BUFFER: [u8; USB_HS_MAX_PACKET_SIZE] = [0u8; USB_HS_MAX_PACKET_SIZE];

// Create scheduler
#[scheduler::new(task_count = 2, core_freq = 480_000_000)]
struct NonPreemptiveScheduler;

// Functions which are bound to task runnables
fn led_process(event_mask: EventMask) {
    match event_mask & (EVENT_USB_ENUMERATION_COMPLETED | EVENT_USB_ENUMERATION_LOST) {
        EVENT_USB_ENUMERATION_COMPLETED => {
            if let Some(led_blue) = LED_BLUE.borrow().borrow_mut().as_mut() {
                log!("Enumeration completed");
                led_blue.set_on();
            }
        }

        EVENT_USB_ENUMERATION_LOST => {
            if let Some(led_blue) = LED_BLUE.borrow().borrow_mut().as_mut() {
                log!("Enumeration lost");
                led_blue.set_off();
            }
        }
        _ => (),
    }
}

#[entry]
fn main() -> ! {
    log_init!();

    let portenta_h7::Board {
        mut cp,
        led_blue,
        usb,
        ..
    } = portenta_h7::Board::take();

    // Configure Usb CDC class
    usb.set_interrupt_priority(&mut cp, 3);
    usb.enable_interrupt();
    let usb_bus: &'static UsbBusAllocator<UsbBus<USB>> = singleton!(
        USB_BUS: UsbBusAllocator<UsbBus<USB>> = UsbBus::new(usb, unsafe { &mut USB_BUS_BUFFER })
    )
    .unwrap();
    USB_SERIAL_PORT
        .borrow()
        .replace(Some(usbd_serial::CdcAcmClass::new(
            usb_bus,
            USB_HS_MAX_PACKET_SIZE as u16,
        )));
    USB_DEV.borrow().replace(Some(
        UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x1234, 0xABCD))
            .manufacturer("example")
            .product("usb-echo")
            .serial_number("0123456789ABCDEF")
            .device_class(usbd_serial::USB_CLASS_CDC)
            .max_packet_size_0(64)
            .max_power(100)
            .build(),
    ));

    // Initialize LED to signal enumeration state
    LED_BLUE.borrow().replace(Some(led_blue));

    // Add tasks to scheduler and launch it
    scheduler::add_task!("led", None, Some(led_process), None, None);
    scheduler::launch!();

    loop {
        panic!("Not expected execution");
    }
}

#[interrupt]
fn OTG_HS() {
    if let (Some(usb_dev), Some(usb_serial_port)) = (
        USB_DEV.borrow().borrow_mut().as_mut(),
        USB_SERIAL_PORT.borrow().borrow_mut().as_mut(),
    ) {
        // Previous state before polling
        let previous_state = usb_dev.state();
        if usb_dev.poll(&mut [usb_serial_port]) {
            // Read from reception fifo.
            match usb_serial_port.read_packet(unsafe { &mut USB_APP_BUFFER[..] }) {
                Ok(cnt) if cnt > 0 => {
                    #[cfg(debug_assertions)]
                    log!(
                        "Received {} bytes: {}",
                        cnt,
                        from_utf8(unsafe { &USB_APP_BUFFER[..cnt] }).unwrap_or("not valid")
                    );
                    // Send back received data
                    match usb_serial_port.write_packet(unsafe { &USB_APP_BUFFER[..cnt] }) {
                        Ok(_) => (),
                        Err(err) => {
                            log!("Error in transmission: {:?}", err)
                        }
                    }
                }
                _ => (),
            }
        }

        // Current state after polling
        match usb_dev.state() {
            // Transition to enumeration
            UsbDeviceState::Configured if previous_state == UsbDeviceState::Addressed => {
                scheduler::set_task_event!("led", EVENT_USB_ENUMERATION_COMPLETED);
            }
            // Already enumerated
            UsbDeviceState::Configured => {}
            // Enumeration lost
            _ if previous_state == UsbDeviceState::Configured => {
                scheduler::set_task_event!("led", EVENT_USB_ENUMERATION_LOST);
            }
            _ => (),
        }
    }
}
