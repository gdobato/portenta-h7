//! Example USB echo
//!
//! Uses a Usb CDC class to echo the data received by the host
//!

#![no_std]
#![no_main]

use core::mem::MaybeUninit;

use cortex_m::singleton;
use portenta_h7::{
    board::{self, Board, UsbBus, USB},
    led::{self, Led},
    log, log_init,
};
use rtic::app;
use rtic_monotonics::systick::*;
use usb_device::prelude::*;
use usbd_serial::CdcAcmClass;

const USB_HS_MAX_PACKET_SIZE: usize = 64;

#[app(device = portenta_h7::hal::pac, peripherals = false)]
mod app {
    use super::*;

    const USB_BUS_BUFFER_SIZE: usize = 1024;
    static mut USB_BUS_BUFFER: MaybeUninit<[u32; USB_BUS_BUFFER_SIZE]> = MaybeUninit::uninit();

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led_blue: led::user::Blue,
        usb_dev: UsbDevice<'static, UsbBus<USB>>,
        usb_serial_port: CdcAcmClass<'static, UsbBus<USB>>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        log_init!();

        let systick_mono_token = rtic_monotonics::create_systick_token!();

        // Init usb buffer
        unsafe {
            let usb_buffer: &mut [MaybeUninit<u32>; USB_BUS_BUFFER_SIZE] =
                &mut *(core::ptr::addr_of_mut!(USB_BUS_BUFFER) as *mut _);
            for element in usb_buffer.iter_mut() {
                element.as_mut_ptr().write(0);
            }
        }

        Systick::start(
            cx.core.SYST,
            board::CORE_FREQUENCY.raw(),
            systick_mono_token,
        );

        // Get board resources
        let Board { led_blue, usb, .. } = Board::take();

        let usb_bus = singleton!(:usb_device::class_prelude::UsbBusAllocator<UsbBus<USB>> = UsbBus::new(usb, unsafe {USB_BUS_BUFFER.assume_init_mut()})).unwrap();

        let usb_serial_port = usbd_serial::CdcAcmClass::new(usb_bus, USB_HS_MAX_PACKET_SIZE as u16);
        let usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x1234, 0xABCD))
            .device_class(usbd_serial::USB_CLASS_CDC)
            .max_packet_size_0(64)
            .unwrap()
            .strings(&[StringDescriptors::default()
                .manufacturer("example")
                .product("usb-echo")
                .serial_number("0123456789ABCDEF")])
            .unwrap()
            .build();

        (
            Shared {},
            Local {
                led_blue,
                usb_dev,
                usb_serial_port,
            },
        )
    }

    #[task(binds = OTG_HS, local = [led_blue, usb_dev, usb_serial_port])]
    fn usb_process(cx: usb_process::Context) {
        let (usb_dev, usb_serial_port) = (cx.local.usb_dev, cx.local.usb_serial_port);
        let previous_state = usb_dev.state();

        // Trigger internal state machine. It should be called either from ISR on USB event,
        // or every 10 ms from normal execution context
        if usb_dev.poll(&mut [usb_serial_port]) {
            let mut app_buff = [0u8; USB_HS_MAX_PACKET_SIZE];

            // Read from reception fifo
            match usb_serial_port.read_packet(&mut app_buff) {
                Ok(cnt) if cnt > 0 => {
                    #[cfg(debug_assertions)]
                    log!(
                        "Received {} bytes: {}",
                        cnt,
                        core::str::from_utf8(&app_buff[..cnt]).unwrap_or("not valid")
                    );
                    // Send back received data
                    match usb_serial_port.write_packet(&app_buff[..cnt]) {
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
                cx.local.led_blue.on();
            }
            // Already enumerated
            UsbDeviceState::Configured => {}
            // Enumeration lost
            _ if previous_state == UsbDeviceState::Configured => {
                log!("Enumeration lost");
                cx.local.led_blue.off();
            }
            _ => (),
        }
    }
}
