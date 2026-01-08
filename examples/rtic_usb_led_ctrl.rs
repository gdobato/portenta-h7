//! Example USB LED Control
//!
//! Controls LEDs over USB using the USB Device Class (CDC) for communication.
//! The `Led` enum represents the different LEDs that can be controlled: Red (`0xAA`), Green (`0xBB`), and Blue (`0xCC`).
//! The `Action` enum represents the actions that can be performed on an LED: turning it on (`0x01`) or off (`0x02`).
//! To control an LED, send the hexadecimal value of the LED followed by the hexadecimal value of the action. For example, to turn the red LED on, send `0xAA 0x01`.
//!

#![no_std]
#![no_main]

use core::mem::size_of;
use defmt::{debug, error, info};
use portenta_h7::board::{
    self,
    non_async_impl::{Board, LedBlue, LedGreen, LedRed, UsbBusImpl},
};
use rtic::app;
use rtic_monotonics::systick::prelude::*;
use rtic_sync::{channel::*, make_channel};
use static_cell::StaticCell;
use usb_device::{class_prelude::UsbBusAllocator, device::UsbRev, prelude::*};
use usbd_serial::CdcAcmClass;

systick_monotonic!(Mono, 1000);

const CHANNEL_CAPACITY: usize = 3;
const CHUNK_SIZE: usize = 2;
type Msg = [u8; CHUNK_SIZE];

const USB_MAX_PACKET_SIZE: usize = 64;
const USB_BUS_BUFFER_SIZE: usize = 1024;

#[derive(Clone, Copy, Debug, defmt::Format)]
pub enum Led {
    Red = 0xAA,
    Green = 0xBB,
    Blue = 0xCC,
}

impl Led {
    pub const fn as_u8(&self) -> u8 {
        *self as u8
    }

    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0xAA => Some(Self::Red),
            0xBB => Some(Self::Green),
            0xCC => Some(Self::Blue),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, defmt::Format)]
pub enum Action {
    On = 0x01,
    Off = 0x02,
}

impl Action {
    pub const fn as_u8(&self) -> u8 {
        *self as u8
    }

    pub const fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::On),
            0x02 => Some(Self::Off),
            _ => None,
        }
    }
}

#[app(device = portenta_h7::hal::pac, peripherals = false, dispatchers = [SPI1])]
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
            led_blue,
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
                .product("usb-led-ctrl")
                .serial_number("0123456789ABCDEF")])
            .unwrap()
            .build();

        // Create a channel to communicate between tasks
        let (sender, receiver) = make_channel!(Msg, CHANNEL_CAPACITY);

        info!("Spawning tasks");
        let _ = led_control::spawn(led_red, led_green, led_blue, receiver);

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
        mut led_blue: LedBlue,
        mut receiver: Receiver<'static, Msg, CHANNEL_CAPACITY>,
    ) {
        loop {
            let msg = receiver.recv().await;
            match msg {
                Ok(data) => {
                    if let (Some(led), Some(action)) =
                        (Led::from_u8(data[0]), Action::from_u8(data[1]))
                    {
                        match (led, action) {
                            (Led::Red, Action::On) => led_red.on(),
                            (Led::Red, Action::Off) => led_red.off(),
                            (Led::Green, Action::On) => led_green.on(),
                            (Led::Green, Action::Off) => led_green.off(),
                            (Led::Blue, Action::On) => led_blue.on(),
                            (Led::Blue, Action::Off) => led_blue.off(),
                        }
                        debug!("Received: {:?} {:?}", led, action);
                    }
                }
                Err(_) => {
                    error!("Error receiving message");
                }
            }
        }
    }

    #[task(priority = 1, binds = OTG_HS, local = [usb_dev, usb_serial_port, sender])]
    fn usb_process(cx: usb_process::Context) {
        let (usb_dev, usb_serial_port, sender) =
            (cx.local.usb_dev, cx.local.usb_serial_port, cx.local.sender);

        // Check if there are events to process by CDC class
        if usb_dev.poll(&mut [usb_serial_port]) {
            let mut app_buff = [0u8; USB_MAX_PACKET_SIZE];

            // Read packets from reception FIFO
            if let Ok(cnt) = usb_serial_port.read_packet(&mut app_buff) {
                debug!("Received {} bytes: {:02X}", cnt, &app_buff[..cnt]);

                // Send data to led control task in chunks
                let chunks = app_buff[..cnt].chunks_exact(size_of::<Msg>());
                for chunk in chunks {
                    if let Ok(msg) = chunk.try_into() {
                        let _ = sender.try_send(msg);
                    }
                }
            }
        }
    }
}
