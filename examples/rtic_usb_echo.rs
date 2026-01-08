//! Example USB echo
//!
//! Sets up the device to appear as a virtual serial port to the host.
//! When the host sends data to this virtual serial port, the device receives it and then sends (echoes) the same data back to the host.
//! Additionally, on enumeration complete, green LED is on, otherwise red LED is on.
//!

#![no_std]
#![no_main]

use defmt::{error, info};
use portenta_h7::board::{
    self,
    non_async_impl::{Board, LedGreen, LedRed, UsbBusImpl},
};
use rtic::app;
use rtic_monotonics::systick::prelude::*;
use rtic_sync::{channel::*, make_channel};
use static_cell::StaticCell;
use usb_device::{class_prelude::UsbBusAllocator, device::UsbRev, prelude::*};
use usbd_serial::CdcAcmClass;

systick_monotonic!(Mono, 1000);

#[derive(Clone, Copy, Debug, defmt::Format)]
enum EnumerationState {
    Complete,
    Lost,
}

const CHANNEL_CAPACITY: usize = 1;
type Msg = EnumerationState;

const USB_MAX_PACKET_SIZE: usize = 64;
const USB_BUS_BUFFER_SIZE: usize = 1024;

#[app(device = portenta_h7::hal::pac, peripherals = false)]
mod app {
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        usb_dev: UsbDevice<'static, UsbBusImpl>,
        usb_serial_port: CdcAcmClass<'static, UsbBusImpl>,
        sender: Sender<'static, Msg, CHANNEL_CAPACITY>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        info!("Init");

        Mono::start(cx.core.SYST, board::CORE_FREQUENCY.raw());

        // Get board resources
        let Board {
            led_red,
            led_green,
            usb,
            ..
        } = Board::take();

        // Init USB stack
        static USB_BUS_BUFFER: StaticCell<[u32; USB_BUS_BUFFER_SIZE]> = StaticCell::new();
        static USB_ALLOCATOR: StaticCell<UsbBusAllocator<UsbBusImpl>> = StaticCell::new();
        let usb_bus = USB_ALLOCATOR.init(UsbBusImpl::new(
            usb,
            USB_BUS_BUFFER.init([0; USB_BUS_BUFFER_SIZE]),
        ));
        let usb_serial_port = usbd_serial::CdcAcmClass::new(usb_bus, USB_MAX_PACKET_SIZE as u16);
        let usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x1234, 0xABCD))
            .usb_rev(UsbRev::Usb200)
            .composite_with_iads()
            .max_packet_size_0(64)
            .unwrap()
            .strings(&[StringDescriptors::default()
                .manufacturer("example")
                .product("usb-echo")
                .serial_number("0123456789ABCDEF")])
            .unwrap()
            .build();

        // Create a channel to communicate between tasks
        let (sender, receiver) = make_channel!(Msg, CHANNEL_CAPACITY);

        info!("Spawning tasks");
        let _ = led_control::spawn(led_red, led_green, receiver);

        (
            Shared {},
            Local {
                usb_dev,
                usb_serial_port,
                sender,
            },
        )
    }

    #[task(priority = 0)]
    async fn led_control(
        _cx: led_control::Context,
        mut led_red: LedRed,
        mut led_green: LedGreen,
        mut receiver: Receiver<'static, Msg, CHANNEL_CAPACITY>,
    ) {
        // Initial state
        led_red.on();

        loop {
            let msg = receiver.recv().await;
            match msg {
                Ok(data) => match data {
                    EnumerationState::Complete => {
                        info!("Enumeration complete");
                        led_red.off();
                        led_green.on();
                    }
                    EnumerationState::Lost => {
                        info!("Enumeration lost");
                        led_green.off();
                        led_red.on();
                    }
                },

                Err(_) => {
                    error!("Error receiving message");
                }
            }
        }
    }

    #[task(priority = 1, binds = OTG_HS, local = [usb_dev, usb_serial_port, sender])]
    fn usb_process(cx: usb_process::Context) {
        let (usb_dev, usb_serial_port) = (cx.local.usb_dev, cx.local.usb_serial_port);
        let previous_state = usb_dev.state();

        // Trigger internal state machine. It should be called either from ISR on USB event,
        // or every 10 ms from normal execution context
        if usb_dev.poll(&mut [usb_serial_port]) {
            let mut app_buff = [0u8; USB_MAX_PACKET_SIZE];

            // Read from reception fifo
            match usb_serial_port.read_packet(&mut app_buff) {
                Ok(cnt) if cnt > 0 => {
                    info!(
                        "Received {} bytes: {}",
                        cnt,
                        core::str::from_utf8(&app_buff[..cnt]).unwrap_or("not valid")
                    );
                    // Send back received data
                    match usb_serial_port.write_packet(&app_buff[..cnt]) {
                        Ok(_) => (),
                        Err(err) => {
                            error!("Error in transmission: {:?}", err as u8)
                        }
                    }
                }
                _ => (),
            }
        }

        // Signal enumeration status
        match usb_dev.state() {
            // Enumeration complete
            UsbDeviceState::Configured if previous_state == UsbDeviceState::Addressed => {
                let _ = cx.local.sender.try_send(EnumerationState::Complete);
            }

            // Enumeration lost
            state
                if previous_state == UsbDeviceState::Configured
                    && state != UsbDeviceState::Configured =>
            {
                let _ = cx.local.sender.try_send(EnumerationState::Lost);
            }
            _ => (),
        }
    }
}
