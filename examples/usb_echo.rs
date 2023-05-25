//! Example USB echo
//!
//! Uses a Usb CDC class to echo the data received by the host
//!

#![no_std]
#![no_main]

use cortex_m::singleton;
use portenta_h7::{
    log, log_init,
    user_led::{self, UserLed},
    UsbBus, USB,
};
use rtic::app;
use systick_monotonic::Systick;
use usb_device::prelude::*;
use usbd_serial::CdcAcmClass;

const USB_BUS_BUFFER_SIZE: usize = 1024;
static mut USB_BUS_BUFFER: [u32; USB_BUS_BUFFER_SIZE] = [0u32; USB_BUS_BUFFER_SIZE];
const USB_HS_MAX_PACKET_SIZE: usize = 512;
static mut USB_APP_BUFFER: [u8; USB_HS_MAX_PACKET_SIZE] = [0u8; USB_HS_MAX_PACKET_SIZE];

#[app(device = portenta_h7::hal::pac, peripherals = false)]
mod app {
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led_blue: user_led::Blue,
        usb_dev: UsbDevice<'static, UsbBus<USB>>,
        usb_serial_port: CdcAcmClass<'static, UsbBus<USB>>,
    }

    #[monotonic(binds = SysTick, default = true)]
    type MonoTimer = Systick<1_000>;

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        log_init!();

        let mono = Systick::new(cx.core.SYST, portenta_h7::CORE_FREQUENCY.raw());

        // Get board resources
        let portenta_h7::Board { led_blue, usb, .. } = portenta_h7::Board::take();

        // Init USB stack
        let usb_bus = singleton!(
            : usb_device::class_prelude::UsbBusAllocator<UsbBus<USB>> =
                UsbBus::new(usb, unsafe { &mut USB_BUS_BUFFER })
        )
        .unwrap();
        let usb_serial_port = usbd_serial::CdcAcmClass::new(usb_bus, USB_HS_MAX_PACKET_SIZE as u16);
        let usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x1234, 0xABCD))
            .manufacturer("example")
            .product("usb-echo")
            .serial_number("0123456789ABCDEF")
            .device_class(usbd_serial::USB_CLASS_CDC)
            .max_packet_size_0(64)
            .max_power(100)
            .build();

        (
            Shared {},
            Local {
                led_blue,
                usb_dev,
                usb_serial_port,
            },
            init::Monotonics(mono),
        )
    }

    #[task(binds = OTG_HS, local = [led_blue, usb_dev, usb_serial_port])]
    fn usb_process(cx: usb_process::Context) {
        let (usb_dev, usb_serial_port) = (cx.local.usb_dev, cx.local.usb_serial_port);
        let previous_state = usb_dev.state();

        // Trigger internal state machine. It should be called either from ISR on USB event,
        // or every 10 ms from normal execution context
        if usb_dev.poll(&mut [usb_serial_port]) {
            // Read from reception fifo
            match usb_serial_port.read_packet(unsafe { &mut USB_APP_BUFFER[..] }) {
                Ok(cnt) if cnt > 0 => {
                    #[cfg(debug_assertions)]
                    log!(
                        "Received {} bytes: {}",
                        cnt,
                        core::str::from_utf8(unsafe { &USB_APP_BUFFER[..cnt] })
                            .unwrap_or("not valid")
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

        // Signal enumeration status
        match usb_dev.state() {
            // Transition to enumeration
            UsbDeviceState::Configured if previous_state == UsbDeviceState::Addressed => {
                log!("Enumeration completed");
                cx.local.led_blue.set_on();
            }
            // Already enumerated
            UsbDeviceState::Configured => {}
            // Enumeration lost
            _ if previous_state == UsbDeviceState::Configured => {
                log!("Enumeration lost");
                cx.local.led_blue.set_off();
            }
            _ => (),
        }
    }
}
